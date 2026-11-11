//! A store in a folder: a desktop app's data folder, a server's disk.
//!
//! Each release is a folder `<name>@<version>/`, with `release.json` (its integrity) and its files under `files/`.

use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use semver::Version;
use serde::Deserialize;

use crate::place::{is_pack_name, release_folder};
use crate::store::{Store, StoredRelease};

/// A folder of kept releases.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderStore {
    root: PathBuf,
}

/// What `release.json` holds.
#[derive(Deserialize)]
struct Kept {
    integrity: String,
}

impl FolderStore {
    /// The store in `root`, which needn't exist yet.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Keeps `pack`'s release `version`, its files already checked against `integrity`, replacing what was kept for
    /// it.
    ///
    /// # Errors
    ///
    /// An I/O error, or `InvalidInput` for a pack's name that isn't one or a file's path that would leave the
    /// release's folder.
    pub fn keep<P: AsRef<str>>(
        &self,
        pack: &str,
        version: &Version,
        integrity: &str,
        files: impl IntoIterator<Item = (P, Vec<u8>)>,
    ) -> io::Result<()> {
        if !is_pack_name(pack) {
            return Err(invalid(format!("`{pack}` isn't a pack's name")));
        }
        let folder = self.folder(pack, version);
        if folder.exists() {
            fs::remove_dir_all(&folder)?;
        }
        for (path, bytes) in files {
            let path = path.as_ref();
            let file = inside(&folder.join("files"), path)
                .ok_or_else(|| invalid(format!("`{path}` would leave the release's folder")))?;
            if let Some(parent) = file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(file, bytes)?;
        }
        fs::create_dir_all(&folder)?;
        let kept = serde_json::json!({ "integrity": integrity }).to_string();
        fs::write(folder.join("release.json"), kept)
    }

    fn folder(&self, pack: &str, version: &Version) -> PathBuf {
        release_folder(pack, version)
            .split('/')
            .fold(self.root.clone(), |folder, part| folder.join(part))
    }
}

impl Store for FolderStore {
    fn releases(&self, pack: &str) -> Vec<StoredRelease> {
        if !is_pack_name(pack) {
            return Vec::new();
        }
        let (scope, name) = pack.split_once('/').unwrap_or_default();
        let Ok(entries) = fs::read_dir(self.root.join(scope)) else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter_map(|entry| {
                let folder = entry.file_name().into_string().ok()?;
                let version: Version =
                    folder.strip_prefix(name)?.strip_prefix('@')?.parse().ok()?;
                let kept: Kept =
                    serde_json::from_slice(&fs::read(entry.path().join("release.json")).ok()?)
                        .ok()?;
                Some(StoredRelease {
                    version,
                    integrity: kept.integrity,
                })
            })
            .collect()
    }

    fn file(&self, pack: &str, version: &Version, path: &str) -> Option<Vec<u8>> {
        if !is_pack_name(pack) {
            return None;
        }
        let file = inside(&self.folder(pack, version).join("files"), path)?;
        file.is_file().then(|| fs::read(file).ok()).flatten()
    }
}

/// `path` under `folder`, when each of its parts is a plain name, so it can't leave it.
fn inside(folder: &Path, path: &str) -> Option<PathBuf> {
    let plain = !path.is_empty()
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
        && Path::new(path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
    plain.then(|| {
        path.split('/')
            .fold(folder.to_path_buf(), |file, part| file.join(part))
    })
}

fn invalid(why: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, why)
}

#[cfg(test)]
mod tests;
