//! A store in a folder: a desktop app's data folder, a server's disk.
//!
//! Each release is a folder `<scope>/<name>@<version>/`, with `release.json` (its integrity) and its files under
//! `files/`. A release is written whole into a hidden folder beside it, then renamed into place, so a keep that fails
//! halfway leaves the release that was there; and only a folder with `release.json` is a release.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use compmd_pack::{FilePath, Integrity, PackName, Place, ReleaseId};
use serde::Deserialize;

use crate::store::Store;
use crate::stored_release::StoredRelease;

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

    /// Keeps `release`, its `files` already checked against `integrity`, replacing what was kept for it. The release
    /// is written whole before it replaces the one kept: when writing fails, the one kept is still served.
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
        let _span = tracing::debug_span!("compmd_host::keep", %release).entered();
        let folder = self.folder(release);
        let staging = beside(&folder, "partial");
        let written = write_release(&staging, integrity, files);
        if let Err(error) = &written {
            tracing::warn!(%error, "not kept: writing it failed");
            drop(fs::remove_dir_all(&staging));
            return written;
        }
        let replaced = replace(&folder, &staging);
        if let Err(error) = &replaced {
            tracing::warn!(%error, "not kept: it couldn't take the kept release's place");
            drop(fs::remove_dir_all(&staging));
        }
        replaced
    }

    /// Every file kept for `release`, by its path in the package, when it is kept.
    #[must_use]
    pub fn files_of(&self, release: &ReleaseId) -> Option<BTreeMap<String, Vec<u8>>> {
        let folder = self.folder(release);
        if !folder.join("release.json").is_file() {
            return None;
        }
        let mut files = BTreeMap::new();
        read_files(&folder.join("files"), "", &mut files).ok()?;
        Some(files)
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
        let folder = self.folder(&place.release);
        if !folder.join("release.json").is_file() {
            return None;
        }
        let file = joined(folder.join("files"), &place.path);
        file.is_file().then(|| fs::read(file).ok()).flatten()
    }
}

/// Writes a release into the new folder `folder`: its `files` under `files/`, then `release.json` with `integrity`.
fn write_release(
    folder: &Path,
    integrity: &Integrity,
    files: impl IntoIterator<Item = (FilePath, Vec<u8>)>,
) -> io::Result<()> {
    fs::create_dir_all(folder)?;
    for (path, bytes) in files {
        let file = joined(folder.join("files"), &path);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file, bytes)?;
    }
    let kept = serde_json::json!({ "integrity": integrity }).to_string();
    fs::write(folder.join("release.json"), kept)
}

/// Adds the files under `folder` to `files`, each by its path after `prefix`.
fn read_files(
    folder: &Path,
    prefix: &str,
    files: &mut BTreeMap<String, Vec<u8>>,
) -> io::Result<()> {
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = format!("{prefix}{name}");
        if entry.file_type()?.is_dir() {
            read_files(&entry.path(), &format!("{path}/"), files)?;
        } else {
            files.insert(path, fs::read(entry.path())?);
        }
    }
    Ok(())
}

/// Puts `staging` in the place of `folder`: the release kept there, if any, is moved aside first, since a folder
/// can't be renamed over another, then removed; put back when `staging` can't take its place.
fn replace(folder: &Path, staging: &Path) -> io::Result<()> {
    let old = folder.exists().then(|| beside(folder, "old"));
    if let Some(old) = &old {
        fs::rename(folder, old)?;
    }
    if let Err(error) = fs::rename(staging, folder) {
        if let Some(old) = &old {
            drop(fs::rename(old, folder));
        }
        return Err(error);
    }
    if let Some(old) = &old {
        drop(fs::remove_dir_all(old));
    }
    Ok(())
}

/// A hidden folder beside `folder`, of its own to this keep, which no listing reads as a release:
/// `.<name>@<version>.<what>-<pid>-<n>`.
fn beside(folder: &Path, what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let name = folder.file_name().unwrap_or_default().to_string_lossy();
    folder.with_file_name(format!(".{name}.{what}-{}-{n}", std::process::id()))
}

/// `path` under `folder`: a `FilePath` can't leave it.
fn joined(folder: PathBuf, path: &FilePath) -> PathBuf {
    path.parts().fold(folder, |file, part| file.join(part))
}

#[cfg(test)]
mod tests;
