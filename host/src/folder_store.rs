//! A store in a folder: a desktop app's data folder, a server's disk.
//!
//! Each release is a folder `<scope>/<name>@<version>/`, with `release.json` (its integrity) and its files under
//! `files/`.

use std::fs;
use std::io;
use std::path::PathBuf;

use compmd_pack::{FilePath, Integrity, PackName, Place, ReleaseId};
use serde::Deserialize;

use crate::store::{Store, StoredRelease};

/// A folder of kept releases.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderStore {
    root: PathBuf,
}

/// What `release.json` holds.
#[derive(Deserialize)]
struct Kept {
    integrity: Integrity,
}

impl FolderStore {
    /// The store in `root`, which needn't exist yet.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Keeps `release`, its `files` already checked against `integrity`, replacing what was kept for it.
    ///
    /// # Errors
    ///
    /// An I/O error.
    pub fn keep(
        &self,
        release: &ReleaseId,
        integrity: &Integrity,
        files: impl IntoIterator<Item = (FilePath, Vec<u8>)>,
    ) -> io::Result<()> {
        let folder = self.folder(release);
        if folder.exists() {
            fs::remove_dir_all(&folder)?;
        }
        for (path, bytes) in files {
            let file = joined(folder.join("files"), &path);
            if let Some(parent) = file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(file, bytes)?;
        }
        fs::create_dir_all(&folder)?;
        let kept = serde_json::json!({ "integrity": integrity }).to_string();
        fs::write(folder.join("release.json"), kept)
    }

    /// The folder of `release`: its scope's, then `<name>@<version>`.
    fn folder(&self, release: &ReleaseId) -> PathBuf {
        self.root
            .join(format!("@{}", release.pack.scope()))
            .join(format!("{}@{}", release.pack.name(), release.version))
    }
}

impl Store for FolderStore {
    fn releases(&self, pack: &PackName) -> Vec<StoredRelease> {
        let Ok(entries) = fs::read_dir(self.root.join(format!("@{}", pack.scope()))) else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter_map(|entry| {
                let folder = entry.file_name().into_string().ok()?;
                let version = folder
                    .strip_prefix(pack.name())?
                    .strip_prefix('@')?
                    .parse()
                    .ok()?;
                let kept: Kept =
                    serde_json::from_slice(&fs::read(entry.path().join("release.json")).ok()?)
                        .ok()?;
                Some(StoredRelease {
                    release: ReleaseId::new(pack.clone(), version),
                    integrity: kept.integrity,
                })
            })
            .collect()
    }

    fn file(&self, place: &Place) -> Option<Vec<u8>> {
        let file = joined(self.folder(&place.release).join("files"), &place.path);
        file.is_file().then(|| fs::read(file).ok()).flatten()
    }
}

/// `path` under `folder`: a `FilePath` can't leave it.
fn joined(folder: PathBuf, path: &FilePath) -> PathBuf {
    path.parts().fold(folder, |file, part| file.join(part))
}

#[cfg(test)]
mod tests;
