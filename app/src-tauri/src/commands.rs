use std::path::PathBuf;

use base64::Engine;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app_state::AppState;
use crate::broadcast::Selection;
use crate::state::Mode;
use crate::storage::EmblemInfo;

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    mode: String,
    selected: Option<Selection>,
}

#[tauri::command]
pub fn get_status(state: State<AppState>) -> Result<StatusResponse, String> {
    let mode = crate::state::read_state(&state.paths);
    let selected = crate::broadcast::read_selection(&state.paths).map_err(|e| e.0)?;
    Ok(StatusResponse {
        mode: mode.as_public_str().to_string(),
        selected,
    })
}

#[tauri::command]
pub fn set_mode(mode: String, state: State<AppState>) -> Result<(), String> {
    let internal = Mode::from_public_str(&mode).ok_or_else(|| "invalid mode".to_string())?;
    crate::state::write_state(&state.paths, internal).map_err(|e| e.0)
}

#[tauri::command]
pub fn list_emblems(state: State<AppState>) -> Result<Vec<EmblemInfo>, String> {
    crate::storage::list_emblems(&state.paths).map_err(|e| e.0)
}

#[tauri::command]
pub fn find_duplicate_emblems(state: State<AppState>) -> Result<Vec<Vec<EmblemInfo>>, String> {
    crate::storage::find_duplicate_groups(&state.paths).map_err(|e| e.0)
}

#[tauri::command]
pub async fn run_connection_diagnostics() -> Vec<crate::diagnostics::DiagnosticCheck> {
    crate::diagnostics::run().await
}

#[tauri::command]
pub fn backup_library(dest_path: String, state: State<AppState>) -> Result<usize, String> {
    crate::backup::export_library(&state.paths, &PathBuf::from(dest_path)).map_err(|e| e.0)
}

#[tauri::command]
pub fn restore_library(src_path: String, state: State<AppState>) -> Result<usize, String> {
    crate::backup::import_library(&state.paths, &PathBuf::from(src_path)).map_err(|e| e.0)
}

#[tauri::command]
pub fn render_emblem_png(group: String, slot: u32, state: State<AppState>) -> Result<String, String> {
    let path = crate::storage::slot_path(&state.paths, &group, slot);
    if !path.exists() {
        return Err("not found".to_string());
    }
    let cache_path = crate::storage::thumb_cache_path(&state.paths, &group, slot, 220);
    let bytes =
        crate::emblem::render::render_file_png_bytes_cached(&path, &cache_path, 220, &state.shapes)
            .map_err(|e| e.0)?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:image/png;base64,{encoded}"))
}

#[tauri::command]
pub fn set_emblem_label(
    group: String,
    slot: u32,
    label: String,
    state: State<AppState>,
) -> Result<(), String> {
    crate::storage::set_emblem_label(&state.paths, &group, slot, label.trim()).map_err(|e| e.0)
}

#[tauri::command]
pub fn select_emblem(group: String, slot: u32, state: State<AppState>) -> Result<(), String> {
    if !crate::storage::slot_exists(&state.paths, &group, slot) {
        return Err("not found".to_string());
    }
    crate::broadcast::select_emblem(&state.paths, &group, slot).map_err(|e| e.0)
}

#[tauri::command]
pub fn clear_selection(state: State<AppState>) -> Result<(), String> {
    crate::broadcast::clear_selection(&state.paths).map_err(|e| e.0)
}

fn unselect_if_matches(state: &AppState, group: &str, slot: u32) {
    if let Ok(Some(sel)) = crate::broadcast::read_selection(&state.paths) {
        if sel.group == group && sel.slot == slot {
            let _ = crate::broadcast::clear_selection(&state.paths);
        }
    }
}

#[tauri::command]
pub fn delete_emblem(group: String, slot: u32, state: State<AppState>) -> Result<(), String> {
    crate::storage::delete_emblem(&state.paths, &group, slot).map_err(|e| e.0)?;
    unselect_if_matches(&state, &group, slot);
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct EmblemRef {
    pub group: String,
    pub slot: u32,
}

#[tauri::command]
pub fn delete_emblems(items: Vec<EmblemRef>, state: State<AppState>) -> Result<(), String> {
    for item in &items {
        crate::storage::delete_emblem(&state.paths, &item.group, item.slot).map_err(|e| e.0)?;
        unselect_if_matches(&state, &item.group, item.slot);
    }
    Ok(())
}

#[tauri::command]
pub fn export_emblem(
    group: String,
    slot: u32,
    dest_path: String,
    state: State<AppState>,
) -> Result<(), String> {
    if !crate::storage::slot_exists(&state.paths, &group, slot) {
        return Err("not found".to_string());
    }
    crate::storage::export_emblem(&state.paths, &group, slot, &PathBuf::from(dest_path))
        .map_err(|e| e.0)
}

#[tauri::command]
pub fn import_emblem(
    src_path: String,
    label: Option<String>,
    state: State<AppState>,
) -> Result<EmblemInfo, String> {
    crate::storage::import_emblem(&state.paths, &PathBuf::from(src_path), label.as_deref())
        .map_err(|e| e.0)
}

#[derive(Debug, Serialize)]
pub struct NetworkInfo {
    lan_ip: Option<String>,
    proxy_port: u16,
}

#[tauri::command]
pub fn get_network_info() -> NetworkInfo {
    NetworkInfo {
        lan_ip: crate::netinfo::get_lan_ip(),
        proxy_port: crate::paths::PROXY_PORT,
    }
}
