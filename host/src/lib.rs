//! `compmd-host`: what every host of CompMD documents shares, whatever its platform ([hosting], decision 1).
//!
//! A host builds a **bundle** before anyone reads (`@compmd/vite`, or `buildBundle`): the sandbox's frames under a
//! content hash, the packs it installs by their place, and a `manifest.json` describing it all. This crate reads that
//! bundle ([`Bundle`], [`Manifest`]), keeps the packs it fetches later in a [`Store`], and answers requests for both
//! with [`respond`], the serving contract every adapter puts behind its origin: a Tauri scheme, an Electron protocol,
//! a web server's route.
//!
//! ```
//! use compmd_host::{respond, Bundle, NoStore};
//!
//! let bundle = Bundle::from_files([
//!     ("manifest.json", br#"{"format": 1, "frames": {"hash": "1df336e75685de8c"}}"#.to_vec()),
//!     ("frames/1df336e75685de8c/strict.html", b"<!doctype html>".to_vec()),
//! ])?;
//! let request = http::Request::get("/frames/1df336e75685de8c/strict.html").body(())?;
//! let response = respond(&bundle, &NoStore, &request);
//! assert_eq!(response.status(), http::StatusCode::OK);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! With the `processor` feature, [`HostPacks`] gives the processor (`compmd`) the same packs the frames load.
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

mod bundle;
mod bundle_error;
mod content_type;
mod files;
mod folder_store;
mod manifest;
mod manifest_error;
#[cfg(feature = "processor")]
mod packs;
mod place;
mod respond;
mod store;

pub use bundle::{Bundle, MANIFEST};
pub use bundle_error::BundleError;
pub use content_type::content_type;
pub use files::Files;
pub use folder_store::FolderStore;
pub use manifest::{BundledPack, FORMAT, Frames, Library, Manifest, Trust};
pub use manifest_error::ManifestError;
#[cfg(feature = "processor")]
pub use packs::HostPacks;
pub use place::Place;
pub use respond::respond;
pub use store::{NoStore, Store, StoredRelease};
