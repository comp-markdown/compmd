//! A document's root folder's files, as the processor reads them (`compmd::host::Files`): an open document's text as
//! the editor holds it, an unsaved change included, and the disk's otherwise ([the language server], decision 5).
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use compmd::host::Files;

/// The text of each document open in the editor, by its path, which every document's worker reads and the server
/// keeps up to date.
#[derive(Clone, Debug, Default)]
pub(crate) struct OpenTexts(Arc<Mutex<HashMap<PathBuf, String>>>);

impl OpenTexts {
    /// The editor's text of the file at `path`, if it is open.
    pub(crate) fn text(&self, path: &Path) -> Option<String> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(path)
            .cloned()
    }

    /// The editor's text of the file at `path` is now `text`.
    pub(crate) fn set(&self, path: PathBuf, text: String) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(path, text);
    }

    /// The file at `path` is closed: the disk's text counts again.
    pub(crate) fn close(&self, path: &Path) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(path);
    }
}

/// The files of the root folder `root`.
pub(crate) struct EditorFiles {
    root: PathBuf,
    open: OpenTexts,
}

impl EditorFiles {
    /// The files under `root`, `open` ones as the editor holds them.
    pub(crate) fn new(root: PathBuf, open: OpenTexts) -> Self {
        Self { root, open }
    }
}

impl Files for EditorFiles {
    fn file(&self, path: &str) -> Option<String> {
        let full = path
            .split('/')
            .fold(self.root.clone(), |full, part| full.join(part));
        self.open
            .text(&full)
            .or_else(|| std::fs::read_to_string(&full).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_open_files_text_comes_before_the_disks() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let open = OpenTexts::default();
        let files = EditorFiles::new(root.clone(), open.clone());
        assert!(
            files
                .file("Cargo.toml")
                .is_some_and(|text| text.contains("compmd-lsp")),
            "the disk's"
        );
        open.set(root.join("Cargo.toml"), "edited".into());
        assert_eq!(files.file("Cargo.toml").as_deref(), Some("edited"));
        open.close(&root.join("Cargo.toml"));
        assert!(
            files
                .file("Cargo.toml")
                .is_some_and(|text| text.contains("compmd-lsp"))
        );
        assert_eq!(files.file("none.comp.md"), None);
    }
}
