//! `project`'s functions as WebAssembly exports, over the host's file system: a JavaScript object with `text(path)`
//! and `isDir(path)`, which `@compmd/vite` makes from Node's `fs`. They run only as WebAssembly; `project` is tested
//! natively.

use std::path::Path;

use compmd_project::ProjectFiles;
use wasm_bindgen::prelude::*;

use crate::project;

#[wasm_bindgen]
extern "C" {
    /// A host's file system: `text(path)`, a file's text or `undefined`; `isDir(path)`, whether it is a folder.
    #[wasm_bindgen(
        typescript_type = "{ text(path: string): string | undefined; isDir(path: string): boolean }"
    )]
    pub type HostFiles;

    #[wasm_bindgen(method, structural)]
    fn text(this: &HostFiles, path: &str) -> Option<String>;

    #[wasm_bindgen(method, structural, js_name = isDir)]
    fn is_dir(this: &HostFiles, path: &str) -> bool;
}

impl ProjectFiles for HostFiles {
    fn text(&self, path: &Path) -> Option<String> {
        Self::text(self, &path.to_string_lossy())
    }

    fn is_dir(&self, path: &Path) -> bool {
        Self::is_dir(self, &path.to_string_lossy())
    }
}

/// The configuration `text` holds, read as coming from `source`: `{config}`, or `{problems, message}`.
#[wasm_bindgen(js_name = readConfig)]
#[must_use]
pub fn read_config(text: &str, source: &str) -> String {
    project::read_config(text, source)
}

/// The configuration of `folder`: `{config}`, `{problems, message}`, or `{error}`.
#[wasm_bindgen(js_name = configOf)]
#[must_use]
pub fn config_of(folder: &str, files: &HostFiles) -> String {
    project::config_of(folder, files)
}

/// Whether `folder` holds a configuration: `compmd.config.json`, or `package.json`'s `compmd` object.
#[wasm_bindgen(js_name = holdsConfig)]
#[must_use]
pub fn holds_config(folder: &str, files: &HostFiles) -> bool {
    compmd_project::holds_config(Path::new(folder), files)
}

/// The installed packs of the project at `root`: `{packs}` or `{error}`.
#[wasm_bindgen(js_name = installedPacks)]
#[must_use]
pub fn installed_packs(root: &str, files: &HostFiles) -> String {
    project::installed_packs(root, files)
}

/// The folder packs among a configuration's pack `keys`, relative to its `folder`: `{packs}` or `{error}`.
#[wasm_bindgen(js_name = folderPacks)]
#[must_use]
#[allow(
    clippy::needless_pass_by_value,
    reason = "wasm-bindgen takes a JavaScript array as an owned Vec"
)]
pub fn folder_packs(keys: Vec<String>, folder: &str, files: &HostFiles) -> String {
    project::folder_packs(&keys, folder, files)
}

/// The integrity of each release the lock file nearest above `root` holds, by `name@version`.
#[wasm_bindgen]
#[must_use]
pub fn integrities(root: &str, files: &HostFiles) -> String {
    project::integrities(root, files)
}

/// The lock file nearest above `root`, if any.
#[wasm_bindgen(js_name = lockFileOf)]
#[must_use]
pub fn lock_file_of(root: &str, files: &HostFiles) -> Option<String> {
    project::lock_file_of(root, files)
}

/// The root folder of the document at `document`, among the editor's `workspace_folders`.
#[wasm_bindgen(js_name = rootOf)]
#[must_use]
#[allow(
    clippy::needless_pass_by_value,
    reason = "wasm-bindgen takes a JavaScript array as an owned Vec"
)]
pub fn root_of(document: &str, workspace_folders: Vec<String>, files: &HostFiles) -> String {
    project::root_of(document, &workspace_folders, files)
}
