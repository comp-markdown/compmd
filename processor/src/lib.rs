//! The CompMD processor: parses, validates and expands documents written in CompMD (Component Markdown), as the
//! [CompMD specification](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md) (version 1, draft)
//! describes.
//!
//! This version implements stage 1, parsing: [`parse`] turns a document's text into its [tree](tree) and its
//! [diagnostics](diagnostic). It also implements the CompMD CEL profile, in [`cel`], with the `md.` functions in
//! [`md`].
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
pub mod md;
mod nearest;
mod parse;
pub mod tree;

pub use parse::{ParseOptions, Parsed, parse};
