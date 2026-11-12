//! Tauri's build step: the bundle the page's build wrote into `compmd/`, embedded (docs/tauri-plugin.md, decision 3);
//! the app's context from `tauri.conf.json`; and its commands, each with an allow permission its capability grants.
//! An app must declare its commands: without them Tauri lets any page call every one, a page on the frames' scheme
//! too (docs/tauri-plugin.md, decision 8).

fn main() {
    tauri_plugin_compmd::build::bundle("compmd");
    let manifest = tauri_build::AppManifest::new().commands(&["mode", "document", "report"]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("the Tauri build runs");
}
