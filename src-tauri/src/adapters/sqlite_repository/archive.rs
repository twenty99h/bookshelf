use super::*;

impl Library {
    pub(super) fn write_archive(
        &self,
        destination: impl AsRef<Path>,
        password: &str,
        state: &LibraryState,
    ) -> Result<(), LibraryError> {
        if password.chars().count() < 8 {
            return Err(DomainError::new(
                "archive_password_weak",
                "Пароль архива должен содержать не менее 8 символов",
            )
            .into());
        }
        let temporary = destination.as_ref().with_extension("age.tmp");
        remove_if_present(&temporary)?;
        let result = (|| {
            let output = fs::File::create(&temporary)?;
            let passphrase = age::secrecy::SecretString::from(password.to_owned());
            let encryptor = age::Encryptor::with_user_passphrase(passphrase);
            let encrypted = encryptor.wrap_output(output).map_err(crypto_io)?;
            let mut archive = tar::Builder::new(encrypted);
            let manifest = ArchiveManifest {
                version: 1,
                state: state.clone(),
            };
            append_bytes(
                &mut archive,
                "manifest.json",
                &serde_json::to_vec(&manifest).map_err(io::Error::other)?,
            )?;
            for book in &manifest.state.books {
                let source = self.absolute_book_path(&book.stored_file);
                if source.exists() {
                    archive.append_path_with_name(source, &book.stored_file)?;
                }
            }
            let encrypted = archive.into_inner()?;
            encrypted.finish().map_err(crypto_io)?;
            fs::rename(&temporary, destination.as_ref())?;
            Ok(())
        })();
        if result.is_err() {
            let _ = remove_if_present(&temporary);
        }
        result
    }

    pub(super) fn read_archive(
        &self,
        source: impl AsRef<Path>,
        password: &str,
    ) -> Result<LibraryState, LibraryError> {
        let input = fs::File::open(source)?;
        let decryptor = age::Decryptor::new(input).map_err(|_| {
            DomainError::new(
                "archive_corrupt",
                "Архив повреждён или имеет неизвестный формат",
            )
        })?;
        let passphrase = age::secrecy::SecretString::from(password.to_owned());
        let identity = age::scrypt::Identity::new(passphrase);
        let reader = decryptor
            .decrypt(iter::once(&identity as &dyn age::Identity))
            .map_err(|_| DomainError::new("archive_password_invalid", "Неверный пароль архива"))?;
        let staging = tempfile::tempdir()?;
        let mut archive = tar::Archive::new(reader);
        archive.unpack(staging.path()).map_err(|_| {
            DomainError::new("archive_corrupt", "Не удалось проверить целостность архива")
        })?;
        let manifest_bytes = fs::read(staging.path().join("manifest.json")).map_err(|_| {
            DomainError::new(
                "archive_corrupt",
                "В архиве отсутствует описание библиотеки",
            )
        })?;
        let manifest: ArchiveManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|_| DomainError::new("archive_corrupt", "Описание библиотеки повреждено"))?;
        if manifest.version != 1 {
            return Err(DomainError::new(
                "archive_version_unsupported",
                "Версия архива не поддерживается",
            )
            .into());
        }
        for book in &manifest.state.books {
            let staged = staging.path().join(&book.stored_file);
            if !staged.is_file() {
                return Err(
                    DomainError::new("archive_corrupt", "В архиве отсутствует файл книги").into(),
                );
            }
        }
        let mut prepared: Vec<(PathBuf, PathBuf)> = Vec::new();
        for book in &manifest.state.books {
            let target = self.absolute_book_path(&book.stored_file);
            if target.exists() {
                cleanup_paths(prepared.iter().map(|(temporary, _)| temporary));
                return Err(DomainError::new(
                    "archive_duplicates",
                    "Файл этой книги уже существует. Удалите оставшиеся данные или выберите чистую библиотеку",
                )
                .into());
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let temporary = target.with_extension(format!("pdf.importing-{}", unique_number()));
            if let Err(error) = fs::copy(staging.path().join(&book.stored_file), &temporary) {
                cleanup_paths(prepared.iter().map(|(path, _)| path));
                return Err(error.into());
            }
            prepared.push((temporary, target));
        }
        let mut committed: Vec<PathBuf> = Vec::new();
        for (temporary, target) in &prepared {
            if let Err(error) = fs::rename(temporary, target) {
                cleanup_paths(prepared.iter().map(|(path, _)| path));
                cleanup_paths(committed.iter());
                return Err(error.into());
            }
            committed.push(target.clone());
        }
        Ok(manifest.state)
    }

    pub(super) fn read_latest_snapshot(&self) -> Result<LibraryState, LibraryError> {
        let dir = self.data_dir.join("snapshots");
        let mut snapshots: Vec<_> = fs::read_dir(dir)?.filter_map(Result::ok).collect();
        snapshots.sort_by_key(|entry| entry.file_name());
        let latest = snapshots.last().ok_or_else(|| {
            DomainError::new(
                "snapshot_not_found",
                "Нет доступного снимка рабочего состояния",
            )
        })?;
        let state: LibraryState =
            serde_json::from_slice(&fs::read(latest.path())?).map_err(|_| {
                DomainError::new("snapshot_corrupt", "Снимок рабочего состояния повреждён")
            })?;
        let valid_sources = state
            .drafts
            .iter()
            .all(|draft| state.books.iter().any(|book| book.id == draft.book_id))
            && state
                .ideas
                .iter()
                .all(|idea| state.books.iter().any(|book| book.id == idea.book_id));
        if !valid_sources {
            return Err(DomainError::new(
                "snapshot_inconsistent",
                "Снимок содержит несогласованные источники",
            )
            .into());
        }
        Ok(state)
    }
}
