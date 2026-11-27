//! A part of a text by its code point offsets ([the language server], *Links and headings*, rename): what a node's
//! position spans, and an offset range as LSP's range in the client's encoding.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use compmd::tree::{Point, Position};
use lsp_types::Range;

use crate::encoding::Encoding;

/// The text `position` spans in `text`.
pub(crate) fn spanned(text: &str, position: &Position) -> String {
    text.chars()
        .skip(position.start.offset)
        .take(position.end.offset.saturating_sub(position.start.offset))
        .collect()
}

/// The code points `from` to `to` of `text`, as LSP's range.
pub(crate) fn range_of(text: &str, encoding: Encoding, (from, to): (usize, usize)) -> Range {
    let position = Position {
        start: point_at(text, from),
        end: point_at(text, to),
    };
    encoding.span(text, &position)
}

/// The point at code point `offset` of `text`.
fn point_at(text: &str, offset: usize) -> Point {
    let (mut line, mut column) = (1, 1);
    for character in text.chars().take(offset) {
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    Point {
        line,
        column,
        offset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_span_is_its_code_points_and_an_offset_range_a_range_in_the_encoding() {
        let text = "# Café\n\nSee [é](a.md).\n";
        let link = Position {
            start: point_at(text, 12),
            end: point_at(text, 21),
        };
        assert_eq!(spanned(text, &link), "[é](a.md)");
        let range = range_of(text, Encoding::Utf8, (13, 14));
        assert_eq!(
            (range.start.line, range.start.character, range.end.character),
            (2, 5, 7),
            "é takes two bytes"
        );
    }
}
