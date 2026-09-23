//! Tracking which application currently has input focus.

use std::sync::mpsc::Sender;

/// Identifies an application: the X11 `WM_CLASS` class on Linux, and the
/// executable name on Windows.
pub type AppId = String;

/// Watches the desktop for focus changes.
pub trait FocusTracker: Send {
    /// Blocks, sending the focused application id on every change. Returns when
    /// the channel is closed or the connection to the desktop is lost.
    fn run(self: Box<Self>, updates: Sender<AppId>);
}
