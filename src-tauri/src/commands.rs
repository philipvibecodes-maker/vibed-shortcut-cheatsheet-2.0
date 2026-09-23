//! Tauri commands invoked by the frontend.

use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, State};

use crate::config::settings::Direction;
use crate::state::{AppState, CheatsheetState};

pub const UPDATE_EVENT: &str = "cheatsheet://update";

type SharedState<'a> = State<'a, Mutex<AppState>>;

fn emit_update(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<Mutex<AppState>>();
    let payload = state.lock().map_err(|e| e.to_string())?.cheatsheet();
    app.emit(UPDATE_EVENT, payload).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_state(state: SharedState<'_>) -> Result<CheatsheetState, String> {
    Ok(state.lock().map_err(|e| e.to_string())?.cheatsheet())
}

#[tauri::command]
pub fn set_font_size(app: AppHandle, state: SharedState<'_>, delta: f32) -> Result<(), String> {
    {
        let mut state = state.lock().map_err(|e| e.to_string())?;
        state.settings.adjust_font_size(delta);
        state.settings.save(&state.config_dir)?;
    }
    emit_update(&app)
}

#[tauri::command]
pub fn reset_font_size(app: AppHandle, state: SharedState<'_>) -> Result<(), String> {
    {
        let mut state = state.lock().map_err(|e| e.to_string())?;
        state.settings.reset_font_size();
        state.settings.save(&state.config_dir)?;
    }
    emit_update(&app)
}

#[tauri::command]
pub fn move_corner(state: SharedState<'_>, direction: Direction) -> Result<(), String> {
    let mut state = state.lock().map_err(|e| e.to_string())?;
    state.settings.corner = state.settings.corner.moved(direction);
    state.settings.save(&state.config_dir)
    // Repositioning is wired up in the window-placement milestone.
}

/// Reported by the frontend whenever the rendered table changes size, so the
/// window can shrink-wrap the table.
#[tauri::command]
pub fn report_content_size(window: tauri::Window, width: u32, height: u32) -> Result<(), String> {
    window
        .set_size(tauri::LogicalSize::new(width, height))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn quit(app: AppHandle) {
    app.exit(0);
}
