//! The CompMD processor: parses, validates and expands documents written in CompMD (Component Markdown), as the
//! [CompMD specification](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md) (version 1, draft)
//! describes.
//!
//! It implements stages 1 to 4b, everything up to rendering, as the states of a [`Processor`]: parsing gives a
//! document's [tree], resolving loads its imports and binds every component name, validating checks it against the
//! signatures, loading gives the inputs their values and loads the data, and expanding makes the [expanded
//! tree](expanded) a renderer draws, for one [render context](context); every stage adds its
//! [diagnostics](diagnostic). Expressions follow the CompMD CEL profile, with the `md.` functions (spec §8.2).
//!
//! The processor does no I/O: the caller hands it files, packs, remote and host data, and the time zones, through the
//! traits of [`host`], so it runs anywhere, a browser among them.
//!
//! ```
//! use compmd::{ParseOptions, Processor};
//!
//! let parsed = Processor::new("a.comp.md", ":: chart[Weekly installs] {src=installs.csv}\n")
//!     .parse(&ParseOptions::default());
//! let json = serde_json::to_value(parsed.tree()).unwrap();
//! assert_eq!(json["children"][0]["type"], "componentLeaf");
//! ```

mod breakpoint;
mod cel;
pub mod components;
pub mod context;
pub mod diagnostic;
pub mod expand;
pub mod expanded;
pub mod frontmatter;
pub mod hints;
pub mod host;
mod lines;
mod load;
mod md;
mod named;
mod nearest;
mod parse;
pub mod plain;
pub mod policy;
pub mod processor;
mod reserved;
mod resolve;
mod settings;
pub mod tree;
mod validate;
pub mod values;

pub use parse::{ParseOptions, TextEdit};
pub use processor::Processor;
pub use resolve::{Ambient, Language};
