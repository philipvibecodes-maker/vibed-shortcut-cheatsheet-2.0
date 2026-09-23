# Shortcut Cheatsheet 2.0 — Technical Design

Status: draft for review. Scope of this pass: the cheatsheet window on Ubuntu/X11.
Settings window is out of scope but the architecture leaves room for it (a
second Tauri window backed by the same Rust core). Windows 11 is a future phase;
the module boundaries below are chosen so it can be added without touching UI code.

## 1. Stack

| Concern | Choice | Why |
|---|---|---|
| Shell | Tauri v2 (Rust backend + WebView frontend) | Requested. Frontend flexibility for future settings UI; Tauri already abstracts window creation, always-on-top, decorations, transparency and single-instance across Linux/Windows. |
| Frontend | Svelte 5 + Vite + TypeScript | Small runtime, reactive table, and ready for the future settings window without a migration. |
| Integration testing | `@hypothesi/tauri-mcp-server` (Tauri MCP server) + its `tauri-mcp` CLI | Drives the *real* built app: screenshots, DOM queries, key/mouse input, IPC call capture, console logs. Requires the MCP bridge plugin in the app (dev/test builds only, behind a cargo feature). CLI used in CI; MCP server used interactively by the agent. |
| Rust ↔ JS | Tauri commands (`#[tauri::command]`) + events (`app.emit`) | Backend pushes a `cheatsheet://update` event; frontend calls commands for font size / corner changes (see §4). |
| X11 access | `x11rb` (pure Rust XCB bindings) | Query `_NET_ACTIVE_WINDOW` + `WM_CLASS`, subscribe to `PropertyNotify` on the root window. |
| Single instance | `tauri-plugin-single-instance` | Official plugin. Second launch invokes a callback in the *first* process with the new argv → we use it to quit (toggle). |
| Store | `tauri-plugin-store` (or plain `serde_json` files) | Persist `settings.json`; `shortcuts.json` handled by our own module for a stable schema. |
| Paths | `tauri::path` (`app_config_dir`) | `~/.config/shortcut-cheatsheet/` on Linux, `%APPDATA%` on Windows. |
| Logging | `tauri-plugin-log` | `RUST_LOG`-style filtering, file + stdout. |

Ubuntu build deps: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libxcb*` (already pulled in by GTK). Node ≥ 20 for the frontend toolchain.

## 2. Module layout

```
src-tauri/
  src/
    main.rs / lib.rs          # tauri::Builder, plugins, command registration, thread spawn
    commands.rs               # #[tauri::command]s: get_state, set_font_size, move_corner, quit
    state.rs                  # AppState (Mutex): ShortcutDb, Settings, last_shown_app
    config/
      mod.rs                  # Settings { font_size, corner } load/save
      shortcuts.rs            # ShortcutDb: BTreeMap<AppId, AppEntry> load/save
      defaults/chrome.json    # include_str!, written to config dir on first run
    platform/
      mod.rs                  # traits + `pub fn current() -> Box<dyn Platform>`
      focus.rs                # trait FocusTracker { fn run(self, tx: Sender<AppId>) }
      window.rs               # trait WindowController { fn move_to(&self, win, corner, size); fn set_always_on_top(&self, win, bool); fn work_area(&self) -> Rect }
      linux_x11/
        focus.rs              # x11rb: _NET_ACTIVE_WINDOW -> WM_CLASS
        window.rs             # x11rb: _NET_WORKAREA; delegates set_position/always_on_top to tauri::Window
      windows/                # (future) GetForegroundWindow + exe name; SystemParametersInfo(SPI_GETWORKAREA)
  tauri.conf.json             # window: decorations=false, transparent=true, alwaysOnTop=true, skipTaskbar=true, resizable=false
  capabilities/default.json   # allow core:window:* for the main window only
src/                          # frontend (Svelte)
  main.ts                     # mount App
  App.svelte                  # listens to `cheatsheet://update`, holds state, ResizeObserver → report_content_size
  lib/ShortcutTable.svelte    # 2-column table
  lib/keys.ts                 # keydown → invoke('move_corner' | 'set_font_size' | 'quit') (pure fn, unit-tested)
  lib/ipc.ts                  # typed wrappers around invoke/listen
  app.css                     # dark theme, rounded frame, table layout
