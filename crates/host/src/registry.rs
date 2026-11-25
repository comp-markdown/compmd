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

mod file_types;
mod integrity;
mod metadata;
mod options;
mod resolution;
mod resolved;
mod step;
mod tar;

pub use metadata::{ABBREVIATED, VersionInfo};
pub use options::{DEFAULT_MAX_PACK_SIZE, Denied, Options};
pub use resolution::{Next, Resolution};
pub use resolved::{FrameFile, Problem, Release, Resolved};
pub use step::{Answer, Files, KeptRelease, KeptVersions, Needed, Step, StepId};
