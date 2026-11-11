//! A bundle: its manifest and its files ([hosting], decision 3).
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use std::collections::BTreeMap;
use std::path::Path;

use crate::bundle_error::BundleError;
use crate::files::Files;
use crate::manifest::Manifest;
use crate::place::{Place, release_folder};

/// The manifest's path in a bundle.
pub const MANIFEST: &str = "manifest.json";

/// A bundle the host serves.
#[derive(Clone, Debug, PartialEq)]
pub struct Bundle {
    manifest: Manifest,
    files: Files,
}

impl Bundle {
    /// The bundle embedded in a binary: its files by path, sorted, `manifest.json` among them.
    ///
    /// # Errors
    ///
    /// [`BundleError`] when it has no manifest, or one that can't be used.
    pub fn embedded(files: &'static [(&'static str, &'static [u8])]) -> Result<Self, BundleError> {
        Self::with(Files::Embedded(files))
    }

    /// The bundle in `folder`.
    ///
    /// # Errors
    ///
    /// [`BundleError`] when it has no manifest, or one that can't be used.
    pub fn read(folder: &Path) -> Result<Self, BundleError> {
        Self::with(Files::Folder(folder.to_path_buf()))
    }

    /// A bundle of `files`, by path, `manifest.json` among them: for a host's tests.
    ///
    /// # Errors
    ///
    /// [`BundleError`] when it has no manifest, or one that can't be used.
    pub fn from_files<P: Into<String>>(
        files: impl IntoIterator<Item = (P, Vec<u8>)>,
    ) -> Result<Self, BundleError> {
        let files: BTreeMap<String, Vec<u8>> = files
            .into_iter()
            .map(|(path, bytes)| (path.into(), bytes))
            .collect();
        Self::with(Files::Owned(files))
    }

    fn with(files: Files) -> Result<Self, BundleError> {
        let text = files.read(MANIFEST).ok_or(BundleError::NoManifest)?;
        let manifest = Manifest::parse(&String::from_utf8_lossy(&text))?;
        Ok(Self { manifest, files })
    }

    /// Its manifest.
    #[must_use]
    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// The bytes of the frames' file at `path`, inside `frames/<hash>/`.
    #[must_use]
    pub fn frame_file(&self, path: &str) -> Option<Vec<u8>> {
        self.files
            .read(&format!("frames/{}/{path}", self.manifest.frames.hash))
    }

    /// The bytes of the pack file at `place`, when the bundle holds that pack's version.
    #[must_use]
    pub fn pack_file(&self, place: &Place) -> Option<Vec<u8>> {
        self.manifest.pack(&place.pack, &place.version)?;
        self.files.read(&format!(
            "packs/{}/{}",
            release_folder(&place.pack, &place.version),
            place.path
        ))
    }

    /// The frames' `base` on `origin` (`compmd://localhost`, `https://frames.example`), as `@compmd/sandbox` takes it.
    #[must_use]
    pub fn frames_base(&self, origin: &str) -> String {
        format!(
            "{}/frames/{}/",
            origin.trim_end_matches('/'),
            self.manifest.frames.hash
        )
    }
}

#[cfg(test)]
pub(crate) mod tests;