tests/
  integration/*.test.ts       # Vitest, drives the built app through the tauri-mcp CLI
```

The MCP bridge plugin (`tauri-plugin-mcp-bridge`) is registered only when the
`mcp-bridge` cargo feature is on (`cargo tauri dev --features mcp-bridge` and CI);
release builds never include it.

Per `style-guidelines.md`, focus tracking, window moving and always-on-top are
each behind their own trait so a Windows backend is a new directory, not edits
to existing code. `platform::current()` is selected with `#[cfg(target_os)]`.
Tauri's own `Window::set_position` / `set_always_on_top` are used where they
suffice; the trait exists so platform quirks (work-area query, WM hints) have a
home.

## 3. Data model

```rust
/// Identifies an application. On X11 this is the WM_CLASS *class* (second
/// string, e.g. "Google-chrome"). On Windows it will be the exe name.
pub struct AppId(String);

#[derive(Serialize, Deserialize)]
pub struct Shortcut { pub action: String, pub keys: String }

#[derive(Serialize, Deserialize)]
pub struct ShortcutDb { pub apps: BTreeMap<String, AppEntry> }   // key = AppId

#[derive(Serialize, Deserialize)]
pub struct AppEntry { pub display_name: String, pub shortcuts: Vec<Shortcut> }

#[derive(Serialize, Deserialize)]
pub struct Settings { pub font_size: f32 /* default 14 */, pub corner: Corner /* default TopRight */ }
pub enum Corner { TopLeft, TopRight, BottomLeft, BottomRight }
```

Files (Linux): `~/.config/shortcut-cheatsheet/shortcuts.json` and `settings.json`.
On first run, `shortcuts.json` is created from the bundled Chrome defaults
(~25 common shortcuts: tabs, navigation, page, find, dev tools). Lookup is
case-insensitive on `AppId` (`google-chrome` vs `Google-chrome`).

```json
{
  "apps": {
    "Google-chrome": {
      "display_name": "Google Chrome",
      "shortcuts": [
        { "action": "New tab",           "keys": "Ctrl+T" },
        { "action": "Close tab",         "keys": "Ctrl+W" },
        { "action": "Reopen closed tab", "keys": "Ctrl+Shift+T" },
        { "action": "Next tab",          "keys": "Ctrl+Tab" },
        { "action": "Focus address bar", "keys": "Ctrl+L" }
      ]
    }
  }
}
```

## 4. Rust ↔ frontend contract

Events (Rust → JS):
- `cheatsheet://update` `{ display_name, shortcuts: Shortcut[], font_size }` — emitted on focus change or font change.

Commands (JS → Rust):
- `get_state() -> { display_name, shortcuts, font_size, corner }` (on load)
- `set_font_size(delta: i32)` — clamps 8–40, persists, re-emits update.
- `move_corner(dir: "up"|"down"|"left"|"right")` — updates `Settings.corner`, persists, repositions.
- `report_content_size(width: u32, height: u32)` — frontend measures the rendered table (`ResizeObserver`) and reports; Rust calls `set_size` then re-snaps to the corner so the window stays flush.
- `quit()`

## 5. Behaviour mapping to specs

| Spec | Implementation |
|---|---|
| Starts top-right, flush | `Settings.corner` (persisted) → `WindowController::move_to` using `_NET_WORKAREA` (avoids GNOME top bar / docks). |
| Borderless, rounded corners | `decorations: false`, `transparent: true`; CSS `html, body { background: transparent }` and `#frame { border-radius: 12px; background: #1e1e1e }`. On X11 this needs a 32-bit ARGB visual + a compositor (GNOME/Mutter, KWin, picom all qualify); Tauri/WebKitGTK set the RGBA visual when `transparent: true`. If the WM reports no compositor (`_NET_WM_CM_S0` selection unowned) we fall back to opaque square corners so the window is still usable. Integration test screenshots the corners to assert transparency. |
| Opens on launch | Main window `visible: true` in `tauri.conf.json`; first `update` event fires after focus tracker's initial query. |
| Focused app's shortcuts; ignore self | Focus thread yields `AppId`. If it equals our own `WM_CLASS` (`shortcut-cheatsheet`), keep `last_shown_app`. `last_shown_app` only updates when the new app has shortcuts. |
| 2 columns, action left / keys right | `<table>` with `<td class="action">` and `<td class="keys">` (right-aligned, monospace, keys rendered as `<kbd>` chips). |
| Window fits table | `ResizeObserver` on `#frame` → `report_content_size` → `Window::set_size` → re-snap. |
| Arrow keys → corners (when focused) | `keydown` in webview → `move_corner`. ←/→ toggle left/right, ↑/↓ toggle top/bottom. |
| Ctrl + / Ctrl - | `keydown` with `ctrlKey` and `=`/`+`/`-`/`NumpadAdd`/`NumpadSubtract` → `set_font_size(±1)`. `Ctrl+0` resets. Browser zoom is disabled so only our handler applies. |
| Persistent font size | `Settings.font_size` saved on every change. |
| Dark theme | CSS variables; `prefers-color-scheme` ignored, always dark. |
| Always on top | `alwaysOnTop: true` + `skipTaskbar: true`; `WindowController::set_always_on_top` re-asserts `_NET_WM_STATE_ABOVE` after show (some WMs drop it). |
| Empty state → hidden | No shortcuts for focused/last app → `window.hide()`; `show()` again when a matching app is focused. |
| Single instance toggle | `tauri_plugin_single_instance::init(|app, _argv, _cwd| app.exit(0))`. Second launch exits immediately after the plugin notifies the first (plugin does this before `setup`). |
| Quit key | `Esc` or `Q` while focused → `quit()`. |
| Context menu / text selection | Disabled in CSS/JS so the webview behaves like a native panel. |

