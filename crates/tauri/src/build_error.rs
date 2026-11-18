//! Why a bundle can't be embedded.

use std::path::PathBuf;

use compmd_host::ManifestError;

/// Why `build::bundle` can't embed a bundle, each saying what to do.
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    /// The folder has no `manifest.json`: the page wasn't built yet.
    #[error("no bundle in {}: build the page with @compmd/vite first", .0.display())]
    NoBundle(PathBuf),
    /// The manifest is of a format this plugin doesn't read.
    #[error("the bundle's manifest is format {found}, and tauri-plugin-compmd {} reads format {known}: use the plugin of the same release as @compmd/vite", env!("CARGO_PKG_VERSION"))]
    Format {
        /// The format it has.
        found: u32,
        /// The format this plugin reads.
        known: u32,
    },
    /// The manifest isn't one.
    #[error("the bundle's manifest can't be read: {0}")]
    Manifest(ManifestError),
    /// The bundle was built for an http origin, so it names no scheme.
    #[error("{}", crate::BUILT_WITHOUT_SOURCE)]
    NoScheme,
    /// The scheme the bundle names isn't a scheme's name.
    #[error(
        "the bundle's source names `{0}`, which isn't a scheme's name: lowercase letters, digits, `+`, `-` and `.`, starting with a letter"
    )]
    Scheme(String),
    /// A folder of the bundle can't be read.
    #[error("{} can't be read: {why}", .folder.display())]
    Unreadable {
        /// The folder.
        folder: PathBuf,
        /// What reading it said.
        why: String,
    },
}
