//! Where a build's files are: embedded in the crate (docs/tauri-plugin.md, decision 2), or in a folder of the app's own
//! build (decision 3).

use std::fs;
use std::path::PathBuf;

/// A build's files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Files {
    /// Embedded in the crate: each file's path in the build, with `/` between its parts, in order, and its bytes.
    Embedded(&'static [(&'static str, &'static [u8])]),
    /// In a folder of the app's own build.
    Folder(PathBuf),
}

impl Files {
    /// The bytes of the file at `parts` in the build, each part a plain name (`Frames::respond` checks them).
    #[must_use]
    pub fn read(&self, parts: &[&str]) -> Option<Vec<u8>> {
        match self {
            Self::Embedded(files) => {
                let path = parts.join("/");
                let found = files
                    .binary_search_by(|(name, _)| (*name).cmp(path.as_str()))
                    .ok()?;
                Some(files[found].1.to_vec())
            }
            Self::Folder(folder) => {
                let file = parts
                    .iter()
                    .fold(folder.clone(), |path, part| path.join(part));
                if file.is_file() {
                    fs::read(file).ok()
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMBEDDED: &[(&str, &[u8])] = &[
        ("chunks/runtime.js", b"export {}"),
        ("strict.html", b"<!doctype html>"),
    ];

    #[test]
    fn an_embedded_file_is_found_by_its_path() {
        let files = Files::Embedded(EMBEDDED);
        assert_eq!(
            files.read(&["strict.html"]),
            Some(b"<!doctype html>".to_vec())
        );
        assert_eq!(
            files.read(&["chunks", "runtime.js"]),
            Some(b"export {}".to_vec())
        );
        assert_eq!(files.read(&["chunks"]), None);
        assert_eq!(files.read(&["missing.js"]), None);
    }

    #[test]
    fn a_folders_file_is_read_and_a_folder_is_not_a_file() {
        let folder =
            std::env::temp_dir().join(format!("compmd-tauri-files-{}", std::process::id()));
        fs::create_dir_all(folder.join("chunks")).expect("made");
        fs::write(folder.join("chunks").join("runtime.js"), "export {}").expect("written");
        let files = Files::Folder(folder.clone());
        assert_eq!(
            files.read(&["chunks", "runtime.js"]),
            Some(b"export {}".to_vec())
        );
        assert_eq!(files.read(&["chunks"]), None);
        assert_eq!(files.read(&["missing.js"]), None);
        drop(fs::remove_dir_all(folder));
    }
}
