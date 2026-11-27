//! Links and headings' settings ([the language server], decision 15): `links.enabled` turns them off altogether, for a
//! writer who keeps Marksman's or VS Code's own.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use serde::Deserialize;

/// `compmd.links`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub(crate) struct LinkSettings {
    /// Whether links and headings are the server's at all: their definitions, references, hovers and symbols.
    pub(crate) enabled: bool,
    /// Whether a link's target and a reference's label are completed, while `enabled`.
    pub(crate) completion: bool,
}

impl Default for LinkSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            completion: true,
        }
    }
}

impl LinkSettings {
    /// Whether links are completed: `enabled` and `completion` both.
    pub(crate) const fn completes(self) -> bool {
        self.enabled && self.completion
    }
}
