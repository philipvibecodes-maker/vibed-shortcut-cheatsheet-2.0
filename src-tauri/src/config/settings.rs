//! Persistent window settings: font size and which corner the window sits in.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const DEFAULT_FONT_SIZE: f32 = 14.0;
pub const MIN_FONT_SIZE: f32 = 8.0;
pub const MAX_FONT_SIZE: f32 = 40.0;

const FILE_NAME: &str = "settings.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Corner {
    pub fn is_top(self) -> bool {
        matches!(self, Corner::TopLeft | Corner::TopRight)
    }

    pub fn is_left(self) -> bool {
        matches!(self, Corner::TopLeft | Corner::BottomLeft)
    }

    /// The corner reached by pressing an arrow key. Vertical keys flip the
    /// vertical half, horizontal keys flip the horizontal half.
    pub fn moved(self, direction: Direction) -> Corner {
        let (top, left) = match direction {
            Direction::Up => (true, self.is_left()),
            Direction::Down => (false, self.is_left()),
            Direction::Left => (self.is_top(), true),
            Direction::Right => (self.is_top(), false),
        };
        match (top, left) {
            (true, true) => Corner::TopLeft,
            (true, false) => Corner::TopRight,
            (false, true) => Corner::BottomLeft,
            (false, false) => Corner::BottomRight,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    pub font_size: f32,
    pub corner: Corner,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: DEFAULT_FONT_SIZE,
            corner: Corner::TopRight,
        }
    }
}

impl Settings {
    fn path(dir: &Path) -> PathBuf {
        dir.join(FILE_NAME)
    }

    /// Reads the settings file, falling back to defaults when it is missing or
    /// unreadable: bad settings should never stop the app from starting.
    pub fn load_or_default(dir: &Path) -> Self {
        match std::fs::read_to_string(Self::path(dir)) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_else(|error| {
                log::warn!("ignoring unreadable settings: {error}");
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, dir: &Path) -> Result<(), String> {
        let raw = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(Self::path(dir), raw).map_err(|e| e.to_string())
    }

    pub fn adjust_font_size(&mut self, delta: f32) {
        self.font_size = (self.font_size + delta).clamp(MIN_FONT_SIZE, MAX_FONT_SIZE);
    }

    pub fn reset_font_size(&mut self) {
        self.font_size = DEFAULT_FONT_SIZE;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_keys_flip_one_axis_at_a_time() {
        assert_eq!(Corner::TopRight.moved(Direction::Left), Corner::TopLeft);
        assert_eq!(Corner::TopRight.moved(Direction::Down), Corner::BottomRight);
        assert_eq!(Corner::BottomLeft.moved(Direction::Up), Corner::TopLeft);
        assert_eq!(
            Corner::BottomLeft.moved(Direction::Left),
            Corner::BottomLeft
        );
    }

    #[test]
    fn font_size_is_clamped() {
        let mut settings = Settings::default();
        settings.adjust_font_size(1000.0);
        assert_eq!(settings.font_size, MAX_FONT_SIZE);
        settings.adjust_font_size(-1000.0);
        assert_eq!(settings.font_size, MIN_FONT_SIZE);
        settings.reset_font_size();
        assert_eq!(settings.font_size, DEFAULT_FONT_SIZE);
    }

    #[test]
    fn settings_round_trip_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        let mut settings = Settings::default();
        settings.adjust_font_size(3.0);
        settings.corner = Corner::BottomLeft;
        settings.save(dir.path()).unwrap();
        assert_eq!(Settings::load_or_default(dir.path()), settings);
    }

    #[test]
    fn corrupt_settings_fall_back_to_defaults() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(FILE_NAME), "not json").unwrap();
        assert_eq!(Settings::load_or_default(dir.path()), Settings::default());
    }
}
