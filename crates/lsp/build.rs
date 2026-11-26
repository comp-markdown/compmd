//! Embeds the preview's page and its bundle (docs/editor-support.md, decision 8), which `npm-packages/devtools/preview/`
//! builds into `preview/`: the page's files under `page/`, the bundle's manifest and frames under `compmd/`. Without
//! them the server is built with no preview, which it then says, and Cargo prints a warning.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::{env, fs};

/// Every file under `dir`, with its path there, `/`-separated, sorted.
fn files_under(dir: &Path) -> Vec<(String, PathBuf)> {
    fn collect(dir: &Path, at: &str, files: &mut Vec<(String, PathBuf)>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let (path, name) = (entry.path(), entry.file_name());
            let named = if at.is_empty() {
                name.to_string_lossy().into_owned()
            } else {
                format!("{at}/{}", name.to_string_lossy())
            };
            if path.is_dir() {
                collect(&path, &named, files);
            } else {
                files.push((named, path));
            }
        }
    }
    let mut files = Vec::new();
    collect(dir, "", &mut files);
    files.sort();
    files
}

/// `files` as a slice of paths and their bytes.
fn slice_of(files: &[(String, PathBuf)]) -> String {
    let mut code = String::from("&[\n");
    for (path, file) in files {
        let _ = writeln!(
            code,
            "    ({path:?}, include_bytes!({:?})),",
            file.display().to_string()
        );
    }
    code.push(']');
    code
}

fn main() {
    let crate_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo sets CARGO_MANIFEST_DIR"));
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR"));
    let preview = crate_dir.join("preview");
    println!("cargo:rerun-if-changed={}", preview.display());
    let (page, bundle) = (
        files_under(&preview.join("page")),
        files_under(&preview.join("compmd")),
    );
    let built = page.iter().any(|(path, _)| path == "index.html")
        && bundle.iter().any(|(path, _)| path == "manifest.json");
    let code = if built {
        format!(
            "::core::option::Option::Some(({}, {}))\n",
            slice_of(&page),
            slice_of(&bundle)
        )
    } else {
        println!(
            "cargo:warning=compmd-lsp: no preview in {}: build it with `pnpm --dir npm-packages/devtools/preview build`; this server has none",
            preview.display()
        );
        "::core::option::Option::None\n".to_owned()
    };
    fs::write(out.join("preview.rs"), code)
        .expect("the preview's files can be written into OUT_DIR");
}
