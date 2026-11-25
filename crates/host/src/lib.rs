//! `compmd-host`: what every host of [CompMD](https://compmd.dev/) documents shares, whatever its platform
//! ([hosting], decision 1).
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
//! With the `processor` feature, [`HostPacks`] gives the processor (`compmd`) the same packs the frames load. With the
//! `registry` feature, [`registry`] has the registry rules: resolving packs from registries, as a state machine the
//! host drives with its own I/O.
//! Packs, releases, places and integrities are `compmd-pack`'s types, re-exported here as [`pack`], the same the
//! processor uses.
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

/// The names packs and their files are known by.
pub use compmd_pack as pack;

mod bundle;
mod bundle_error;
mod bundled_pack;
mod content_type;
mod files;
mod folder_store;
mod frames;
mod frames_hash;
mod headers;
mod library;
mod manifest;
mod manifest_error;
mod no_store;
#[cfg(feature = "processor")]
mod packs;
mod refusal;
#[cfg(feature = "registry")]
pub mod registry;
mod respond;
mod route;
mod store;
mod stored_release;
mod trust;

pub use bundle::{Bundle, MANIFEST};
pub use bundle_error::BundleError;
pub use bundled_pack::BundledPack;
pub use content_type::content_type;
pub use folder_store::FolderStore;
pub use frames::Frames;
pub use frames_hash::FramesHash;
pub use headers::{refusal_headers, served_headers};
pub use library::Library;
pub use manifest::{FORMAT, Manifest};
pub use manifest_error::ManifestError;
pub use no_store::NoStore;
#[cfg(feature = "processor")]
pub use packs::HostPacks;
pub use refusal::Refusal;
pub use respond::{refused, respond};
pub use route::{Route, route};
pub use store::Store;
pub use stored_release::StoredRelease;
pub use trust::Trust;
