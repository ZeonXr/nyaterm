use super::{get_config_dir, load_json, save_json, uuid_v4};
use crate::crypto;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

/// Managed password entry. The password field is AES-256-GCM encrypted on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPassword {
    #[serde(default = "uuid_v4")]
    pub id: String,
    pub name: String,
    /// Encrypted password on disk; plaintext only after `load_password_by_id`.
    #[serde(default)]
    pub password: Option<String>,
    /// Transient: true when encrypted password data exists on disk.
    #[serde(default, skip_serializing)]
    pub has_password: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PasswordsConfig {
    #[serde(default)]
    pub passwords: Vec<SavedPassword>,
}

pub fn load_passwords(app: &AppHandle) -> AppResult<PasswordsConfig> {
    let dir = get_config_dir(app)?;
    let path = dir.join("passwords.json");
    let mut config: PasswordsConfig = load_json(&path)?;
    for p in &mut config.passwords {
        p.has_password = p.password.is_some();
    }
    Ok(config)
}

pub fn save_passwords(app: &AppHandle, config: &PasswordsConfig) -> AppResult<()> {
    let dir = get_config_dir(app)?;
    save_json(&dir.join("passwords.json"), config)
}

pub fn load_password_by_id(app: &AppHandle, id: &str) -> AppResult<SavedPassword> {
    let cfg = load_passwords(app)?;
    let mut entry = cfg
        .passwords
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::Config(format!("Password '{}' not found", id)))?;
    if let Some(ct) = entry.password.clone() {
        entry.password = crypto::decrypt(&ct).ok();
    }
    Ok(entry)
}
