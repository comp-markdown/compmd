//! Where the files the processor names are on the disk: a path in the document's root, or a pack's place
//! (`@kit/notes@1.0.0/index.comp.md`) in the folder the pack was found in.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use compmd_project::PackFolder;

/// A document's root, and the folder of each pack found for it, by its release (`@kit/notes@1.0.0`).
#[derive(Debug, Default)]
pub(crate) struct Places {
    root: PathBuf,
    packs: BTreeMap<String, PathBuf>,
}

impl Places {
    /// The places of a document whose root is `root`, with the packs `found`.
    pub(crate) fn new<'a>(root: &Path, found: impl IntoIterator<Item = &'a PackFolder>) -> Self {
        let packs = found
            .into_iter()
            .map(|pack| (pack.release.to_string(), pack.dir.clone()))
            .collect();
        Self {
            root: root.to_owned(),
            packs,
        }
    }

    /// The path of `file`, as diagnostics name it; `None` for a pack that wasn't found on the disk (a registry's, or
    /// `@compmd/core`).
    pub(crate) fn path_of(&self, file: &str) -> Option<PathBuf> {
        if !file.starts_with('@') {
            return Some(self.root.join(file));
        }
        let (scope, rest) = file.split_once('/')?;
        let (pack, path) = rest.split_once('/')?;
        Some(self.packs.get(&format!("{scope}/{pack}"))?.join(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_is_in_the_root_and_a_place_in_its_packs_folder() {
        let pack = PackFolder {
            release: "@kit/notes@1.0.0".parse().expect("a release"),
            dir: PathBuf::from("/site/node_modules/@kit/notes"),
            key: None,
            integrity: None,
        };
        let places = Places::new(Path::new("/site"), [&pack]);
        assert_eq!(
            places.path_of("posts/a.comp.md"),
            Some(PathBuf::from("/site/posts/a.comp.md"))
        );
        assert_eq!(
            places.path_of("@kit/notes@1.0.0/index.comp.md"),
            Some(PathBuf::from("/site/node_modules/@kit/notes/index.comp.md"))
        );
        assert_eq!(places.path_of("@compmd/core@1.0.0/index.comp.md"), None);
        assert_eq!(places.path_of("@kit"), None);
    }
}
