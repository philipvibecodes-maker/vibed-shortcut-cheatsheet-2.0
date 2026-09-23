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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::Corner;

    fn state() -> AppState {
        let mut shortcuts = ShortcutDb::default();
        shortcuts.apps.insert(
            "Google-chrome".to_string(),
            AppEntry {
                display_name: "Google Chrome".to_string(),
                shortcuts: vec![crate::config::shortcuts::Shortcut {
                    action: "New tab".to_string(),
                    keys: "Ctrl+T".to_string(),
                }],
            },
        );
        AppState::new(
            PathBuf::from("/tmp"),
            Settings {
                font_size: 14.0,
                corner: Corner::BottomLeft,
            },
            shortcuts,
        )
    }

    /// The `get_state` contract includes the window corner so the frontend can
    /// reflect where the window sits and reposition it.
    #[test]
    fn cheatsheet_payload_includes_the_corner() {
        let payload = serde_json::to_value(state().cheatsheet()).unwrap();
        assert_eq!(payload["corner"], "bottom_left");
    }

    /// With no app to show, the payload must mark the window hidden so the
    /// frontend does not render an empty frame.
    #[test]
    fn empty_cheatsheet_is_marked_hidden() {
        let payload = serde_json::to_value(state().cheatsheet()).unwrap();
        assert_eq!(payload["visible"], false);
    }

    /// A focused app with shortcuts marks the window visible and sends its
    /// table.
    #[test]
    fn shown_cheatsheet_is_marked_visible() {
        let mut state = state();
        state.last_shown_app = Some("Google-chrome".to_string());
        let payload = serde_json::to_value(state.cheatsheet()).unwrap();
        assert_eq!(payload["visible"], true);
        assert_eq!(payload["displayName"], "Google Chrome");
    }
}
