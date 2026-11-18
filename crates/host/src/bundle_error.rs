//! Why a bundle can't be used.

use crate::manifest_error::ManifestError;

/// What is wrong with a bundle.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BundleError {
    /// It has no `manifest.json`: the frames weren't built, or the folder isn't a bundle.
    #[error(
        "no bundle here: its manifest.json is missing; build the page (with @compmd/vite) first"
    )]
    NoManifest,
    /// Its manifest can't be used.
    #[error(transparent)]
    Manifest(#[from] ManifestError),
}
