//! The CompMD processor: parses, validates and expands documents written in CompMD (Component Markdown), as the
//! [CompMD specification](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md) (version 1, draft)
//! describes.
//!
//! It implements stages 1 to 4b, everything up to rendering, as the states of a [`Processor`]: parsing gives a
//! document's [tree], resolving loads its imports and binds every component name, validating checks it against the
//! signatures, loading gives the inputs their values and loads the data, and expanding makes the [expanded
//! tree](expanded) a renderer draws, for one [render context](context); every stage adds its
//! [diagnostics](diagnostic). [`process`](process::process) runs them all on one JSON request, as the WebAssembly
//! build does. It also implements the CompMD CEL profile, in [`cel`], with the `md.` functions in [`md`].
//!
//! The processor does no I/O: the caller hands it files, packs, remote and host data through traits
//! ([`Sources`](resolve::Sources)), and the time zones, so it runs anywhere, a browser among them.
//!
//! ```
//! use compmd::{ParseOptions, Processor};
//!
//! let parsed = Processor::new("a.comp.md", ":: chart[Weekly installs] {src=installs.csv}\n")
//!     .parse(&ParseOptions::default());
//! let json = serde_json::to_value(parsed.tree()).unwrap();
//! assert_eq!(json["children"][0]["type"], "componentLeaf");
//! ```

pub mod cel;
pub mod context;
pub mod diagnostic;
pub mod expand;
pub mod expanded;
mod lines;
pub mod load;
pub mod md;
mod named;
mod nearest;
mod parse;
pub mod process;
pub mod processor;
mod reserved;
pub mod resolve;
mod settings;
pub mod tree;
mod validate;

pub use parse::ParseOptions;
pub use processor::Processor;
