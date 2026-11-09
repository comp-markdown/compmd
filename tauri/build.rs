//! The plugin's commands, each with an allow permission an app's capability grants.

/// The commands: `base` alone, the frames' `base` (docs/tauri-plugin.md, decision 5).
const COMMANDS: &[&str] = &["base"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
