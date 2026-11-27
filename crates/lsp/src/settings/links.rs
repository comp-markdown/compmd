//! Links and headings' settings ([the language server], decision 15): `links.enabled` turns them off altogether, for a
//! writer who keeps Marksman's or VS Code's own, `links.completion`, `links.diagnostics` and `links.rename` each of
//! those features, and `links.wiki` wiki links in each of them.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use serde::Deserialize;

/// `compmd.links`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub(crate) struct LinkSettings {
    /// Whether links and headings are the server's at all: their definitions, references, hovers and symbols.
    pub(crate) enabled: bool,
    /// Whether wiki links are followed, where a document has them, while `enabled`.
    pub(crate) wiki: bool,
    /// Which of their features are on, while `enabled`.
    #[serde(flatten)]
    pub(crate) features: LinkFeatures,
}

/// The features of links and headings each switch turns off, beside `links.enabled`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub(crate) struct LinkFeatures {
    /// A link's target and a reference's label completed.
    pub(crate) completion: bool,
    /// A broken link warned about.
    pub(crate) diagnostics: bool,
    /// Headings, labels and files renamed with their links.
    pub(crate) rename: bool,
}

impl Default for LinkSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            wiki: true,
            features: LinkFeatures::default(),
        }
    }
}

impl Default for LinkFeatures {
    fn default() -> Self {
        Self {
            completion: true,
            diagnostics: true,
            rename: true,
        }
    }
}

impl LinkSettings {
    /// Whether links are completed: `enabled` and `completion` both.
    pub(crate) const fn completes(self) -> bool {
        self.enabled && self.features.completion
    }

    /// Whether broken links are warned about: `enabled` and `diagnostics` both.
    pub(crate) const fn warns(self) -> bool {
        self.enabled && self.features.diagnostics
    }

    /// Whether renames edit links: `enabled` and `rename` both.
    pub(crate) const fn renames(self) -> bool {
        self.enabled && self.features.rename
    }

    /// Whether wiki links are followed: `enabled` and `wiki` both.
    pub(crate) const fn follows_wiki(self) -> bool {
        self.enabled && self.wiki
    }
}
