//! The tables' settings ([a document drawn in the editor], decisions 6 and 7), under `compmd.tables`: whether a table
//! is aligned as it is typed, and when the document is formatted.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use serde::Deserialize;

/// The tables' settings, every default filled in.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct TableSettings {
    /// Whether typing a `|` or a newline in a table aligns it (`textDocument/onTypeFormatting`).
    pub(crate) align_on_type: bool,
    /// Whether formatting the document, or a range of it, aligns its tables (`textDocument/formatting`).
    pub(crate) align_on_format: bool,
}

impl Default for TableSettings {
    fn default() -> Self {
        Self {
            align_on_type: true,
            align_on_format: true,
        }
    }
}
