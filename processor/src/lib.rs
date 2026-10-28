//! The CompMD processor: parses, validates and expands documents written in CompMD (Component Markdown), as the
//! [CompMD specification](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md) (version 1, draft)
//! describes.
//!
//! It implements stages 1 to 4b, everything up to rendering. Stage 1, parsing: [`parse`] turns a document's text
//! into its [tree] and its [diagnostics](diagnostic). Stage 2, [`resolve`](resolve::resolve), loads its imports and
//! binds every component name; stage 3, [`validate`](validate::validate), checks it against the signatures. Stage
//! 4a, [`load`](load::load), gives the inputs their values and loads the data; stage 4b,
//! [`expand`](expand::expand), makes the [expanded tree](expanded) a renderer draws, for one [render
//! context](context). It also implements the CompMD CEL profile, in [`cel`], with the `md.` functions in [`md`].
//!
//! The processor does no I/O: the caller hands it files, packs, remote and host data through traits, and the time
//! zones, so it runs anywhere, a browser among them.
//!
//! ```
//! use compmd::{ParseOptions, parse};
//!
//! let parsed = parse(":: chart[Weekly installs] {src=installs.csv}\n", &ParseOptions::default());
//! let json = serde_json::to_value(&parsed.root).unwrap();
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
mod reserved;
pub mod resolve;
mod settings;
pub mod tree;
pub mod validate;

pub use parse::{ParseOptions, Parsed, parse};
