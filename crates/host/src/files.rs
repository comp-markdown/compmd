//! Where a bundle's files are: embedded in a binary, in a folder, or in memory (a test's).

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

/// A bundle's files, each by its path in the bundle, its parts joined by `/`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Files {
    /// Embedded in a binary (`include_bytes!`), sorted by path.
    Embedded(&'static [(&'static str, &'static [u8])]),
    /// In a folder.
    Folder(PathBuf),
    /// In memory, as a test makes them.
    Owned(BTreeMap<String, Vec<u8>>),
}

impl Files {
    /// The bytes of the file at `path`, when there is one; a folder is not a file.
    #[must_use]
    pub fn read(&self, path: &str) -> Option<Vec<u8>> {
        match self {
            Self::Embedded(files) => files
                .binary_search_by(|(name, _)| (*name).cmp(path))
                .ok()
                .map(|found| files[found].1.to_vec()),
            Self::Folder(folder) => {
                let file = path
                    .split('/')
                    .fold(folder.clone(), |file, part| file.join(part));
                file.is_file().then(|| fs::read(file).ok()).flatten()
            }
            Self::Owned(files) => files.get(path).cloned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMBEDDED: &[(&str, &[u8])] = &[
        ("frames/a/chunks/runtime.js", b"export {}"),
        ("frames/a/strict.html", b"<!doctype html>"),
    ];

    #[test]
    fn an_embedded_file_is_found_by_its_path() {
        let files = Files::Embedded(EMBEDDED);
        assert_eq!(
            files.read("frames/a/strict.html"),
            Some(b"<!doctype html>".to_vec())
        );
        assert_eq!(
            files.read("frames/a/chunks/runtime.js"),
            Some(b"export {}".to_vec())
        );
        assert_eq!(files.read("frames/a/chunks"), None);
    }

    #[test]
    fn a_folders_file_is_read_and_a_folder_is_not_a_file() {
        let folder = std::env::temp_dir().join(format!("compmd-host-files-{}", std::process::id()));
        fs::create_dir_all(folder.join("chunks")).expect("made");
        fs::write(folder.join("chunks").join("runtime.js"), "export {}").expect("written");
        let files = Files::Folder(folder.clone());
        assert_eq!(files.read("chunks/runtime.js"), Some(b"export {}".to_vec()));
        assert_eq!(files.read("chunks"), None);
        assert_eq!(files.read("missing.js"), None);
        drop(fs::remove_dir_all(folder));
    }

    #[test]
    fn owned_files_are_found_by_their_path() {
        let files = Files::Owned(BTreeMap::from([("a.js".to_owned(), b"1".to_vec())]));
        assert_eq!(files.read("a.js"), Some(b"1".to_vec()));
        assert_eq!(files.read("b.js"), None);
    }
}
