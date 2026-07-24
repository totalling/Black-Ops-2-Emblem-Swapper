use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::paths::{Paths, ACTIVE_NAME};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selection {
    pub group: String,
    pub slot: u32,
}

fn data_path(paths: &Paths) -> std::path::PathBuf {
    paths.group_dir(ACTIVE_NAME).join("selected.bin")
}

fn meta_path(paths: &Paths) -> std::path::PathBuf {
    paths.group_dir(ACTIVE_NAME).join("selected_meta.json")
}

pub fn read_selection(paths: &Paths) -> Result<Option<Selection>, AppError> {
    match std::fs::read(meta_path(paths)) {
        Ok(bytes) => match serde_json::from_slice(&bytes) {
            Ok(sel) => Ok(Some(sel)),
            Err(e) => {
                eprintln!("warning: ignoring invalid selection metadata: {e}");
                Ok(None)
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn select_emblem(paths: &Paths, group: &str, slot: u32) -> Result<(), AppError> {
    std::fs::create_dir_all(paths.group_dir(ACTIVE_NAME))?;
    let data = std::fs::read(crate::storage::slot_path(paths, group, slot))?;
    std::fs::write(data_path(paths), data)?;
    let meta = Selection {
        group: group.to_string(),
        slot,
    };
    std::fs::write(meta_path(paths), serde_json::to_vec(&meta)?)?;
    Ok(())
}

pub fn clear_selection(paths: &Paths) -> Result<(), AppError> {
    for p in [data_path(paths), meta_path(paths)] {
        if p.exists() {
            std::fs::remove_file(p)?;
        }
    }
    Ok(())
}

pub fn read_selected_data(paths: &Paths) -> Result<Option<Vec<u8>>, AppError> {
    match std::fs::read(data_path(paths)) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
