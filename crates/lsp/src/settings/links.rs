//! Links and headings' settings ([the language server], decision 15): `links.enabled` turns them off altogether, for a
//! writer who keeps Marksman's or VS Code's own.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use serde::Deserialize;

/// `compmd.links`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub(crate) struct LinkSettings {
    /// Whether links and headings are the server's at all: their definitions, references, hovers and symbols.
    pub(crate) enabled: bool,
}

impl Default for LinkSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}
