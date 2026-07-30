use crate::application::{
    execute_library_action, ApplicationError, ArchivePort, ExportPort, LibraryRepository,
    ReadingPort, SearchPort, SearchResult, SearchResultKind, SystemClock, SystemIdGenerator,
};
use crate::domain::*;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    iter,
    path::{Path, PathBuf},
};

mod archive;
mod reading;
mod search;

pub struct Library {
    data_dir: PathBuf,
    database_file: PathBuf,
}

pub(crate) struct ReadingStorage<'a>(&'a Library);
pub(crate) struct ArchiveStorage<'a>(&'a Library);
pub(crate) struct TextFileStorage;

#[derive(Deserialize, Serialize)]
struct ArchiveManifest {
    version: u32,
    state: LibraryState,
}

impl Library {
    pub(crate) fn reading_storage(&self) -> ReadingStorage<'_> {
        ReadingStorage(self)
    }

    pub(crate) fn archive_storage(&self) -> ArchiveStorage<'_> {
        ArchiveStorage(self)
    }

    pub(crate) fn text_file_storage(&self) -> TextFileStorage {
        TextFileStorage
    }

    pub fn open(data_dir: impl AsRef<Path>) -> io::Result<Self> {
        fs::create_dir_all(data_dir.as_ref())?;
        fs::create_dir_all(data_dir.as_ref().join("books"))?;
        fs::create_dir_all(data_dir.as_ref().join("snapshots"))?;
        let library = Self {
            data_dir: data_dir.as_ref().into(),
            database_file: data_dir.as_ref().join("library.sqlite3"),
        };
        library.migrate()?;
        let state = library.load()?;
        library.replace_state(&state)?;
        Ok(library)
    }

    pub fn load(&self) -> io::Result<LibraryState> {
        let connection = Connection::open(&self.database_file).map_err(sqlite_io)?;
        let json: Option<String> = connection
            .query_row("SELECT json FROM workspace_state WHERE id = 1", [], |row| {
                row.get(0)
            })
            .optional()
            .map_err(sqlite_io)?;
        json.map(|value| serde_json::from_str(&value).map_err(io::Error::other))
            .unwrap_or_else(|| Ok(LibraryState::default()))
    }

    pub fn validate_review_request_id(&self, request_id: &str) -> Result<(), LibraryError> {
        let valid_request_id = (16..=64).contains(&request_id.len())
            && request_id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-');
        if !valid_request_id {
            return Err(DomainError::new(
                "codex_request_invalid",
                "Некорректный идентификатор проверки",
            )
            .into());
        }
        Ok(())
    }

    pub fn prepare_review_package(
        &self,
        idea_id: &str,
        request_kind: ReviewKind,
        recall_answer: Option<&str>,
    ) -> Result<String, LibraryError> {
        if recall_answer.is_some_and(|answer| answer.chars().count() > 10_000) {
            return Err(DomainError::new(
                "codex_package_invalid",
                "Ответ для проверки слишком велик",
            )
            .into());
        }
        let state = self.load()?;
        let idea = find_idea(&state, idea_id)?;
        let book = find_book(&state, &idea.book_id)?;
        let fragment = idea.fragments.first();
        let source = format!(
            "Источник: {}, {}{}",
            book.title,
            idea.section,
            fragment
                .map(|fragment| format!(", стр. {}", fragment.page))
                .unwrap_or_default()
        );
        let related = (request_kind == ReviewKind::LinkSuggestion).then(|| {
            let candidates = state
                .ideas
                .iter()
                .filter(|candidate| candidate.id != idea.id)
                .map(|candidate| candidate.formulation.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            format!(
                "Кандидаты для сравнения: {}",
                if candidates.is_empty() {
                    "нет"
                } else {
                    &candidates
                }
            )
        });
        let question = match request_kind {
            ReviewKind::IdeaReview => {
                "При каких условиях моя формулировка неточна или неприменима?"
            }
            ReviewKind::RecallGaps => {
                "Какие существенные пробелы есть в моём ответе без выставления самооценки?"
            }
            ReviewKind::TopicSuggestion => {
                "Предложи одну подходящую тему знаний и объясни связь."
            }
            ReviewKind::LinkSuggestion => {
                "Предложи ровно одну наиболее обоснованную смысловую связь с одной из перечисленных идей."
            }
        };
        Ok([
            Some("Инструкция: укажи возможные пробелы и ограничения; не переписывай идею за читателя и не выставляй итоговую оценку.".to_owned()),
            Some(source),
            fragment.map(|fragment| format!("Выбранный фрагмент: {}", fragment.excerpt)),
            Some(format!("Авторская формулировка: {}", idea.formulation)),
            (request_kind == ReviewKind::RecallGaps)
                .then(|| format!("Ответ читателя: {}", recall_answer.unwrap_or_default())),
            related,
            Some(format!("Вопрос: {question}")),
            Some("Критерии ответа: точность, существенные ограничения, связь с показанным источником; никаких автоматических изменений.".to_owned()),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n\n"))
    }

    pub fn approve_review_package(
        &self,
        idea_id: &str,
        request_kind: ReviewKind,
        recall_answer: Option<&str>,
        approved_package: &str,
    ) -> Result<String, LibraryError> {
        let current = self.prepare_review_package(idea_id, request_kind, recall_answer)?;
        if current != approved_package {
            return Err(DomainError::new(
                "codex_package_changed",
                "Идея или источник изменились после подтверждения. Проверьте пакет ещё раз",
            )
            .into());
        }
        Ok(current)
    }

    pub fn absolute_book_path(&self, stored_file: &str) -> PathBuf {
        self.data_dir.join(stored_file)
    }

    pub fn apply(&self, action: LibraryAction) -> Result<LibraryState, LibraryError> {
        match execute_library_action(self, &SystemClock, &SystemIdGenerator, action) {
            Ok(state) => Ok(state),
            Err(ApplicationError::Domain(error)) => Err(LibraryError::Domain(error)),
            Err(ApplicationError::Persistence(error)) => Err(LibraryError::Io(error)),
        }
    }

    pub fn claim_debt_notification(&self) -> Result<Option<usize>, LibraryError> {
        let mut state = self.load()?;
        let quiet_period_seconds = u64::from(if state.debt_reminder_days == 0 {
            7
        } else {
            state.debt_reminder_days
        }) * 86_400;
        let due = state.debt() > 0
            && state.last_debt_changed_at > 0
            && now().saturating_sub(state.last_debt_changed_at) >= quiet_period_seconds
            && state.debt_notification_sent_at.is_none();
        if !due {
            return Ok(None);
        }
        let debt = state.debt();
        state.debt_notification_sent_at = Some(now());
        self.replace_state(&state)?;
        Ok(Some(debt))
    }

    fn create_snapshot(&self, state: &LibraryState) -> io::Result<()> {
        let dir = self.data_dir.join("snapshots");
        let path = dir.join(format!("snapshot-{}.json", unique_number()));
        let snapshot = without_transient_ai(state.clone());
        atomic_write(
            &path,
            &serde_json::to_vec(&snapshot).map_err(io::Error::other)?,
        )?;
        let mut snapshots: Vec<_> = fs::read_dir(&dir)?.filter_map(Result::ok).collect();
        snapshots.sort_by_key(|entry| entry.file_name());
        let remove_count = snapshots.len().saturating_sub(5);
        for entry in snapshots.into_iter().take(remove_count) {
            fs::remove_file(entry.path())?;
        }
        Ok(())
    }

    fn replace_state(&self, state: &LibraryState) -> io::Result<()> {
        let mut connection = Connection::open(&self.database_file).map_err(sqlite_io)?;
        let transaction = connection.transaction().map_err(sqlite_io)?;
        transaction.execute("INSERT INTO workspace_state (id, json) VALUES (1, ?1) ON CONFLICT(id) DO UPDATE SET json = excluded.json", [serde_json::to_string(state).map_err(io::Error::other)?]).map_err(sqlite_io)?;
        transaction
            .execute("DELETE FROM search_index", [])
            .map_err(sqlite_io)?;
        for book in &state.books {
            transaction.execute("INSERT INTO search_index (entity_id, kind, title, context) VALUES (?1, 'book', ?2, 'Название книги')", params![book.id, book.title]).map_err(sqlite_io)?;
        }
        for idea in &state.ideas {
            let book = book_title(state, &idea.book_id);
            let context = format!(
                "{book} · {} · {}",
                idea.section,
                idea.fragments
                    .iter()
                    .map(|item| format!("стр. {} · {} · {}", item.page, item.excerpt, item.context))
                    .collect::<Vec<_>>()
                    .join(" ")
            );
            transaction.execute("INSERT INTO search_index (entity_id, kind, title, context) VALUES (?1, 'idea', ?2, ?3)", params![idea.id, idea.formulation, context]).map_err(sqlite_io)?;
        }
        for topic in &state.topics {
            let context = state
                .ideas
                .iter()
                .filter(|idea| idea.topic_ids.contains(&topic.id))
                .map(|idea| {
                    let book = book_title(state, &idea.book_id);
                    format!("{book} · {} · {}", idea.section, idea.formulation)
                })
                .collect::<Vec<_>>()
                .join(" · ");
            transaction.execute("INSERT INTO search_index (entity_id, kind, title, context) VALUES (?1, 'topic', ?2, ?3)", params![topic.id, topic.name, context]).map_err(sqlite_io)?;
        }
        for material in &state.materials {
            let context = [
                material.problem.as_str(),
                material.idea.as_str(),
                material.example.as_str(),
                material.result.as_str(),
                material.limitations.as_str(),
            ]
            .into_iter()
            .filter(|part| !part.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" · ");
            transaction.execute("INSERT INTO search_index (entity_id, kind, title, context) VALUES (?1, 'material', ?2, ?3)", params![material.id, material.title, context]).map_err(sqlite_io)?;
        }
        transaction.commit().map_err(sqlite_io)
    }

    fn migrate(&self) -> io::Result<()> {
        let connection = Connection::open(&self.database_file).map_err(sqlite_io)?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY); CREATE TABLE IF NOT EXISTS workspace_state (id INTEGER PRIMARY KEY CHECK (id = 1), json TEXT NOT NULL); CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(entity_id UNINDEXED, kind UNINDEXED, title, context, tokenize='unicode61'); INSERT OR IGNORE INTO schema_migrations(version) VALUES (1);").map_err(sqlite_io)?;
        let legacy = self.data_dir.join("library.json");
        let has_state: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM workspace_state WHERE id = 1)",
                [],
                |row| row.get(0),
            )
            .map_err(sqlite_io)?;
        if !has_state && legacy.exists() {
            let state: LibraryState =
                serde_json::from_slice(&fs::read(&legacy)?).map_err(io::Error::other)?;
            drop(connection);
            self.replace_state(&state)?;
            fs::rename(&legacy, legacy.with_extension("json.migrated"))?;
        }
        Ok(())
    }
}

