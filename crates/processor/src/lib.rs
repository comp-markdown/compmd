//! The [CompMD](https://compmd.dev/) processor: parses, validates and expands documents written in CompMD (Component
//! Markdown), as the [CompMD specification](https://github.com/comp-markdown/compmd/blob/main/spec/compmd-spec-v1.md)
//! describes, at the version [`SPEC_VERSION`] names.
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

mod allowed_origins;
mod breakpoint;
pub mod cel;
pub mod components;
pub mod context;
pub mod diagnostic;
mod direction;
pub mod expand;
pub mod expanded;
pub mod frontmatter;
pub mod hints;
pub mod host;
mod line_index;
mod lines;
mod load;
pub mod md;
mod named;
mod nearest;
mod nesting;
mod parse;
pub mod plain;
pub mod policy;
pub mod processor;
pub mod render;
mod reserved;
mod resolve;
mod settings;
pub mod tree;
mod validate;
pub mod values;

/// The names packs and their files are known by: `PackName`, `ReleaseId`, `FilePath`, `Place`, `Integrity`.
pub use allowed_origins::AllowedOrigins;
pub use compmd_pack as pack;
pub use parse::{ParseOptions, TextEdit};
pub use processor::Processor;
pub use resolve::{Ambient, Language};

/// The version of the CompMD specification this processor implements (spec §1): a draft, `1.0-draft.N`, until the
/// spec's version 1 is released. The spec is versioned on its own, apart from this crate.
///
/// ```
/// assert!(compmd::SPEC_VERSION.starts_with("1.0"));
/// ```
pub const SPEC_VERSION: &str = "1.0-draft.18";

#[cfg(test)]
mod tests {
    #[test]
    fn the_readme_names_the_spec_version_implemented() {
        assert!(include_str!("../README.md").contains(super::SPEC_VERSION));
    }
}
