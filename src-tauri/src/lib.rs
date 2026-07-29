mod library;

use library::{Library, LibraryState};
use std::sync::Mutex;
use tauri::Manager;

struct AppState {
    library: Mutex<Library>,
}

#[tauri::command]
fn load_library(state: tauri::State<'_, AppState>) -> Result<LibraryState, String> {
    state
        .library
        .lock()
        .map_err(|_| "Не удалось получить доступ к личной библиотеке".to_string())?
        .load()
        .map_err(|error| format!("Не удалось открыть личную библиотеку: {error}"))
}

#[tauri::command]
fn save_workspace_note(
    note: String,
    state: tauri::State<'_, AppState>,
) -> Result<LibraryState, String> {
    state
        .library
        .lock()
        .map_err(|_| "Не удалось получить доступ к личной библиотеке".to_string())?
        .save_workspace_note(note)
        .map_err(|error| format!("Не удалось сохранить изменение: {error}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
