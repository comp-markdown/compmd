//! A resolution's cache in a folder ([the language server], decision 6): the releases fetched, in a [`FolderStore`],
//! and each pack's versions at each registry, kept for as long as the registry's `Cache-Control` says. A version is
//! fetched once per machine, and a session without the network uses what was kept.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use compmd_pack::{FilePath, Integrity, PackName, ReleaseId};
use serde::{Deserialize, Serialize};

use super::metadata::VersionInfo;
use super::step::{Files, KeptRelease, KeptVersions};
use crate::folder_store::FolderStore;
use crate::store::Store;

/// A pack's versions at one registry, as the cache keeps them.
#[derive(Serialize, Deserialize)]
struct Listed {
    versions: Vec<VersionInfo>,
    /// Until when they are fresh, in seconds since 1970.
    until: u64,
}

/// A folder the registry rules' cache steps are answered from: releases under `@<scope>/`, as [`FolderStore`] keeps
/// them, and versions under `versions/`.
///
/// ```
/// use compmd_host::registry::FolderCache;
///
/// let cache = FolderCache::new(std::env::temp_dir().join("compmd-doc-cache"));
/// assert_eq!(cache.kept_versions("https://registry.npmjs.org/", &"@kit/none".parse()?), None);
/// # Ok::<(), compmd_host::pack::Invalid>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderCache {
    root: PathBuf,
    store: FolderStore,
}

/// The seconds since 1970, now.
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

impl FolderCache {
    /// The cache in `root`, which needn't exist yet.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            store: FolderStore::new(&root),
            root,
        }
    }

    /// The file of `pack`'s versions, at every registry.
    fn versions_file(&self, pack: &PackName) -> PathBuf {
        self.root
            .join("versions")
            .join(format!("@{}", pack.scope()))
            .join(format!("{}.json", pack.name()))
    }

    /// Every registry's versions of `pack` kept.
    fn listed(&self, pack: &PackName) -> BTreeMap<String, Listed> {
        fs::read(self.versions_file(pack))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// `pack`'s versions at `registry`, if kept, and whether they are still fresh: [`super::Step::KeptVersions`].
    #[must_use]
    pub fn kept_versions(&self, registry: &str, pack: &PackName) -> Option<KeptVersions> {
        let listed = self.listed(pack).remove(registry)?;
        Some(KeptVersions {
            fresh: now() < listed.until,
            versions: listed.versions,
        })
    }

    /// Keeps `pack`'s `versions` at `registry`, fresh for `max_age` seconds: [`super::Step::KeepVersions`].
    ///
    /// # Errors
    ///
    /// An I/O error.
    pub fn keep_versions(
        &self,
        registry: &str,
        pack: &PackName,
        versions: Vec<VersionInfo>,
        max_age: u64,
    ) -> io::Result<()> {
        let mut listed = self.listed(pack);
        let until = now().saturating_add(max_age);
        listed.insert(registry.to_owned(), Listed { versions, until });
        let file = self.versions_file(pack);
        if let Some(folder) = file.parent() {
            fs::create_dir_all(folder)?;
        }
        fs::write(file, serde_json::to_vec(&listed)?)
    }

    /// The files kept of `release`, when the release kept has its integrity: [`super::Step::Kept`].
    #[must_use]
    pub fn kept(&self, release: &KeptRelease) -> Option<Files> {
        let id = ReleaseId::new(release.pack.clone(), release.version.clone());
        let kept = self
            .store
            .releases(&release.pack)
            .into_iter()
            .any(|stored| {
                stored.release == id && stored.integrity.to_string() == release.integrity
            });
        kept.then(|| self.store.files_of(&id).map(Files::from))
            .flatten()
    }

    /// Keeps `release`'s checked `files`: [`super::Step::Keep`].
    ///
    /// # Errors
    ///
    /// An I/O error, or a release whose integrity or paths aren't ones a store keeps.
    pub fn keep(&self, release: &KeptRelease, files: Files) -> io::Result<()> {
        let invalid =
            |error: compmd_pack::Invalid| io::Error::new(io::ErrorKind::InvalidInput, error);
        let integrity: Integrity = release.integrity.parse().map_err(invalid)?;
        let id = ReleaseId::new(release.pack.clone(), release.version.clone());
        let texts = files
            .text
            .into_iter()
            .map(|(path, text)| (path, text.into_bytes()));
        let files = texts
            .chain(files.binary)
            .map(|(path, bytes)| Ok((path.parse::<FilePath>().map_err(invalid)?, bytes)))
            .collect::<io::Result<Vec<_>>>()?;
        self.store.keep(&id, &integrity, files)
    }
}

#[cfg(test)]
mod tests;
