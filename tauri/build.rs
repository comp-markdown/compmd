//! The plugin's commands, each with an allow permission an app's capability grants, and the frames the crate
//! embeds (docs/tauri-plugin.md, decision 2).

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// The commands: `base` alone, the frames' `base` (docs/tauri-plugin.md, decision 5).
const COMMANDS: &[&str] = &["base"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
    let manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets it"));
    let frames = manifest.join("frames");
    println!("cargo:rerun-if-changed={}", frames.display());
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets it")).join("embedded.rs");
    fs::write(out, embedded(&frames)).expect("the embedded frames are written");
}

/// The Rust that embeds the build `frames/frames.json` names: `HASH` and `FILES`, each file's path in the build and
/// its bytes, in order. Without a build, none, with a warning: the app's page then reports that the frames didn't load.
fn embedded(frames: &Path) -> String {
    let hash = fs::read_to_string(frames.join("frames.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|named| named["hash"].as_str().map(str::to_owned))
        .filter(|hash| {
            hash.len() == 16
                && hash
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        });
    let Some(hash) = hash else {
        println!(
            "cargo:warning=tauri-plugin-compmd: no frames in {}: run `cargo make tauri-frames`; the app serves none",
            frames.display()
        );
        return "/// The embedded build's hash.\npub const HASH: Option<&str> = None;\n/// The embedded build's files.\npub static FILES: &[(&str, &[u8])] = &[];\n".to_owned();
    };
    let build = frames.join(&hash);
    let mut files = Vec::new();
    collect(&build, &build, &mut files);
    files.sort();
    let mut code = format!(
        "/// The embedded build's hash.\npub const HASH: Option<&str> = Some({hash:?});\n/// The embedded build's files.\npub static FILES: &[(&str, &[u8])] = &[\n"
    );
    for (name, path) in files {
        writeln!(
            code,
            "    ({name:?}, include_bytes!({:?})),",
            path.display().to_string()
        )
        .expect("a string takes it");
    }
    code.push_str("];\n");
    code
}

/// Every file under `folder`, with its path from `root`, `/` between its parts.
fn collect(root: &Path, folder: &Path, files: &mut Vec<(String, PathBuf)>) {
    for entry in fs::read_dir(folder)
        .expect("the build can be read")
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            collect(root, &path, files);
        } else {
            let name = path
                .strip_prefix(root)
                .expect("under the build")
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            files.push((name, path));
        }
    }
}
