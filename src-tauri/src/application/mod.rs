use crate::domain::{new_id, now, Book, DomainError, LibraryAction, LibraryState};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub(crate) struct SearchResult {
    pub(crate) id: String,
    pub(crate) kind: SearchResultKind,
    pub(crate) title: String,
    pub(crate) context: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub(crate) struct ImportPdfResult {
    pub(crate) state: LibraryState,
    pub(crate) book_id: String,
    pub(crate) duplicate: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub(crate) enum SearchResultKind {
    Book,
    Draft,
    Idea,
    Topic,
    Material,
}

impl SearchResultKind {
    pub(crate) fn from_database(value: &str) -> Option<Self> {
        match value {
            "book" => Some(Self::Book),
            "draft" => Some(Self::Draft),
            "idea" => Some(Self::Idea),
            "topic" => Some(Self::Topic),
            "material" => Some(Self::Material),
            _ => None,
        }
    }
}

pub(crate) trait ReadingPort {
    fn store_pdf(&self, path: String, title: String, id: String) -> Result<Book, ApplicationError>;
    fn remove_pdf(&self, stored_file: &str) -> Result<(), std::io::Error>;
}

pub(crate) trait SearchPort {
    fn search(&self, query: &str) -> Result<Vec<SearchResult>, ApplicationError>;
}

pub(crate) trait ArchivePort {
    fn write_archive(
        &self,
        path: String,
        password: &str,
        state: &LibraryState,
    ) -> Result<(), ApplicationError>;
    fn read_archive(&self, path: String, password: &str) -> Result<LibraryState, ApplicationError>;
    fn read_latest_snapshot(&self) -> Result<LibraryState, ApplicationError>;
    fn rollback_import(&self, state: &LibraryState);
}

pub(crate) trait ExportPort {
    fn write_text(&self, path: String, contents: &str) -> Result<(), std::io::Error>;
}

pub(crate) fn import_pdf(
    port: &impl ReadingPort,
    repository: &impl LibraryRepository,
    ids: &impl IdGenerator,
    path: String,
    title: String,
) -> Result<ImportPdfResult, ApplicationError> {
    let book = port.store_pdf(path, title, ids.next_id("book"))?;
    let stored_file = book.stored_file.clone();
    let mut state = repository.load()?;
    if let Some(existing) = state.books.iter().find(|existing| {
        !book.content_hash.is_empty() && existing.content_hash == book.content_hash
    }) {
        let book_id = existing.id.clone();
        let _ = port.remove_pdf(&stored_file);
        return Ok(ImportPdfResult {
            state,
            book_id,
            duplicate: true,
        });
    }
    let book_id = book.id.clone();
    state.books.push(book);
    if let Err(error) = repository.commit(&state) {
        let _ = port.remove_pdf(&stored_file);
        return Err(ApplicationError::Persistence(error));
    }
    Ok(ImportPdfResult {
        state,
        book_id,
        duplicate: false,
    })
}

pub(crate) fn search_library(
    port: &impl SearchPort,
    query: &str,
) -> Result<Vec<SearchResult>, ApplicationError> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    port.search(query)
}

pub(crate) fn delete_book(
    files: &impl ReadingPort,
    repository: &impl LibraryRepository,
    clock: &impl Clock,
    ids: &impl IdGenerator,
    book_id: &str,
) -> Result<LibraryState, ApplicationError> {
    let original = repository.load()?;
    let stored_file = original
        .books
        .iter()
        .find(|book| book.id == book_id)
        .map(|book| book.stored_file.clone())
        .ok_or_else(|| DomainError::new("book_not_found", "Книга не найдена"))?;
    let mut state = original.clone();
    let mut make_id = |prefix: &str| ids.next_id(prefix);
    state.apply_with(
        LibraryAction::DeleteBook {
            book_id: book_id.into(),
        },
        clock.now(),
        &mut make_id,
    )?;
    repository.commit(&state)?;
    if let Err(error) = files.remove_pdf(&stored_file) {
        repository.commit(&original)?;
        return Err(ApplicationError::Persistence(error));
    }
    Ok(state)
}

pub(crate) fn export_archive(
    port: &impl ArchivePort,
    repository: &impl LibraryRepository,
    path: String,
    password: &str,
) -> Result<(), ApplicationError> {
    let mut state = repository.load()?;
    for review in &mut state.reviews {
        review.response.clear();
    }
    port.write_archive(path, password, &state)
}

pub(crate) fn import_archive(
    port: &impl ArchivePort,
    repository: &impl LibraryRepository,
    path: String,
    password: &str,
) -> Result<LibraryState, ApplicationError> {
    if repository.load()? != LibraryState::default() {
        return Err(DomainError::new(
            "archive_target_not_empty",
            "Импорт возможен только в пустую библиотеку: сохраните текущую библиотеку отдельно или используйте чистую установку",
        )
        .into());
    }
    let state = port.read_archive(path, password)?;
    if let Err(error) = repository.commit(&state) {
        port.rollback_import(&state);
        return Err(ApplicationError::Persistence(error));
    }
    Ok(state)
}

pub(crate) fn restore_latest_snapshot(
    port: &impl ArchivePort,
    repository: &impl LibraryRepository,
) -> Result<LibraryState, ApplicationError> {
    let state = port.read_latest_snapshot()?;
    repository.commit(&state)?;
    Ok(state)
}

pub(crate) fn export_material(
    port: &impl ExportPort,
    repository: &impl LibraryRepository,
    material_id: &str,
    path: String,
) -> Result<(), ApplicationError> {
    let state = repository.load()?;
    let material = state
        .materials
        .iter()
        .find(|item| item.id == material_id)
        .ok_or_else(|| DomainError::new("material_not_found", "Материал не найден"))?;
    let sources = material
        .idea_ids
        .iter()
        .filter_map(|id| state.ideas.iter().find(|idea| &idea.id == id))
        .map(|idea| {
            let book = state
                .books
                .iter()
                .find(|book| book.id == idea.book_id)
                .map(|book| book.title.as_str())
                .unwrap_or("Книга");
            format!("- {book}, {} — {}", idea.section, idea.formulation)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let markdown = format!("# {}\n\n## Проблема\n\n{}\n\n## Идея\n\n{}\n\n## Пример применения\n\n{}\n\n## Результат\n\n{}\n\n## Ограничения\n\n{}\n\n## Источники\n\n{}\n", material.title, material.problem, material.idea, material.example, material.result, material.limitations, sources);
    port.write_text(path, &markdown)?;
    Ok(())
}

pub(crate) fn export_draft(
    port: &impl ExportPort,
    repository: &impl LibraryRepository,
    clock: &impl Clock,
    ids: &impl IdGenerator,
    draft_id: &str,
    path: String,
) -> Result<LibraryState, ApplicationError> {
    let state = repository.load()?;
    let draft = state
        .drafts
        .iter()
        .find(|item| item.id == draft_id)
        .ok_or_else(|| DomainError::new("draft_not_found", "Черновая заметка не найдена"))?;
    let book = state
        .books
        .iter()
        .find(|book| book.id == draft.book_id)
        .map(|book| book.title.as_str())
        .unwrap_or("Книга");
    let markdown = format!(
        "# Черновая заметка\n\nИсточник: {book}, {}, стр. {}\n\n> {}\n\n{}\n",
        draft.section, draft.page, draft.excerpt, draft.comment
    );
    port.write_text(path, &markdown)?;
    execute_library_action(
        repository,
        clock,
        ids,
        LibraryAction::DiscardDraft {
            draft_id: draft_id.into(),
        },
    )
}

pub(crate) fn export_diagnostics(
    port: &impl ExportPort,
    path: String,
    entries: &[String],
) -> Result<(), ApplicationError> {
    let first = entries.len().saturating_sub(100);
    let contents = entries[first..].join("\n");
    port.write_text(path, &contents)?;
    Ok(())
}

pub(crate) trait LibraryRepository {
    fn load(&self) -> Result<LibraryState, std::io::Error>;
    fn commit(&self, state: &LibraryState) -> Result<(), std::io::Error>;
}

pub(crate) trait Clock {
    fn now(&self) -> u64;
}

pub(crate) trait IdGenerator {
    fn next_id(&self, prefix: &str) -> String;
}

pub(crate) struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> u64 {
        now()
    }
}

pub(crate) struct SystemIdGenerator;
impl IdGenerator for SystemIdGenerator {
    fn next_id(&self, prefix: &str) -> String {
        new_id(prefix)
    }
}

pub(crate) fn execute_library_action(
    repository: &impl LibraryRepository,
    clock: &impl Clock,
    ids: &impl IdGenerator,
    action: LibraryAction,
) -> Result<LibraryState, ApplicationError> {
    let mut state = repository.load()?;
    let timestamp = clock.now();
    let mut make_id = |prefix: &str| ids.next_id(prefix);
    state.apply_with(action, timestamp, &mut make_id)?;
    repository.commit(&state)?;
    Ok(state)
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ApplicationError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    Persistence(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Book, ReadingPosition};
    use std::cell::RefCell;

    struct MemoryRepository(RefCell<LibraryState>);

    impl LibraryRepository for MemoryRepository {
        fn load(&self) -> Result<LibraryState, std::io::Error> {
            Ok(self.0.borrow().clone())
        }

        fn commit(&self, state: &LibraryState) -> Result<(), std::io::Error> {
            self.0.replace(state.clone());
            Ok(())
        }
    }

    struct FixedClock;
    impl Clock for FixedClock {
        fn now(&self) -> u64 {
            1_700_000_000
        }
    }

    struct PredictableIds;
    impl IdGenerator for PredictableIds {
        fn next_id(&self, prefix: &str) -> String {
            format!("{prefix}-fixed")
        }
    }

    struct MemoryPdf;
    impl ReadingPort for MemoryPdf {
        fn store_pdf(
            &self,
            _path: String,
            title: String,
            id: String,
        ) -> Result<Book, ApplicationError> {
            Ok(Book {
                id,
                title,
                stored_file: "books/book-fixed.pdf".into(),
                has_text_layer: true,
                ..Book::default()
            })
        }

        fn remove_pdf(&self, _stored_file: &str) -> Result<(), std::io::Error> {
            Ok(())
        }
    }

    struct MemoryWriter(RefCell<String>);
    impl ExportPort for MemoryWriter {
        fn write_text(&self, _path: String, contents: &str) -> Result<(), std::io::Error> {
            self.0.replace(contents.into());
            Ok(())
        }
    }

    struct RecordingPdf {
        removed: RefCell<Vec<String>>,
        fail_removal: bool,
    }

    impl ReadingPort for RecordingPdf {
        fn store_pdf(
            &self,
            _path: String,
            _title: String,
            _id: String,
        ) -> Result<Book, ApplicationError> {
            unreachable!("not used by deletion")
        }

        fn remove_pdf(&self, stored_file: &str) -> Result<(), std::io::Error> {
            self.removed.borrow_mut().push(stored_file.into());
            if self.fail_removal {
                Err(std::io::Error::other("file is locked"))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn draft_capture_uses_injected_time_and_id_and_returns_one_snapshot() {
        let repository = MemoryRepository(RefCell::new(LibraryState {
            books: vec![Book {
                id: "book-1".into(),
                title: "Надёжные системы".into(),
                stored_file: "books/book-1.pdf".into(),
                has_text_layer: true,
                outline: vec![],
                reading: ReadingPosition::default(),
                reading_completed: false,
                retrospective: None,
                ..Book::default()
            }],
            ..LibraryState::default()
        }));

        let state = execute_library_action(
            &repository,
            &FixedClock,
            &PredictableIds,
            LibraryAction::CaptureDraft {
                book_id: "book-1".into(),
                section: "Глава 1".into(),
                page: 12,
                excerpt: "Репликация требует явных компромиссов".into(),
                context: String::new(),
                comment: String::new(),
            },
        )
        .expect("capture draft");

        assert_eq!(state.drafts[0].id, "draft-fixed");
        assert_eq!(state.drafts[0].created_at, 1_700_000_000);
        assert_eq!(*repository.0.borrow(), state);
    }

    #[test]
    fn pdf_import_orchestrates_storage_and_atomic_repository_commit() {
        let repository = MemoryRepository(RefCell::new(LibraryState::default()));
        let result = import_pdf(
            &MemoryPdf,
            &repository,
            &PredictableIds,
            "source.pdf".into(),
            "Надёжные системы".into(),
        )
        .unwrap();

        assert_eq!(result.book_id, "book-fixed");
        assert!(!result.duplicate);
        assert_eq!(result.state.books[0].title, "Надёжные системы");
        assert_eq!(*repository.0.borrow(), result.state);
    }

    #[test]
    fn draft_export_builds_markdown_then_commits_domain_removal() {
        let repository = MemoryRepository(RefCell::new(LibraryState {
            books: vec![Book::for_test("book", "Надёжные системы")],
            drafts: vec![crate::domain::DraftNote::for_test("draft", "book")],
            ..LibraryState::default()
        }));
        let writer = MemoryWriter(RefCell::new(String::new()));

        let state = export_draft(
            &writer,
            &repository,
            &FixedClock,
            &PredictableIds,
            "draft",
            "draft.md".into(),
        )
        .unwrap();

        assert!(writer.0.borrow().contains("Надёжные системы"));
        assert!(state.drafts.is_empty());
        assert_eq!(*repository.0.borrow(), state);
    }

    #[test]
    fn permanent_deletion_commits_dependencies_and_removes_the_stored_pdf() {
        let repository = MemoryRepository(RefCell::new(LibraryState {
            books: vec![Book {
                id: "book".into(),
                stored_file: "books/book.pdf".into(),
                ..Book::default()
            }],
            drafts: vec![crate::domain::DraftNote::for_test("draft", "book")],
            ..LibraryState::default()
        }));
        let files = RecordingPdf {
            removed: RefCell::new(Vec::new()),
            fail_removal: false,
        };

        let state = delete_book(&files, &repository, &FixedClock, &PredictableIds, "book").unwrap();

        assert!(state.books.is_empty());
        assert!(state.drafts.is_empty());
        assert_eq!(&*files.removed.borrow(), &["books/book.pdf"]);
    }

    #[test]
    fn permanent_deletion_restores_state_when_the_pdf_cannot_be_removed() {
        let original = LibraryState {
            books: vec![Book {
                id: "book".into(),
                stored_file: "books/book.pdf".into(),
                ..Book::default()
            }],
            ..LibraryState::default()
        };
        let repository = MemoryRepository(RefCell::new(original.clone()));
        let files = RecordingPdf {
            removed: RefCell::new(Vec::new()),
            fail_removal: true,
        };

        let result = delete_book(&files, &repository, &FixedClock, &PredictableIds, "book");

        assert!(matches!(result, Err(ApplicationError::Persistence(_))));
        assert_eq!(*repository.0.borrow(), original);
    }

    #[test]
    fn diagnostic_export_keeps_only_the_latest_hundred_local_entries() {
        let writer = MemoryWriter(RefCell::new(String::new()));
        let entries = (0..105)
            .map(|index| format!("entry-{index}"))
            .collect::<Vec<_>>();

        export_diagnostics(&writer, "diagnostics.log".into(), &entries).unwrap();

        let output = writer.0.borrow();
        assert!(!output.contains("entry-4\n"));
        assert!(output.starts_with("entry-5\n"));
        assert!(output.ends_with("entry-104"));
    }
}
