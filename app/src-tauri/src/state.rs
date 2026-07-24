use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::paths::Paths;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    Passthrough,
    Capture,
    Inject,
}

impl Mode {
    pub fn as_internal_str(self) -> &'static str {
        match self {
            Mode::Passthrough => "PASSTHROUGH",
            Mode::Capture => "CAPTURE",
            Mode::Inject => "INJECT",
        }
    }

    pub fn from_internal_str(s: &str) -> Option<Mode> {
        match s.trim().to_uppercase().as_str() {
            "PASSTHROUGH" => Some(Mode::Passthrough),
            "CAPTURE" => Some(Mode::Capture),
            "INJECT" => Some(Mode::Inject),
            _ => None,
        }
    }

    pub fn as_public_str(self) -> &'static str {
        match self {
            Mode::Passthrough => "off",
            Mode::Capture => "capture",
            Mode::Inject => "inject",
        }
    }

    pub fn from_public_str(s: &str) -> Option<Mode> {
        match s {
            "off" => Some(Mode::Passthrough),
            "capture" => Some(Mode::Capture),
            "inject" => Some(Mode::Inject),
            _ => None,
        }
    }
}

pub fn read_state(paths: &Paths) -> Mode {
    match std::fs::read_to_string(&paths.state_file) {
        Ok(s) => Mode::from_internal_str(&s).unwrap_or(Mode::Passthrough),
        Err(_) => Mode::Passthrough,
    }
}

pub fn write_state(paths: &Paths, mode: Mode) -> Result<(), AppError> {
    std::fs::write(&paths.state_file, mode.as_internal_str())?;
    Ok(())
}

pub fn ensure_state_file(paths: &Paths) -> Result<(), AppError> {
    if !paths.state_file.exists() {
        write_state(paths, Mode::Passthrough)?;
    }
    Ok(())
}
