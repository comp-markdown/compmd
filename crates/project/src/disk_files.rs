//! A project's files on the disk.

use std::fs;
use std::path::Path;

use crate::project_files::ProjectFiles;

/// The files on the disk, read with `std::fs`, following links (a pnpm `node_modules` is links).
#[derive(Clone, Copy, Debug, Default)]
pub struct DiskFiles;

impl ProjectFiles for DiskFiles {
    fn text(&self, path: &Path) -> Option<String> {
        fs::read_to_string(path).ok()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn reads_the_disks_files_and_knows_its_folders() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        assert!(
            DiskFiles
                .text(&manifest)
                .is_some_and(|text| text.contains("compmd-project"))
        );
        assert!(DiskFiles.is_dir(Path::new(env!("CARGO_MANIFEST_DIR"))));
        assert!(!DiskFiles.is_dir(&manifest));
        assert_eq!(DiskFiles.text(Path::new(env!("CARGO_MANIFEST_DIR"))), None);
    }
}
