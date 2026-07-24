use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::error::AppError;

pub const ACTIVE_NAME: &str = "_active";
pub const PROXY_HOST: &str = "0.0.0.0";
pub const PROXY_PORT: u16 = 8080;
pub const TARGET_HOST_SUBSTR: &str = "ops2-ps3-cs.prod.demonware.net";

#[derive(Clone)]
pub struct Paths {
    pub saved_dir: PathBuf,
    pub state_file: PathBuf,
    pub shapes_dir: PathBuf,
}

impl Paths {
    pub fn resolve(app: &AppHandle) -> Result<Self, AppError> {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::new(format!("could not resolve app data dir: {e}")))?;
        std::fs::create_dir_all(&data_dir)?;

        let resource_dir = app
            .path()
            .resource_dir()
            .map_err(|e| AppError::new(format!("could not resolve resource dir: {e}")))?;

        Ok(Paths {
            saved_dir: data_dir.join("saved"),
            state_file: data_dir.join("state.txt"),
            shapes_dir: resource_dir.join("resources").join("reference_shapes"),
        })
    }

    pub fn group_dir(&self, name: &str) -> PathBuf {
        self.saved_dir.join(name)
    }
}
