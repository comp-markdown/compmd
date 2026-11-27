//! How a root's wiki links are read and followed ([the language server], *Links and headings*, Q11): a CompMD
//! document has them when it opts in with `md.syntax`, a plain `.md` file when `markdown.wikiLinks` is on, and the
//! server follows them while `links.wiki` is on. The project index keeps them, as its files are parsed with them and
//! the documents' workers read them.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use crate::settings::Settings;

/// The wiki-link switches of a root.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WikiLinks {
    /// Whether a plain `.md` file parses them (`markdown.wikiLinks`).
    pub(crate) in_markdown_files: bool,
    /// Whether the server follows them (`links.enabled` and `links.wiki`).
    pub(crate) followed: bool,
}

impl WikiLinks {
    /// The switches `settings` give.
    pub(crate) const fn of(settings: &Settings) -> Self {
        Self {
            in_markdown_files: settings.markdown.wiki_links,
            followed: settings.links.follows_wiki(),
        }
    }
}

impl Default for WikiLinks {
    fn default() -> Self {
        Self::of(&Settings::default())
    }
}
