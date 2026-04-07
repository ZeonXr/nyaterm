//! Config persistence for sessions, UI, and quick commands.
//!
//! Stores JSON files in `~/.dragonfly/`. Credentials are AES-256-GCM encrypted in-place.

mod keys;
mod passwords;
mod quick_commands;
mod sessions;
mod settings;
mod ui;

pub use keys::*;
pub use passwords::*;
pub use quick_commands::*;
pub use sessions::*;
pub use settings::*;

use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub(crate) fn get_config_dir(app: &AppHandle) -> AppResult<PathBuf> {
    let home_dir = app
        .path()
        .home_dir()
        .map_err(|e| AppError::Config(e.to_string()))?;
    let config_dir = home_dir.join(".dragonfly");
    fs::create_dir_all(&config_dir)?;
    Ok(config_dir)
}

pub(crate) fn load_json<T: serde::de::DeserializeOwned + Default>(path: &PathBuf) -> AppResult<T> {
    if !path.exists() {
        return Ok(T::default());
    }
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

pub(crate) fn save_json<T: Serialize>(path: &PathBuf, data: &T) -> AppResult<()> {
    let content = serde_json::to_string_pretty(data)?;
    fs::write(path, content)?;
    Ok(())
}

pub(crate) fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_false() -> bool {
    false
}
