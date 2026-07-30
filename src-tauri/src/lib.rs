mod adapters;
mod application;
mod domain;
mod ipc;

use crate::adapters::sqlite_repository::Library;
use crate::ipc::AppState;
use std::{collections::HashMap, sync::Mutex};
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            ipc::load_library,
            ipc::execute_library_action,
            ipc::import_pdf,
            ipc::search_library,
            ipc::book_file_path,
            ipc::export_library_archive,
            ipc::import_library_archive,
            ipc::restore_latest_snapshot,
            ipc::export_material_markdown,
            ipc::export_draft_markdown,
            ipc::install_signed_update,
            ipc::prepare_codex_review,
            ipc::run_codex_review,
            ipc::cancel_codex_review,
            ipc::start_codex_login,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