impl LibraryRepository for Library {
    fn load(&self) -> io::Result<LibraryState> {
        Library::load(self)
    }

    fn commit(&self, state: &LibraryState) -> io::Result<()> {
        self.replace_state(state)?;
        self.create_snapshot(state)
    }
}

fn application_error(error: LibraryError) -> ApplicationError {
    match error {
        LibraryError::Domain(error) => ApplicationError::Domain(error),
        LibraryError::Io(error) => ApplicationError::Persistence(error),
    }
}

impl ReadingPort for ReadingStorage<'_> {
    fn store_pdf(&self, path: String, title: String, id: String) -> Result<Book, ApplicationError> {
        self.0.store_pdf(path, title, id).map_err(application_error)
    }

    fn remove_pdf(&self, stored_file: &str) -> Result<(), io::Error> {
        remove_if_present(&self.0.absolute_book_path(stored_file))
    }
}

impl SearchPort for Library {
    fn search(&self, query: &str) -> Result<Vec<SearchResult>, ApplicationError> {
        Library::search(self, query).map_err(ApplicationError::Persistence)
    }
}

impl ArchivePort for ArchiveStorage<'_> {
    fn write_archive(
        &self,
        path: String,
        password: &str,
        state: &LibraryState,
    ) -> Result<(), ApplicationError> {
        self.0
            .write_archive(path, password, state)
            .map_err(application_error)
    }

    fn read_archive(&self, path: String, password: &str) -> Result<LibraryState, ApplicationError> {
        self.0
            .read_archive(path, password)
            .map_err(application_error)
    }

    fn read_latest_snapshot(&self) -> Result<LibraryState, ApplicationError> {
        self.0.read_latest_snapshot().map_err(application_error)
    }

    fn rollback_import(&self, state: &LibraryState) {
        for book in &state.books {
            let _ = remove_if_present(&self.0.absolute_book_path(&book.stored_file));
        }
    }
}

impl ExportPort for TextFileStorage {
    fn write_text(&self, path: String, contents: &str) -> Result<(), io::Error> {
        atomic_write(Path::new(&path), contents.as_bytes())
    }
}

fn without_transient_ai(mut state: LibraryState) -> LibraryState {
    for review in &mut state.reviews {
        review.response.clear();
    }
    state
}

#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Domain(#[from] DomainError),
}

fn atomic_write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temporary = path.with_extension("tmp");
    remove_if_present(&temporary)?;
    let mut file = fs::File::create(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(temporary, path)
}
fn crypto_io(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}
fn sqlite_io(error: rusqlite::Error) -> io::Error {
    io::Error::other(error)
}
fn cleanup_paths<'a>(paths: impl Iterator<Item = &'a PathBuf>) {
    for path in paths {
        let _ = remove_if_present(path);
    }
}
fn append_bytes<W: Write>(
    archive: &mut tar::Builder<W>,
    path: &str,
    bytes: &[u8],
) -> io::Result<()> {
    let mut header = tar::Header::new_gnu();
    header.set_path(path)?;
    header.set_size(bytes.len() as u64);
    header.set_mode(0o600);
    header.set_cksum();
    archive.append(&header, bytes)
}
fn remove_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests;
