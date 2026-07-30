use crate::domain::{new_id, now, DomainError, LibraryAction, LibraryState, SessionStatus};
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

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub(crate) enum SearchResultKind {
    Book,
    Idea,
    Topic,
    Material,
}

impl SearchResultKind {
    pub(crate) fn from_database(value: &str) -> Option<Self> {
        match value {
            "book" => Some(Self::Book),
            "idea" => Some(Self::Idea),
            "topic" => Some(Self::Topic),
            "material" => Some(Self::Material),
            _ => None,
        }
    }
}

pub(crate) trait ReadingPort {
    fn import_pdf(&self, path: String, title: String) -> Result<LibraryState, ApplicationError>;
}

pub(crate) trait SearchPort {
    fn search(&self, query: &str) -> Result<Vec<SearchResult>, ApplicationError>;
}

pub(crate) trait ArchivePort {
    fn export_archive(&self, path: String, password: &str) -> Result<(), ApplicationError>;
    fn import_archive(
        &self,
        path: String,
        password: &str,
    ) -> Result<LibraryState, ApplicationError>;
    fn restore_latest_snapshot(&self) -> Result<LibraryState, ApplicationError>;
}

pub(crate) trait ExportPort {
    fn export_material(&self, material_id: &str, path: String) -> Result<(), ApplicationError>;
    fn export_draft(&self, draft_id: &str, path: String) -> Result<LibraryState, ApplicationError>;
}

pub(crate) fn import_pdf(
    port: &impl ReadingPort,
    path: String,
    title: String,
) -> Result<LibraryState, ApplicationError> {
    port.import_pdf(path, title)
}

pub(crate) fn search_library(
    port: &impl SearchPort,
    query: &str,
) -> Result<Vec<SearchResult>, ApplicationError> {
    port.search(query)
}

pub(crate) fn export_archive(
    port: &impl ArchivePort,
    path: String,
    password: &str,
) -> Result<(), ApplicationError> {
    port.export_archive(path, password)
}

pub(crate) fn import_archive(
    port: &impl ArchivePort,
    path: String,
    password: &str,
) -> Result<LibraryState, ApplicationError> {
    port.import_archive(path, password)
}

pub(crate) fn restore_latest_snapshot(
    port: &impl ArchivePort,
) -> Result<LibraryState, ApplicationError> {
    port.restore_latest_snapshot()
}

pub(crate) fn export_material(
    port: &impl ExportPort,
    material_id: &str,
    path: String,
) -> Result<(), ApplicationError> {
    port.export_material(material_id, path)
}

pub(crate) fn export_draft(
    port: &impl ExportPort,
    draft_id: &str,
    path: String,
) -> Result<LibraryState, ApplicationError> {
    port.export_draft(draft_id, path)
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
    let previous_debt = state.debt() as i32;
    let session_debt_baseline = match &action {
        LibraryAction::ResolveSession {
            session_id, status, ..
        } if *status == SessionStatus::Completed => state
            .sessions
            .iter()
            .find(|session| &session.id == session_id)
            .map(|session| session.debt_at_start as i32),
        _ => None,
    };
    let timestamp = clock.now();
    let mut make_id = |prefix: &str| ids.next_id(prefix);
    state.apply_with(action, timestamp, &mut make_id)?;
    state.last_debt_change = state.debt() as i32 - session_debt_baseline.unwrap_or(previous_debt);
    if state.last_debt_change != 0 {
        state.last_debt_changed_at = timestamp;
        state.debt_notification_sent_at = None;
    }
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
                study_completed: false,
                retrospective: None,
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
}
