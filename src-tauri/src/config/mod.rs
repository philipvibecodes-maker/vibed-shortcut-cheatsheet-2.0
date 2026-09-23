//! On-disk configuration: window settings and the shortcut database.

pub mod settings;
pub mod shortcuts;

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

/// Directory holding `settings.json` and `shortcuts.json`, created if missing.
pub fn config_dir(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = app.path().app_config_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
