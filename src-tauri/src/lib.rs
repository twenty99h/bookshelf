mod library;

use library::{Library, LibraryState};
use serde::Serialize;
use std::sync::Mutex;
use tauri::Manager;

struct AppState {
    library: Mutex<Library>,
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
}

#[tauri::command]
fn load_library(state: tauri::State<'_, AppState>) -> Result<LibraryState, CommandError> {
    state
        .library
        .lock()
        .map_err(|_| CommandError::library_access())?
        .load()
        .map_err(|error| CommandError::persistence("открыть личную библиотеку", error))
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let library = Library::open(data_dir)?;
            app.manage(AppState {
                library: Mutex::new(library),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![load_library, save_workspace_note])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