## 6. Focus tracking (X11 detail)

- Dedicated thread opened in `setup()`; selects `PROPERTY_CHANGE` on the root
  window and reacts to `_NET_ACTIVE_WINDOW` changes (event-driven). Fallback:
  poll every 250 ms if the WM lacks EWMH support.
- Active window → `WM_CLASS` (class part). Missing `WM_CLASS` → look up the
  top-level client via `_NET_CLIENT_LIST`.
- Sends `AppId` via `std::sync::mpsc`; a receiver task on the Tauri side
  updates `AppState`, hides/shows the window and emits `cheatsheet://update`.

## 7. Threads

```
main thread   ── Tauri/GTK event loop (window ops must happen here; use `app.run_on_main_thread`)
focus thread  ── x11rb event loop → mpsc → main
(single-instance plugin uses its own IPC — no custom thread needed)
```

## 8. Testing

- Rust unit tests: `ShortcutDb` load/save round-trip, case-insensitive lookup,
  `Settings` defaults + clamp, corner transitions, corner → position math given
  a work area and window size.
- Frontend unit tests (Vitest): `lib/keys.ts` mapping of `KeyboardEvent` → command.
- Integration via Tauri MCP server (`Xvfb` + `openbox` + `picom` in CI so the
  compositor path is exercised): build with `--features mcp-bridge`, launch,
  then through the `tauri-mcp` CLI / MCP tools:
  - `webview-screenshot` → window is dark, rounded, positioned top-right of the work area.
  - DOM query → table has 2 columns and N rows matching `chrome.json`.
  - send `ArrowLeft`/`ArrowDown` → `get_window_info` position equals bottom-left, flush.
  - send `Ctrl+=` ×3, restart app → font size persisted (read `settings.json` + DOM computed style).
  - simulate focus change (launch `xterm`, `xdotool windowactivate`) → window hidden; activate a
    window with `WM_CLASS=Google-chrome` (xterm `-class Google-chrome`) → shown with Chrome table.
  - IPC monitor → `report_content_size` fires after table render and window size matches.
  - launch a second instance → first process exits (toggle).
- Manual checklist on Ubuntu X11: snapping under GNOME top bar, font
  persistence, hide/show switching Chrome ↔ terminal, always-on-top, rounded
  corners with the compositor.

## 9. Milestones (one PR each)

1. Skeleton: `create-tauri-app` (svelte-ts), dark borderless transparent rounded window, config load/save with Chrome defaults, single-instance toggle, `quit`, MCP bridge plugin behind feature flag.
2. X11 focus tracker + `WM_CLASS` matching, self-ignore, hide on empty, `update` event, table rendering.
3. Fit-to-content resizing, corner snapping with `_NET_WORKAREA`, arrow keys, font-size keys + persistence.
4. Tests, CI (`cargo fmt`, `clippy`, `cargo test`, `vitest` unit + MCP-driven integration under Xvfb), README with Ubuntu build deps.

## 10. Decisions

1. Rounded corners are required; implemented via transparent window + CSS radius, opaque fallback only when no compositor is present.
2. Focusability: click-to-focus (default). Global hotkey deferred.
3. Autostart on login: out of scope.
4. Binary / `WM_CLASS` / bundle identifier: `shortcut-cheatsheet` / `com.philip.shortcut-cheatsheet`.
5. Frontend: Svelte 5.
6. Integration tests: Tauri MCP server (`hypothesi/mcp-server-tauri`) — assumed to be the intended server; confirm if a different one was meant.
