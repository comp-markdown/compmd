//! Why a bundle's manifest can't be used.

/// What is wrong with a `manifest.json`.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ManifestError {
    /// It isn't the manifest's JSON, or a name, path or integrity in it isn't one.
    #[error("the bundle's manifest can't be read: {0}")]
    Json(String),
    /// It was written by a version of `@compmd/vite` this crate doesn't know.
    #[error(
        "the bundle's manifest is format {found}, and this compmd-host reads format {known}: upgrade compmd-host to the version of the @compmd packages that built the bundle"
    )]
    UnknownFormat {
        /// The format it names.
        found: u32,
        /// The format this crate reads.
        known: u32,
    },
    /// The frames' hash isn't 16 lowercase hex digits, so it could name another folder.
    #[error("the frames' hash `{0}` isn't 16 lowercase hex digits")]
    Hash(String),
}
