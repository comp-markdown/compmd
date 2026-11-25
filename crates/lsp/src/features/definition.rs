//! Go to definition ([the language server], *Go to definition*): a component's name to its definition fence, or to
//! the file or pack document that exports it; an import's `from` to what it loads; `md.data.<name>` to its data
//! import.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#completion-hover-signature-help-go-to-definition

use std::fs;

use compmd::hints::Defined;
use lsp_types::{Location, Position, Range, Uri};

use super::Asked;

/// Where `defined` is, as LSP's location: in the asked document, or in the file it names; `None` for a file that isn't
/// on the disk.
pub(crate) fn location(defined: &Defined, asked: &Asked) -> Option<Location> {
    let (uri, text) = match &defined.file {
        None => (asked.uri.clone(), asked.text.to_owned()),
        Some(file) => {
            let path = asked.places.path_of(file)?;
            let uri: Uri = url::Url::from_file_path(&path)
                .ok()?
                .as_str()
                .parse()
                .ok()?;
            (uri, fs::read_to_string(&path).unwrap_or_default())
        }
    };
    let top = Range {
        start: Position::default(),
        end: Position::default(),
    };
    let range = defined
        .range
        .as_ref()
        .map_or(top, |range| asked.encoding.range(&text, range));
    Some(Location { uri, range })
}

#[cfg(test)]
mod tests;
