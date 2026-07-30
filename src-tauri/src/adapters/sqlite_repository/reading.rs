use super::*;

impl Library {
    pub(super) fn store_pdf(
        &self,
        source: impl AsRef<Path>,
        title: String,
        id: String,
    ) -> Result<Book, LibraryError> {
        let bytes = fs::read(source.as_ref())?;
        if !bytes.starts_with(b"%PDF") {
            return Err(DomainError::new("pdf_invalid", "Выбранный файл не является PDF").into());
        }
        let has_text_layer = bytes.windows(3).any(|part| part == b" BT")
            || bytes.windows(5).any(|part| part == b"/Font");
        if !has_text_layer {
            return Err(DomainError::new(
                "pdf_text_layer_missing",
                "В PDF нет пригодного текстового слоя. OCR пока не поддерживается",
            )
            .into());
        }
        let title = if title.trim().is_empty() {
            source
                .as_ref()
                .file_stem()
                .and_then(|item| item.to_str())
                .unwrap_or("Книга")
                .to_owned()
        } else {
            title
        };
        let relative = format!("books/{id}.pdf");
        let target = self.data_dir.join(&relative);
        let temporary = target.with_extension("pdf.tmp");
        fs::write(&temporary, bytes)?;
        fs::rename(&temporary, &target)?;
        Ok(Book {
            id,
            title,
            stored_file: relative,
            has_text_layer,
            ..Book::default()
        })
    }
}
