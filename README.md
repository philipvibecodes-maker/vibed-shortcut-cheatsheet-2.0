# Shortcut Cheatsheet 2.0

A borderless, always-on-top window that lists the keyboard shortcuts of the
currently focused application. See [`overview.md`](overview.md) for the product
overview, [`specs/`](specs) for the specifications and
[`docs/technical-design.md`](docs/technical-design.md) for the architecture.

## Prerequisites (Ubuntu)

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev \
  libayatana-appindicator3-dev libxdo-dev patchelf build-essential
```

Plus [Rust](https://rustup.rs) and Node.js 20+.

## Development

```bash
npm install
npm run tauri dev          # run the app
npm run tauri dev -- --features mcp-bridge   # run with the automation bridge
```

Launching the app a second time quits the running instance, so the launcher
command acts as a toggle.

## Checks

```bash
npm run check                      # svelte-check
npm test                           # frontend unit tests
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

## Configuration

`~/.config/com.philip.shortcut-cheatsheet/`

- `shortcuts.json` — shortcuts per application, keyed by X11 `WM_CLASS`. Seeded
  with Google Chrome defaults on first run.
- `settings.json` — font size and the corner the window sits in.
