use std::path::Path;

/// The MCP bridge plugin only exists when the `mcp-bridge` feature is on, and
/// Tauri rejects capabilities that reference permissions of absent plugins, so
/// its capability file is placed (and removed) to match the feature.
fn sync_mcp_bridge_capability() {
    let target = Path::new("capabilities/mcp-bridge.json");
    if std::env::var_os("CARGO_FEATURE_MCP_BRIDGE").is_some() {
        std::fs::copy("capabilities-optional/mcp-bridge.json", target)
            .expect("failed to enable the mcp-bridge capability");
    } else if target.exists() {
        std::fs::remove_file(target).expect("failed to disable the mcp-bridge capability");
    }
}

fn main() {
    println!("cargo:rerun-if-changed=capabilities-optional/mcp-bridge.json");
    sync_mcp_bridge_capability();
    tauri_build::build()
}
