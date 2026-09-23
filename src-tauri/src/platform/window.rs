//! Placing and pinning the cheatsheet window.

use crate::config::settings::Corner;

/// A screen area in physical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Window placement that the windowing system, rather than Tauri, has to answer
/// for: where the usable screen area is, and how "always on top" is enforced.
pub trait WindowController: Send + Sync {
    /// The screen area excluding panels and docks, so a window placed in a
    /// corner sits flush against the usable edge.
    fn work_area(&self) -> Rect;

    /// Re-asserts the always-on-top hint. Some window managers drop it when a
    /// window is hidden and shown again.
    fn set_always_on_top(&self, window: &tauri::Window, on_top: bool);
}

/// The top-left position that puts a window of `size` flush into `corner` of
/// `area`.
pub fn corner_position(area: Rect, corner: Corner, width: u32, height: u32) -> (i32, i32) {
    let x = if corner.is_left() {
        area.x
    } else {
        area.x + area.width as i32 - width as i32
    };
    let y = if corner.is_top() {
        area.y
    } else {
        area.y + area.height as i32 - height as i32
    };
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Rect = Rect {
        x: 0,
        y: 27,
        width: 1920,
        height: 1053,
    };

    #[test]
    fn corners_sit_flush_against_the_work_area() {
        assert_eq!(corner_position(AREA, Corner::TopLeft, 300, 200), (0, 27));
        assert_eq!(
            corner_position(AREA, Corner::TopRight, 300, 200),
            (1620, 27)
        );
        assert_eq!(
            corner_position(AREA, Corner::BottomLeft, 300, 200),
            (0, 880)
        );
        assert_eq!(
            corner_position(AREA, Corner::BottomRight, 300, 200),
            (1620, 880)
        );
    }
}
