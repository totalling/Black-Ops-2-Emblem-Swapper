mod app_state;
mod backup;
mod broadcast;
mod commands;
mod diagnostics;
pub mod emblem;
mod error;
mod netinfo;
mod paths;
mod proxy;
mod state;
mod storage;

use std::sync::Arc;

use tauri::Manager;
use tokio::sync::Mutex as TokioMutex;

use app_state::AppState;
use emblem::render::ShapeCache;
use paths::Paths;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let resolved_paths = Paths::resolve(&handle)
                .map_err(|e| -> Box<dyn std::error::Error> { Box::new(std::io::Error::other(e.0)) })?;

            let shapes = Arc::new(ShapeCache::new(resolved_paths.shapes_dir.clone()));
            let capture_lock = Arc::new(TokioMutex::new(()));

            app.manage(AppState {
                paths: resolved_paths.clone(),
                shapes: shapes.clone(),
            });

            let proxy_app_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                proxy::serve(
                    Arc::new(resolved_paths),
                    capture_lock,
                    shapes,
                    proxy_app_handle,
                )
                .await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::set_mode,
            commands::list_emblems,
            commands::render_emblem_png,
            commands::set_emblem_label,
            commands::select_emblem,
            commands::clear_selection,
            commands::delete_emblem,
            commands::delete_emblems,
            commands::get_network_info,
            commands::export_emblem,
            commands::import_emblem,
            commands::find_duplicate_emblems,
            commands::run_connection_diagnostics,
            commands::backup_library,
            commands::restore_library,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
