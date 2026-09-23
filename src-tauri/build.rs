use std::path::Path;

/// The MCP bridge plugin only exists when the `mcp-bridge` feature is on, and
/// Tauri rejects capabilities that reference permissions of absent plugins, so
/// its capability file is placed (and removed) to match the feature. The file
/// is only rewritten when its contents differ, otherwise `tauri dev` sees a
/// change on every rebuild and loops.
fn sync_mcp_bridge_capability() {
    let target = Path::new("capabilities/mcp-bridge.json");
    if std::env::var_os("CARGO_FEATURE_MCP_BRIDGE").is_some() {
        let source = std::fs::read("capabilities-optional/mcp-bridge.json")
            .expect("failed to read the optional mcp-bridge capability");
        if std::fs::read(target).ok().as_deref() != Some(source.as_slice()) {
            std::fs::write(target, &source).expect("failed to enable the mcp-bridge capability");
        }
    } else if target.exists() {
        std::fs::remove_file(target).expect("failed to disable the mcp-bridge capability");
    }
}

fn main() {
    println!("cargo:rerun-if-changed=capabilities-optional/mcp-bridge.json");
    sync_mcp_bridge_capability();
    tauri_build::build()
}
