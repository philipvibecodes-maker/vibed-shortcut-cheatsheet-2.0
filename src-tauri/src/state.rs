//! In-memory application state shared between the Tauri commands and the
//! focus-tracking thread.

use std::path::PathBuf;

use serde::Serialize;

use crate::config::settings::Settings;
use crate::config::shortcuts::{AppEntry, ShortcutDb};

/// The payload sent to the frontend whenever the cheatsheet contents change.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CheatsheetState {
    pub display_name: String,
    pub shortcuts: Vec<crate::config::shortcuts::Shortcut>,
    pub font_size: f32,
}

#[derive(Debug)]
pub struct AppState {
    pub config_dir: PathBuf,
    pub settings: Settings,
    pub shortcuts: ShortcutDb,
    /// The last application that had shortcuts to show. Kept so that focusing
    /// the cheatsheet itself does not blank the table.
    pub last_shown_app: Option<String>,
}

impl AppState {
    pub fn new(config_dir: PathBuf, settings: Settings, shortcuts: ShortcutDb) -> Self {
        Self {
            config_dir,
            settings,
            shortcuts,
            last_shown_app: None,
        }
    }

    fn current_entry(&self) -> Option<&AppEntry> {
        self.last_shown_app
            .as_deref()
            .and_then(|app_id| self.shortcuts.get(app_id))
    }

    /// The payload for the frontend. Empty when no tracked application has been
    /// focused yet; the window is hidden in that case.
    pub fn cheatsheet(&self) -> CheatsheetState {
        match self.current_entry() {
            Some(entry) => CheatsheetState {
                display_name: entry.display_name.clone(),
                shortcuts: entry.shortcuts.clone(),
                font_size: self.settings.font_size,
            },
            None => CheatsheetState {
                display_name: String::new(),
                shortcuts: Vec::new(),
                font_size: self.settings.font_size,
            },
        }
    }
}
