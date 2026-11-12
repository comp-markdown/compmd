//! `compmd-pack`: the names CompMD packs and their files are known by (spec §6.4), each checked once, when it is
//! made, so a name that could leave a folder, or isn't a pack's, can't exist.
//!
//! The processor (`compmd`, which re-exports this crate as `compmd::pack`) and every host's code (`compmd-host`) use
//! the same types, so a pack's name, a release and a file's place mean one thing everywhere.
//!
//! ```
//! use compmd_pack::{Integrity, PackName, Place};
//!
//! let place: Place = "@kit/mermaid@1.2.0/dist/mermaid.js".parse()?;
//! assert_eq!(place.release.pack, "@kit/mermaid".parse::<PackName>()?);
//! assert_eq!(place.release.to_string(), "@kit/mermaid@1.2.0");
//! assert_eq!(place.path.as_str(), "dist/mermaid.js");
//!
//! let integrity: Integrity = "sha512-9c1fQe0=".parse()?;
//! assert_eq!(integrity.to_string(), "sha512-9c1fQe0=");
//! # Ok::<(), compmd_pack::Invalid>(())
//! ```

mod file_path;
mod integrity;
mod invalid;
mod name;
mod place;
mod release;

pub use file_path::FilePath;
pub use integrity::{Algorithm, Integrity};
pub use invalid::Invalid;
pub use name::PackName;
pub use place::Place;
pub use release::ReleaseId;
