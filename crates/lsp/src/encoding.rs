//! Positions ([the language server], *Positions*): the processor counts code points; LSP counts UTF-16 by default, or
//! UTF-8 or UTF-32 when the client offers them, which the server then asks for.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#architecture

use lsp_types::{ClientCapabilities, Position, PositionEncodingKind, Range};

/// How a position's `character` counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Encoding {
    /// Bytes of UTF-8.
    Utf8,
    /// UTF-16 units: LSP's default.
    Utf16,
    /// Code points, as the processor counts.
    Utf32,
}

impl Encoding {
    /// The encoding to use with a client of `capabilities`: UTF-32 when it offers it, else UTF-8, else UTF-16.
    pub(crate) fn negotiate(capabilities: &ClientCapabilities) -> Self {
        let offered = capabilities
            .general
            .as_ref()
            .and_then(|general| general.position_encodings.as_ref());
        let offers =
            |kind: &PositionEncodingKind| offered.is_some_and(|offered| offered.contains(kind));
        if offers(&PositionEncodingKind::UTF32) {
            Self::Utf32
        } else if offers(&PositionEncodingKind::UTF8) {
            Self::Utf8
        } else {
            Self::Utf16
        }
    }

    /// The encoding's name, as `initialize`'s answer gives it.
    pub(crate) fn kind(self) -> PositionEncodingKind {
        match self {
            Self::Utf8 => PositionEncodingKind::UTF8,
            Self::Utf16 => PositionEncodingKind::UTF16,
            Self::Utf32 => PositionEncodingKind::UTF32,
        }
    }

    /// How many units `c` takes.
    fn width(self, c: char) -> usize {
        match self {
            Self::Utf8 => c.len_utf8(),
            Self::Utf16 => c.len_utf16(),
            Self::Utf32 => 1,
        }
    }

    /// The code point offset in `text` of `position`; a position past its line's end is the line's end, and one past
    /// the text's last line is its end.
    pub(crate) fn offset(self, text: &str, position: Position) -> usize {
        let mut offset = 0;
        let mut line = 0;
        let mut chars = text.chars().peekable();
        while line < position.line {
            match chars.next() {
                Some('\n') => line += 1,
                Some(_) => {}
                None => return offset,
            }
            offset += 1;
        }
        let mut units = 0;
        while let Some(&c) = chars.peek() {
            let width = self.width(c);
            if c == '\n' || units + width > position.character as usize {
                break;
            }
            units += width;
            offset += 1;
            chars.next();
        }
        offset
    }

    /// The position of the processor's `line` and `column` (from 1, in code points) in `text`.
    pub(crate) fn position(self, text: &str, line: usize, column: usize) -> Position {
        let line_text = text.split('\n').nth(line.saturating_sub(1)).unwrap_or("");
        let units: usize = line_text
            .chars()
            .take(column.saturating_sub(1))
            .map(|c| self.width(c))
            .sum();
        Position {
            line: u32::try_from(line.saturating_sub(1)).unwrap_or(u32::MAX),
            character: u32::try_from(units).unwrap_or(u32::MAX),
        }
    }

    /// The processor's `range` of `text` as LSP's: its end, when it has none, is its start.
    pub(crate) fn range(self, text: &str, range: &compmd::diagnostic::Range) -> Range {
        let start = self.position(text, range.start.line, range.start.column);
        let end = range
            .end
            .map_or(start, |end| self.position(text, end.line, end.column));
        Range { start, end }
    }
}

#[cfg(test)]
mod tests {
    use lsp_types::GeneralClientCapabilities;

    use super::*;

    fn offering(kinds: Option<Vec<PositionEncodingKind>>) -> ClientCapabilities {
        ClientCapabilities {
            general: Some(GeneralClientCapabilities {
                position_encodings: kinds,
                ..GeneralClientCapabilities::default()
            }),
            ..ClientCapabilities::default()
        }
    }

    #[test]
    fn the_client_s_offer_decides_and_utf16_is_the_default() {
        assert_eq!(
            Encoding::negotiate(&ClientCapabilities::default()),
            Encoding::Utf16
        );
        assert_eq!(Encoding::negotiate(&offering(None)), Encoding::Utf16);
        assert_eq!(
            Encoding::negotiate(&offering(Some(vec![
                PositionEncodingKind::UTF8,
                PositionEncodingKind::UTF16
            ]))),
            Encoding::Utf8
        );
        assert_eq!(
            Encoding::negotiate(&offering(Some(vec![
                PositionEncodingKind::UTF8,
                PositionEncodingKind::UTF32
            ]))),
            Encoding::Utf32
        );
        assert_eq!(Encoding::Utf8.kind(), PositionEncodingKind::UTF8);
        assert_eq!(Encoding::Utf16.kind(), PositionEncodingKind::UTF16);
        assert_eq!(Encoding::Utf32.kind(), PositionEncodingKind::UTF32);
    }

    /// A line with a character outside the BMP: one code point, two UTF-16 units, four UTF-8 bytes.
    const TEXT: &str = "# Notes\na😀b\nend";

    #[test]
    fn a_position_is_a_code_point_offset_in_each_encoding() {
        let at = |line, character| Position { line, character };
        assert_eq!(
            Encoding::Utf16.offset(TEXT, at(1, 3)),
            10,
            "after the emoji"
        );
        assert_eq!(Encoding::Utf8.offset(TEXT, at(1, 5)), 10);
        assert_eq!(Encoding::Utf32.offset(TEXT, at(1, 2)), 10);
        assert_eq!(
            Encoding::Utf16.offset(TEXT, at(1, 2)),
            9,
            "inside the emoji: before it"
        );
        assert_eq!(
            Encoding::Utf16.offset(TEXT, at(1, 99)),
            11,
            "past the line's end"
        );
        assert_eq!(
            Encoding::Utf16.offset(TEXT, at(9, 0)),
            15,
            "past the text's end"
        );
        assert_eq!(Encoding::Utf16.offset(TEXT, at(0, 0)), 0);
    }

    #[test]
    fn a_processor_place_is_a_position_in_each_encoding() {
        assert_eq!(
            Encoding::Utf16.position(TEXT, 2, 3),
            Position {
                line: 1,
                character: 3
            }
        );
        assert_eq!(
            Encoding::Utf8.position(TEXT, 2, 3),
            Position {
                line: 1,
                character: 5
            }
        );
        assert_eq!(
            Encoding::Utf32.position(TEXT, 2, 3),
            Position {
                line: 1,
                character: 2
            }
        );
        assert_eq!(
            Encoding::Utf16.position(TEXT, 9, 1),
            Position {
                line: 8,
                character: 0
            }
        );
        assert_eq!(
            Encoding::Utf16.position(TEXT, 0, 0),
            Position {
                line: 0,
                character: 0
            }
        );
    }
}
