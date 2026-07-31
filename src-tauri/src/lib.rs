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
            ipc::delete_book,
            ipc::import_pdf,
            ipc::search_library,
            ipc::book_file_path,
            ipc::export_library_archive,
            ipc::backup_metadata,
            ipc::import_library_archive,
            ipc::restore_latest_snapshot,
            ipc::export_material_markdown,
            ipc::export_draft_markdown,
            ipc::export_diagnostic_log,
            ipc::install_signed_update,
            ipc::prepare_codex_review,
            ipc::run_codex_review,
            ipc::cancel_codex_review,
            ipc::start_codex_login,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod native_smoke {
    use super::*;
    use serde_json::{json, Value};
    use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};

    fn invoke(
        webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        command: &str,
        body: Value,
    ) -> Value {
        get_ipc_response(
            webview,
            tauri::webview::InvokeRequest {
                cmd: command.into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.into(),
            },
        )
        .unwrap()
        .deserialize::<Value>()
        .unwrap()
    }

    fn app(data_dir: &std::path::Path) -> tauri::App<tauri::test::MockRuntime> {
        mock_builder()
            .manage(AppState {
                library: Mutex::new(Library::open(data_dir).unwrap()),
                codex_cancellations: Mutex::new(HashMap::new()),
            })
            .invoke_handler(tauri::generate_handler![
                ipc::import_pdf,
                ipc::book_file_path,
                ipc::execute_library_action,
            ])
            .build(mock_context(noop_assets()))
            .unwrap()
    }

    #[test]
    fn ipc_import_local_path_reader_position_and_restart_are_one_native_chain() {
        let data_dir = tempfile::tempdir().unwrap();
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pdf/text-layer.pdf");
        let native_app = app(data_dir.path());
        let webview = tauri::WebviewWindowBuilder::new(&native_app, "main", Default::default())
            .build()
            .unwrap();
        let imported = invoke(
            &webview,
            "import_pdf",
            json!({ "path": source, "title": "Native smoke" }),
        );
        let book_id = imported["bookId"].as_str().unwrap();
        let local_path = invoke(&webview, "book_file_path", json!({ "bookId": book_id }));
        assert!(std::path::Path::new(local_path.as_str().unwrap()).is_file());
        invoke(
            &webview,
            "execute_library_action",
            json!({ "action": { "kind": "updateReading", "bookId": book_id, "page": 2, "zoom": 1.35, "scroll": 0.64 } }),
        );
        drop(webview);
        drop(native_app);

        let restored = Library::open(data_dir.path()).unwrap().load().unwrap();
        assert_eq!(restored.books[0].reading.page, 2);
        assert_eq!(restored.books[0].reading.zoom, 1.35);
    }
}
