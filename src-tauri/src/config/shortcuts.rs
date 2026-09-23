//! The user's shortcut database, keyed by application id.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "shortcuts.json";

/// Shipped so a fresh install already shows something useful.
const DEFAULTS: &str = include_str!("defaults/chrome.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut {
    pub action: String,
    pub keys: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppEntry {
    pub display_name: String,
    pub shortcuts: Vec<Shortcut>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutDb {
    /// Keyed by application id: the X11 `WM_CLASS` class on Linux.
    pub apps: BTreeMap<String, AppEntry>,
}

impl ShortcutDb {
    fn path(dir: &Path) -> PathBuf {
        dir.join(FILE_NAME)
    }

    /// Loads the database, writing the bundled defaults first if the user has
    /// none yet.
    pub fn load_or_seed(dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::path(dir);
        if !path.exists() {
            std::fs::write(&path, DEFAULTS)?;
        }
        Ok(serde_json::from_str(&std::fs::read_to_string(&path)?)?)
    }

    pub fn save(&self, dir: &Path) -> Result<(), String> {
        let raw = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(Self::path(dir), raw).map_err(|e| e.to_string())
    }

    /// Window classes vary in case between toolkits (`google-chrome` vs
    /// `Google-chrome`), so lookups ignore case.
    pub fn get(&self, app_id: &str) -> Option<&AppEntry> {
        self.apps
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(app_id))
            .map(|(_, entry)| entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_defaults_parse_and_contain_chrome() {
        let db: ShortcutDb = serde_json::from_str(DEFAULTS).unwrap();
        let chrome = db.get("Google-chrome").expect("chrome defaults");
        assert_eq!(chrome.display_name, "Google Chrome");
        assert!(chrome.shortcuts.len() >= 20);
    }

    #[test]
    fn lookup_ignores_case() {
        let db: ShortcutDb = serde_json::from_str(DEFAULTS).unwrap();
        assert_eq!(db.get("google-chrome"), db.get("Google-chrome"));
        assert!(db.get("xterm").is_none());
    }

    #[test]
    fn seeds_defaults_then_reads_user_edits() {
        let dir = tempfile::tempdir().unwrap();
        let seeded = ShortcutDb::load_or_seed(dir.path()).unwrap();
        assert!(seeded.get("Google-chrome").is_some());

        let mut edited = ShortcutDb::default();
        edited.apps.insert(
            "xterm".to_string(),
            AppEntry {
                display_name: "XTerm".to_string(),
                shortcuts: vec![Shortcut {
                    action: "Paste".to_string(),
                    keys: "Shift+Insert".to_string(),
                }],
            },
        );
        edited.save(dir.path()).unwrap();

        let reloaded = ShortcutDb::load_or_seed(dir.path()).unwrap();
        assert_eq!(reloaded, edited);
    }
}
