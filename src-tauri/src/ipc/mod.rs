use crate::adapters::codex::{
    CodexAdapter, CodexError, CodexErrorKind, CodexStreamEvent, CodexStreamEventKind,
};
use crate::adapters::sqlite_repository::{Library, LibraryError};
use crate::application::{self, ApplicationError, ImportPdfResult, SearchResult};
use crate::domain::{DomainErrorKind, LibraryAction, LibraryState, ReviewKind};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::{Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;
use ts_rs::TS;

pub(crate) struct AppState {
    pub(crate) library: Mutex<Library>,
    pub(crate) codex_cancellations: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

#[derive(Clone, Copy, Debug, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub(crate) enum IpcErrorCode {
    Validation,
    NotFound,
    Conflict,
    DataIntegrity,
    Persistence,
    Filesystem,
    ExternalProcess,
    Cancelled,
    Internal,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub(crate) struct CommandError {
    code: IpcErrorCode,
    message: String,
}

impl CommandError {
    fn library_access() -> Self {
        Self {
            code: IpcErrorCode::Internal,
            message: "Не удалось получить доступ к личной библиотеке".into(),
        }
    }

    fn persistence(action: &'static str, error: std::io::Error) -> Self {
        Self {
            code: IpcErrorCode::Persistence,
            message: format!("Не удалось {action}: {error}"),
        }
    }

    fn from_library(error: LibraryError) -> Self {
        match error {
            LibraryError::Domain(error) => {
                let code = match error.kind() {
                    DomainErrorKind::Validation => IpcErrorCode::Validation,
                    DomainErrorKind::NotFound => IpcErrorCode::NotFound,
                    DomainErrorKind::Conflict => IpcErrorCode::Conflict,
                    DomainErrorKind::DataIntegrity => IpcErrorCode::DataIntegrity,
                };
                let message = error.into_message();
                Self { code, message }
            }
            LibraryError::Io(error) => Self::persistence("изменить личную библиотеку", error),
        }
    }

    fn from_application(error: ApplicationError, persistence_action: &'static str) -> Self {
        match error {
            ApplicationError::Domain(error) => Self::from_library(LibraryError::Domain(error)),
            ApplicationError::Persistence(error) => Self::persistence(persistence_action, error),
        }
    }

    fn from_codex(error: CodexError) -> Self {
        Self {
            code: match error.kind() {
                CodexErrorKind::Cancelled => IpcErrorCode::Cancelled,
                _ => IpcErrorCode::ExternalProcess,
            },
            message: error.into_message(),
        }
    }
}

#[tauri::command]
pub(crate) fn load_library(
    _app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, CommandError> {
    let library = state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?;
    library
        .load()
        .map_err(|error| CommandError::persistence("открыть личную библиотеку", error))
}

#[tauri::command]
pub(crate) fn execute_library_action(
    action: LibraryAction,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, CommandError> {
    state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?
        .apply(action)
        .map_err(CommandError::from_library)
}

#[tauri::command]
pub(crate) fn delete_book(
    book_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, CommandError> {
    let library = state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?;
    application::delete_book(
        &library.reading_storage(),
        &*library,
        &application::SystemClock,
        &application::SystemIdGenerator,
        &book_id,
    )
    .map_err(|error| CommandError::from_application(error, "удалить книгу"))
}

#[tauri::command]
pub(crate) fn import_pdf(
    path: String,
    title: String,
    state: tauri::State<'_, AppState>,
) -> Result<ImportPdfResult, CommandError> {
    let library = state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?;
    let storage = library.reading_storage();
    application::import_pdf(
        &storage,
        &*library,
        &application::SystemIdGenerator,
        path,
        title,
    )
    .map_err(|error| CommandError::from_application(error, "импортировать PDF"))
}

#[tauri::command]
pub(crate) fn search_library(
    query: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SearchResult>, CommandError> {
    let library = state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?;
    application::search_library(&*library, &query)
        .map_err(|error| CommandError::from_application(error, "выполнить локальный поиск"))
}

#[tauri::command]
pub(crate) fn book_file_path(
    book_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, CommandError> {
    let library = state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?;
    let snapshot = library
        .load()
        .map_err(|error| CommandError::persistence("открыть книгу", error))?;
    let book = snapshot
        .books
        .iter()
        .find(|book| book.id == book_id)
        .ok_or(CommandError {
            code: IpcErrorCode::NotFound,
            message: "Книга не найдена".into(),
        })?;
    Ok(library
        .absolute_book_path(&book.stored_file)
        .to_string_lossy()
        .into_owned())
}

#[tauri::command]
pub(crate) fn export_library_archive(
    path: String,
    password: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), CommandError> {
    let library = state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?;
    let archive = library.archive_storage();
    application::export_archive(&archive, &*library, path, &password)
        .map_err(|error| CommandError::from_application(error, "экспортировать архив"))
}

#[tauri::command]
pub(crate) fn import_library_archive(
    path: String,
    password: String,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, CommandError> {
    let library = state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?;
    let archive = library.archive_storage();
    application::import_archive(&archive, &*library, path, &password)
        .map_err(|error| CommandError::from_application(error, "импортировать архив"))
}

#[tauri::command]
pub(crate) fn restore_latest_snapshot(
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, CommandError> {
    let library = state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?;
    let archive = library.archive_storage();
    application::restore_latest_snapshot(&archive, &*library)
        .map_err(|error| CommandError::from_application(error, "восстановить снимок"))
}

#[tauri::command]
pub(crate) fn export_material_markdown(
    material_id: String,
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), CommandError> {
    let library = state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?;
    application::export_material(&library.text_file_storage(), &*library, &material_id, path)
        .map_err(|error| CommandError::from_application(error, "экспортировать материал"))
}

#[tauri::command]
pub(crate) fn export_draft_markdown(
    draft_id: String,
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, CommandError> {
    let library = state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?;
    application::export_draft(
        &library.text_file_storage(),
        &*library,
        &application::SystemClock,
        &application::SystemIdGenerator,
        &draft_id,
        path,
    )
    .map_err(|error| CommandError::from_application(error, "экспортировать черновую заметку"))
}

#[tauri::command]
pub(crate) fn export_diagnostic_log(
    path: String,
    entries: Vec<String>,
) -> Result<(), CommandError> {
    application::export_diagnostics(
        &crate::adapters::sqlite_repository::TextFileStorage,
        path,
        &entries,
    )
    .map_err(|error| CommandError::from_application(error, "экспортировать диагностический журнал"))
}

#[tauri::command]
pub(crate) async fn install_signed_update(app: tauri::AppHandle) -> Result<bool, CommandError> {
    let updater = app.updater().map_err(|error| CommandError {
        code: IpcErrorCode::ExternalProcess,
        message: format!("Проверка обновления недоступна: {error}"),
    })?;
    let Some(update) = updater.check().await.map_err(|error| CommandError {
        code: IpcErrorCode::ExternalProcess,
        message: format!("Не удалось проверить подписанное обновление: {error}"),
    })?
    else {
        return Ok(false);
    };
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|error| CommandError {
            code: IpcErrorCode::ExternalProcess,
            message: format!("Обновление отклонено или не установлено: {error}"),
        })?;
    Ok(true)
}

#[tauri::command]
pub(crate) fn prepare_codex_review(
    idea_id: String,
    request_kind: ReviewKind,
    recall_answer: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<String, CommandError> {
    state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?
        .prepare_review_package(&idea_id, request_kind, recall_answer.as_deref())
        .map_err(CommandError::from_library)
}

#[tauri::command]
pub(crate) async fn run_codex_review(
    app: tauri::AppHandle,
    request_id: String,
    idea_id: String,
    request_kind: ReviewKind,
    recall_answer: Option<String>,
    approved_package: String,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, CommandError> {
    let package = {
        let library = state
            .library
            .lock()
            .map_err(|_| CommandError::library_access())?;
        library
            .validate_review_request_id(&request_id)
            .map_err(CommandError::from_library)?;
        library
            .approve_review_package(
                &idea_id,
                request_kind,
                recall_answer.as_deref(),
                &approved_package,
            )
            .map_err(CommandError::from_library)?
    };
    let cancellation = Arc::new(AtomicBool::new(false));
    state
        .codex_cancellations
        .lock()
        .map_err(|_| CommandError::library_access())?
        .insert(request_id.clone(), cancellation.clone());
    let data_dir = app.path().app_data_dir().map_err(|error| CommandError {
        code: IpcErrorCode::Filesystem,
        message: format!("Не удалось подготовить изолированное состояние Codex: {error}"),
    })?;
    let adapter = CodexAdapter::bundled(&data_dir).map_err(|error| CommandError {
        code: IpcErrorCode::Filesystem,
        message: format!("Не удалось подготовить изолированное состояние Codex: {error}"),
    })?;
    let event_app = app.clone();
    let terminal_event_app = app.clone();
    let result = adapter
        .review(
            &request_id,
            &package,
            cancellation,
            move |event: CodexStreamEvent| {
                let _ = event_app.emit("codex-review-event", event);
            },
        )
        .await;
    state
        .codex_cancellations
        .lock()
        .map_err(|_| CommandError::library_access())?
        .remove(&request_id);
    let response = match result {
        Ok(response) => {
            let _ = terminal_event_app.emit(
                "codex-review-event",
                CodexStreamEvent {
                    request_id: request_id.clone(),
                    kind: CodexStreamEventKind::Completed,
                    text: String::new(),
                },
            );
            response
        }
        Err(error) => {
            let kind = if error.kind() == CodexErrorKind::Cancelled {
                CodexStreamEventKind::Cancelled
            } else {
                CodexStreamEventKind::Error
            };
            let text = error.to_string();
            let _ = terminal_event_app.emit(
                "codex-review-event",
                CodexStreamEvent {
                    request_id: request_id.clone(),
                    kind,
                    text,
                },
            );
            return Err(CommandError::from_codex(error));
        }
    };
    state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?
        .apply(LibraryAction::RecordReviewResponse {
            idea_id,
            request_kind,
            response,
        })
        .map_err(CommandError::from_library)
}

#[tauri::command]
pub(crate) async fn start_codex_login(app: tauri::AppHandle) -> Result<(), CommandError> {
    let data_dir = app.path().app_data_dir().map_err(|error| CommandError {
        code: IpcErrorCode::Filesystem,
        message: format!("Не удалось подготовить изолированное состояние Codex: {error}"),
    })?;
    let adapter = CodexAdapter::bundled(&data_dir).map_err(|error| CommandError {
        code: IpcErrorCode::Filesystem,
        message: format!("Не удалось подготовить изолированное состояние Codex: {error}"),
    })?;
    let event_app = app.clone();
    let result = adapter
        .login(move |event| {
            let _ = event_app.emit("codex-login-event", event);
        })
        .await;
    match result {
        Ok(()) => {
            let _ = app.emit(
                "codex-login-event",
                CodexStreamEvent {
                    request_id: "login".into(),
                    kind: CodexStreamEventKind::Completed,
                    text: String::new(),
                },
            );
            Ok(())
        }
        Err(error) => {
            let text = error.to_string();
            let _ = app.emit(
                "codex-login-event",
                CodexStreamEvent {
                    request_id: "login".into(),
                    kind: CodexStreamEventKind::Error,
                    text,
                },
            );
            Err(CommandError::from_codex(error))
        }
    }
}

#[tauri::command]
pub(crate) fn cancel_codex_review(
    request_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), CommandError> {
    let cancellations = state
        .codex_cancellations
        .lock()
        .map_err(|_| CommandError::library_access())?;
    let cancellation = cancellations.get(&request_id).ok_or(CommandError {
        code: IpcErrorCode::NotFound,
        message: "Активная проверка не найдена".into(),
    })?;
    cancellation.store(true, Ordering::Relaxed);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DomainError;

    #[test]
    fn domain_persistence_filesystem_and_process_errors_have_stable_codes() {
        let validation = CommandError::from_library(LibraryError::Domain(DomainError::new(
            "recall_invalid",
            "Напишите ответ и выберите самооценку",
        )));
        let persistence = CommandError::from_library(LibraryError::Io(std::io::Error::other("db")));
        let filesystem = CommandError {
            code: IpcErrorCode::Filesystem,
            message: "Не удалось открыть файл".into(),
        };
        let process = CommandError::from_codex(CodexError::new(
            "codex_crashed",
            "Codex завершился до ответа",
        ));

        assert!(matches!(validation.code, IpcErrorCode::Validation));
        assert!(matches!(persistence.code, IpcErrorCode::Persistence));
        assert!(matches!(filesystem.code, IpcErrorCode::Filesystem));
        assert!(matches!(process.code, IpcErrorCode::ExternalProcess));
        assert_eq!(
            serde_json::to_value(validation).unwrap(),
            serde_json::json!({
                "code": "validation",
                "message": "Напишите ответ и выберите самооценку"
            })
        );
    }

    #[test]
    fn not_found_and_cancelled_are_distinct_machine_results() {
        assert!(matches!(
            CommandError::from_library(LibraryError::Domain(DomainError::new(
                "draft_not_found",
                "Черновая заметка не найдена",
            )))
            .code,
            IpcErrorCode::NotFound,
        ));
        assert!(matches!(
            CommandError::from_codex(CodexError::new("codex_cancelled", "Проверка отменена")).code,
            IpcErrorCode::Cancelled
        ));
    }
}
