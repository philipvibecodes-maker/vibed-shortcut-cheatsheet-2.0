//! Shortcut Cheatsheet: a borderless always-on-top window listing the keyboard
//! shortcuts of the currently focused application.

pub mod commands;
pub mod config;
pub mod platform;
pub mod state;

use std::sync::Mutex;

use tauri::Manager;

use crate::config::{settings::Settings, shortcuts::ShortcutDb};
use crate::state::AppState;

/// A second launch of the application asks the running instance to quit, which
/// makes launching the app a toggle.
fn quit_running_instance(app: &tauri::AppHandle) {
    log::info!("second instance launched; quitting");
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    #[cfg_attr(not(feature = "mcp-bridge"), allow(unused_mut))]
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            quit_running_instance(app);
        }))
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::set_font_size,
            commands::reset_font_size,
            commands::move_corner,
            commands::report_content_size,
            commands::quit,
        ])
        .setup(|app| {
            if let Some(window) = app.get_webview_window("main") {
                // GTK otherwise keeps the toolkit's own floor, which stops the
                // window from shrink-wrapping small tables.
                window.set_min_size(Some(tauri::LogicalSize::new(1.0, 1.0)))?;
            }
            let config_dir = config::config_dir(app.handle())?;
            let settings = Settings::load_or_default(&config_dir);
            let shortcuts = ShortcutDb::load_or_seed(&config_dir)?;
            app.manage(Mutex::new(AppState::new(config_dir, settings, shortcuts)));
            Ok(())
        });

    #[cfg(feature = "mcp-bridge")]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
