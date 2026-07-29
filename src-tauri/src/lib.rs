mod adapters;
mod library;

use adapters::codex::{CodexAdapter, CodexError, CodexStreamEvent};
use library::{Library, LibraryAction, LibraryError, LibraryState, ReviewKind, SearchResult};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::UpdaterExt;

struct AppState {
    library: Mutex<Library>,
    codex_cancellations: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

#[derive(Serialize)]
struct CommandError {
    code: &'static str,
    message: String,
}

impl CommandError {
    fn library_access() -> Self {
        Self {
            code: "library_access_failed",
            message: "Не удалось получить доступ к личной библиотеке".into(),
        }
    }

    fn persistence(action: &'static str, error: std::io::Error) -> Self {
        Self {
            code: "library_persistence_failed",
            message: format!("Не удалось {action}: {error}"),
        }
    }

    fn from_library(error: LibraryError) -> Self {
        match error {
            LibraryError::Domain(error) => Self {
                code: error.code,
                message: error.message,
            },
            LibraryError::Io(error) => Self::persistence("изменить личную библиотеку", error),
        }
    }

    fn from_codex(error: CodexError) -> Self {
        Self {
            code: error.code,
            message: error.message,
        }
    }
}

#[tauri::command]
fn load_library(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, CommandError> {
    let library = state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?;
    let snapshot = library
        .load()
        .map_err(|error| CommandError::persistence("открыть личную библиотеку", error))?;
    if let Ok(Some(debt)) = library.claim_debt_notification() {
        let _ = app
            .notification()
            .builder()
            .title("Bookshelf")
            .body(format!(
                "В очереди изучения {debt} действий. Выберите удобный следующий шаг."
            ))
            .show();
    }
    Ok(snapshot)
}

#[tauri::command]
fn save_workspace_note(
    note: String,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, CommandError> {
    if note.chars().count() > 240 {
        return Err(CommandError {
            code: "workspace_note_too_long",
            message: "Пометка не может быть длиннее 240 символов".into(),
        });
    }

    state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?
        .save_workspace_note(note)
        .map_err(|error| CommandError::persistence("сохранить изменение", error))
}

#[tauri::command]
fn execute_library_action(
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
fn import_pdf(
    path: String,
    title: String,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, CommandError> {
    state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?
        .import_pdf(path, title)
        .map_err(CommandError::from_library)
}

#[tauri::command]
fn search_library(
    query: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SearchResult>, CommandError> {
    state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?
        .search(&query)
        .map_err(|error| CommandError::persistence("выполнить локальный поиск", error))
}

#[tauri::command]
fn book_file_path(
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
            code: "book_not_found",
            message: "Книга не найдена".into(),
        })?;
    Ok(library
        .absolute_book_path(&book.stored_file)
        .to_string_lossy()
        .into_owned())
}

#[tauri::command]
fn export_library_archive(
    path: String,
    password: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), CommandError> {
    state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?
        .export_archive(path, &password)
        .map_err(CommandError::from_library)
}

#[tauri::command]
fn import_library_archive(
    path: String,
    password: String,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, CommandError> {
    state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?
        .import_archive(path, &password)
        .map_err(CommandError::from_library)
}

#[tauri::command]
fn restore_latest_snapshot(
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, CommandError> {
    state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?
        .restore_latest_snapshot()
        .map_err(CommandError::from_library)
}

#[tauri::command]
fn export_material_markdown(
    material_id: String,
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), CommandError> {
    state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?
        .export_material_markdown(&material_id, path)
        .map_err(CommandError::from_library)
}

#[tauri::command]
fn export_draft_markdown(
    draft_id: String,
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, CommandError> {
    state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?
        .export_draft_markdown(&draft_id, path)
        .map_err(CommandError::from_library)
}

#[tauri::command]
async fn install_signed_update(app: tauri::AppHandle) -> Result<bool, CommandError> {
    let updater = app.updater().map_err(|error| CommandError {
        code: "updater_unavailable",
        message: format!("Проверка обновления недоступна: {error}"),
    })?;
    let Some(update) = updater.check().await.map_err(|error| CommandError {
        code: "update_check_failed",
        message: format!("Не удалось проверить подписанное обновление: {error}"),
    })?
    else {
        return Ok(false);
    };
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|error| CommandError {
            code: "update_install_failed",
            message: format!("Обновление отклонено или не установлено: {error}"),
        })?;
    Ok(true)
}

#[tauri::command]
fn prepare_codex_review(
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
async fn run_codex_review(
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
        code: "codex_state_unavailable",
        message: format!("Не удалось подготовить изолированное состояние Codex: {error}"),
    })?;
    let adapter = CodexAdapter::bundled(&data_dir).map_err(|error| CommandError {
        code: "codex_state_unavailable",
        message: format!("Не удалось подготовить изолированное состояние Codex: {error}"),
    })?;
    let event_app = app.clone();
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
    let response = result.map_err(CommandError::from_codex)?;
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
async fn start_codex_login(app: tauri::AppHandle) -> Result<(), CommandError> {
    let data_dir = app.path().app_data_dir().map_err(|error| CommandError {
        code: "codex_state_unavailable",
        message: format!("Не удалось подготовить изолированное состояние Codex: {error}"),
    })?;
    let adapter = CodexAdapter::bundled(&data_dir).map_err(|error| CommandError {
        code: "codex_state_unavailable",
        message: format!("Не удалось подготовить изолированное состояние Codex: {error}"),
    })?;
    let event_app = app.clone();
    adapter
        .login(move |event| {
            let _ = event_app.emit("codex-login-event", event);
        })
        .await
        .map_err(CommandError::from_codex)
}

#[tauri::command]
fn cancel_codex_review(
    request_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), CommandError> {
    let cancellations = state
        .codex_cancellations
        .lock()
        .map_err(|_| CommandError::library_access())?;
    let cancellation = cancellations.get(&request_id).ok_or(CommandError {
        code: "codex_review_not_running",
        message: "Активная проверка не найдена".into(),
    })?;
    cancellation.store(true, Ordering::Relaxed);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let library = Library::open(data_dir)?;
            app.manage(AppState {
                library: Mutex::new(library),
                codex_cancellations: Mutex::new(HashMap::new()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_library,
            save_workspace_note,
            execute_library_action,
            import_pdf,
            search_library,
            book_file_path,
            export_library_archive,
            import_library_archive,
            restore_latest_snapshot,
            export_material_markdown,
            export_draft_markdown,
            install_signed_update,
            prepare_codex_review,
            run_codex_review,
            cancel_codex_review,
            start_codex_login,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
