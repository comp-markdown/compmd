//! For the app's `build.rs`, with the `build` feature: embeds the bundle `@compmd/vite` wrote (docs/tauri-plugin.md,
//! decision 3), so the binary carries it: no resource to bundle, no file read at startup.

use std::ffi::OsString;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use compmd_host::{FORMAT, MANIFEST, Manifest, ManifestError};

use crate::build_error::BuildError;
use crate::origin::is_scheme;

/// The file in `OUT_DIR` that [`include_bundle!`](crate::include_bundle) includes.
const EMBEDDED: &str = "compmd_bundle.rs";

/// The bundle's folders the scheme serves, besides the manifest.
const SERVED: [&str; 2] = ["frames", "packs"];

/// Embeds the bundle in `folder`, relative to the app's `src-tauri/`, for [`include_bundle!`](crate::include_bundle),
/// and tells Cargo to build again when it changes. A folder with no bundle is embedded as none, with a warning: the
/// app then serves nothing, and its page says so.
///
/// # Panics
///
/// When the bundle can't be used: its manifest is of a format this plugin doesn't read, or names no scheme, or a file
/// can't be read; and outside a build script. Each says what to do.
///
/// # Example
///
/// ```no_run
/// // In src-tauri/build.rs's `main`, before `tauri_build::build()`:
/// tauri_plugin_compmd::build::bundle("compmd");
/// ```
pub fn bundle(folder: impl AsRef<Path>) {
    bundle_with(folder.as_ref(), |name| std::env::var_os(name));
}

/// [`bundle`], with Cargo's environment variables from `var`.
fn bundle_with(folder: &Path, var: impl Fn(&str) -> Option<OsString>) {
    let (Some(app), Some(out)) = (var("CARGO_MANIFEST_DIR"), var("OUT_DIR")) else {
        panic!(
            "tauri-plugin-compmd: build::bundle runs in the app's build script, where Cargo sets CARGO_MANIFEST_DIR and OUT_DIR"
        );
    };
    for line in embed(&Path::new(&app).join(folder), Path::new(&out)) {
        println!("{line}");
    }
}

/// Writes the code embedding the bundle in `folder` into `out`, and gives what to tell Cargo.
///
/// # Panics
///
/// As [`bundle`].
fn embed(folder: &Path, out: &Path) -> Vec<String> {
    let mut said = vec![format!("cargo:rerun-if-changed={}", folder.display())];
    let code = match embedding(folder) {
        Ok(code) => code,
        Err(error @ BuildError::NoBundle(_)) => {
            said.push(format!(
                "cargo:warning=tauri-plugin-compmd: {error}; the app serves none"
            ));
            NONE.to_owned()
        }
        Err(error) => panic!("tauri-plugin-compmd: {error}"),
    };
    if let Err(error) = fs::write(out.join(EMBEDDED), code) {
        panic!("tauri-plugin-compmd: the bundle can't be written into OUT_DIR: {error}");
    }
    said
}

/// The code of no bundle.
const NONE: &str = "::core::option::Option::None\n";

/// The code that embeds the bundle in `folder`: its manifest, then every file it serves, sorted by path.
///
/// # Errors
///
/// [`BuildError`] when there is no bundle, or it can't be used.
pub(crate) fn embedding(folder: &Path) -> Result<String, BuildError> {
    let manifest = folder.join(MANIFEST);
    let Ok(text) = fs::read_to_string(&manifest) else {
        return Err(BuildError::NoBundle(folder.to_path_buf()));
    };
    let parsed = Manifest::parse(&text).map_err(|error| match error {
        ManifestError::UnknownFormat { found, .. } => BuildError::Format {
            found,
            known: FORMAT,
        },
        other => BuildError::Manifest(other),
    })?;
    match parsed.scheme() {
        None => return Err(BuildError::NoScheme),
        Some(scheme) if !is_scheme(scheme) => return Err(BuildError::Scheme(scheme.to_owned())),
        Some(_) => {}
    }
    let mut files = vec![(MANIFEST.to_owned(), manifest)];
    for served in SERVED {
        collect(&folder.join(served), served, &mut files)?;
    }
    files.sort();
    let mut code = String::from("::core::option::Option::Some(&[\n");
    for (path, file) in files {
        let _ = writeln!(
            code,
            "    ({path:?}, include_bytes!({:?})),",
            file.display().to_string()
        );
    }
    code.push_str("])\n");
    Ok(code)
}

/// Every file under `dir`, whose path in the bundle is `at`, with its own path there; none when there is no `dir`.
fn collect(dir: &Path, at: &str, files: &mut Vec<(String, PathBuf)>) -> Result<(), BuildError> {
    if !dir.exists() {
        return Ok(());
    }
    let unreadable = |error: std::io::Error| BuildError::Unreadable {
        folder: dir.to_path_buf(),
        why: error.to_string(),
    };
    for entry in fs::read_dir(dir).map_err(unreadable)? {
        let entry = entry.map_err(unreadable)?;
        let (path, name) = (
            entry.path(),
            format!("{at}/{}", entry.file_name().to_string_lossy()),
        );
        if path.is_dir() {
            collect(&path, &name, files)?;
        } else {
            files.push((name, path));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
