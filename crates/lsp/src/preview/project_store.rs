//! The project's packs as the preview's frames load them (docs/editor-support.md, decision 8): its folder and installed
//! packs, read from where they were found, as bytes, so images and fonts load too.

use std::fs;

use compmd_host::pack::{PackName, Place};
use compmd_host::{Store, StoredRelease};
use compmd_project::PackFolder;

/// The packs found in a project.
pub(crate) struct ProjectStore {
    packs: Vec<PackFolder>,
}

impl ProjectStore {
    /// The store of `packs`, those found in the project.
    pub(crate) fn new(packs: Vec<PackFolder>) -> Self {
        Self { packs }
    }
}

impl Store for ProjectStore {
    /// The releases of `pack` found with the integrity their lock file gives; a folder pack has none.
    fn releases(&self, pack: &PackName) -> Vec<StoredRelease> {
        self.packs
            .iter()
            .filter(|found| found.release.pack == *pack)
            .filter_map(|found| {
                Some(StoredRelease {
                    release: found.release.clone(),
                    integrity: found.integrity.clone()?,
                })
            })
            .collect()
    }

    /// The file at `place`, in the folder its release was found in.
    fn file(&self, place: &Place) -> Option<Vec<u8>> {
        let found = self
            .packs
            .iter()
            .find(|found| found.release == place.release)?;
        let path = place
            .path
            .parts()
            .fold(found.dir.clone(), |path, part| path.join(part));
        fs::read(path).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_found_packs_files_are_read_from_its_folder() {
        let dir =
            std::env::temp_dir().join(format!("compmd-lsp-project-store-{}", std::process::id()));
        fs::create_dir_all(dir.join("dist")).expect("a folder");
        fs::write(dir.join("dist/notes.js"), b"export {}").expect("written");
        let found = |integrity: Option<&str>, version: &str| PackFolder {
            release: format!("@kit/notes@{version}").parse().expect("a release"),
            dir: dir.clone(),
            key: None,
            integrity: integrity.map(|integrity| integrity.parse().expect("an integrity")),
        };
        let store = ProjectStore::new(vec![
            found(Some("sha512-abc"), "1.0.0"),
            found(None, "2.0.0"),
        ]);
        let notes: PackName = "@kit/notes".parse().expect("a name");
        let releases: Vec<String> = store
            .releases(&notes)
            .iter()
            .map(|stored| stored.release.to_string())
            .collect();
        assert_eq!(releases, ["@kit/notes@1.0.0"], "only with an integrity");
        let place = |text: &str| text.parse::<Place>().expect("a place");
        assert_eq!(
            store
                .file(&place("@kit/notes@2.0.0/dist/notes.js"))
                .as_deref(),
            Some(&b"export {}"[..])
        );
        assert_eq!(store.file(&place("@kit/notes@2.0.0/dist/none.js")), None);
        assert_eq!(store.file(&place("@kit/other@1.0.0/dist/notes.js")), None);
    }
}
