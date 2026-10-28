//! The CompMD processor: parses, validates and expands documents written in CompMD (Component Markdown), as the
//! [CompMD specification](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md) (version 1, draft)
//! describes.
//!
//! This version implements stages 1 to 3. Stage 1, parsing: [`parse`] turns a document's text into its
//! [tree] and its [diagnostics](diagnostic). Stage 2, [`resolve`](resolve::resolve), loads its imports and binds
//! every component name; stage 3, [`validate`](validate::validate), checks it against the signatures. It also
//! implements the CompMD CEL profile, in [`cel`], with the `md.` functions in [`md`].
//!
//! ```
//! use compmd::{ParseOptions, parse};
//!
//! let parsed = parse(":: chart[Weekly installs] {src=installs.csv}\n", &ParseOptions::default());
//! let json = serde_json::to_value(&parsed.root).unwrap();
//! assert_eq!(json["children"][0]["type"], "componentLeaf");
//! ```

pub mod cel;
pub mod diagnostic;
mod lines;
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
