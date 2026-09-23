//! Platform-specific integrations.
//!
//! Getting the focused window, moving the cheatsheet window and pinning it
//! above other applications are each isolated behind a trait so that adding a
//! platform means adding a module here, not editing the rest of the app.

pub mod focus;
pub mod window;

#[cfg(target_os = "linux")]
pub mod linux_x11;
