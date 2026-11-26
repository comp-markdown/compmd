//! The registry rules ([the registry rules]; [hosting], H1): resolving a request's packs from registries, the policy
//! before any download, checking a tarball and reading it, once, for every host. The rules do no I/O: a [`Resolution`]
//! says what it needs next, and the host does it its own way, synchronously on a thread of its own or awaited, and
//! gives the answer back.
//!
//! ```
//! use std::collections::BTreeMap;
//! use compmd_host::registry::{Answer, Next, Options, Resolution, Step};
//!
//! let options = Options::new(BTreeMap::from([("*".to_owned(), "https://registry.npmjs.org/".to_owned())]));
//! let mut resolution = Resolution::new(options);
//! let resolved = loop {
//!     match resolution.next_steps() {
//!         Next::Steps(steps) => {
//!             for step in steps {
//!                 // The host's I/O: here, a request that imports nothing.
//!                 let answer = match step {
//!                     Step::Missing { .. } => Answer::Missing { packs: Vec::new() },
//!                     _ => Answer::Done,
//!                 };
//!                 resolution.give(step.id(), answer);
//!             }
//!         }
//!         Next::Done(resolved) => break resolved,
//!     }
//! };
//! assert!(resolved.releases.is_empty());
//! ```
//!
//! [the registry rules]: https://github.com/comp-markdown/compmd/blob/main/docs/registry-rules.md
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

#[cfg(feature = "client")]
mod client;
mod file_types;
mod folder_cache;
mod integrity;
#[cfg(any(test, feature = "local-registry"))]
mod local_registry;
mod metadata;
mod options;
#[cfg(feature = "client")]
mod proxy;
mod resolution;
#[cfg(feature = "client")]
mod resolve;
mod resolved;
mod step;
mod tar;

#[cfg(feature = "client")]
pub use client::{Client, Metadata};
pub use folder_cache::FolderCache;
#[cfg(any(test, feature = "local-registry"))]
pub use local_registry::{LocalRegistry, ServedPack, tarball};
pub use metadata::{
    ABBREVIATED, VersionInfo, answered_versions, metadata_url, off_origin, versions_of,
};
pub use options::{DEFAULT_MAX_PACK_SIZE, Denied, Options};
#[cfg(feature = "client")]
pub use proxy::Proxy;
pub use resolution::{Next, Resolution};
#[cfg(feature = "client")]
pub use resolve::resolve;
pub use resolved::{FrameFile, Problem, Release, Resolved};
pub use step::{Answer, Files, KeptRelease, KeptVersions, Needed, Step, StepId};
