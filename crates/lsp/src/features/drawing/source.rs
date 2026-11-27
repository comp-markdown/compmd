//! A document's text by its lines, for the plan: each line's characters, and a place in it as LSP's position, in the
//! encoding agreed at `initialize`. Lines and columns count from 0, columns in code points, as the processor's.

use compmd::tree::Point;
use lsp_types::{Position, Range};

use super::line_span::{LineSpan, lsp_line};
use crate::encoding::Encoding;

/// A document's text, by its lines.
pub(crate) struct Source<'t> {
    lines: Vec<&'t str>,
    encoding: Encoding,
}

impl<'t> Source<'t> {
    /// The lines of `text`, a `\r` before each line's end let go, its positions counted in `encoding`.
    pub(crate) fn new(text: &'t str, encoding: Encoding) -> Self {
        let lines = text
            .split('\n')
            .map(|line| line.strip_suffix('\r').unwrap_or(line))
            .collect();
        Self { lines, encoding }
    }

    /// The line `line`; empty past the text's end.
    pub(crate) fn line(&self, line: usize) -> &'t str {
        self.lines.get(line).copied().unwrap_or_default()
    }

    /// The characters of `line` from `from` up to `to`.
    pub(crate) fn slice(&self, line: usize, from: usize, to: usize) -> String {
        self.line(line)
            .chars()
            .skip(from)
            .take(to.saturating_sub(from))
            .collect()
    }

    /// The character of `line` at `column`.
    pub(crate) fn char_at(&self, line: usize, column: usize) -> Option<char> {
        self.line(line).chars().nth(column)
    }

    /// How many characters `line` has.
    pub(crate) fn len(&self, line: usize) -> usize {
        self.line(line).chars().count()
    }

    /// The column past `line`'s last character that isn't a space.
    pub(crate) fn trimmed_len(&self, line: usize) -> usize {
        self.line(line).trim_end().chars().count()
    }

    /// The column of `line`'s first character from `from` on that isn't a space or a tab.
    pub(crate) fn skip_spaces(&self, line: usize, from: usize) -> usize {
        from + self
            .line(line)
            .chars()
            .skip(from)
            .take_while(|c| *c == ' ' || *c == '\t')
            .count()
    }

    /// The position of `column` on `line`.
    pub(crate) fn position(&self, line: usize, column: usize) -> Position {
        let before: String = self.line(line).chars().take(column).collect();
        Position {
            line: lsp_line(line),
            character: self.encoding.units(&before),
        }
    }

    /// The range of `line` from `from` up to `to`.
    pub(crate) fn range(&self, line: usize, from: usize, to: usize) -> Range {
        Range {
            start: self.position(line, from),
            end: self.position(line, to),
        }
    }

    /// The range from `column` on `first` to the end of `last`.
    pub(crate) fn to_end(&self, (first, column): (usize, usize), last: usize) -> Range {
        Range {
            start: self.position(first, column),
            end: self.position(last, self.len(last)),
        }
    }

    /// The range of the whole lines `lines`: to the start of the line after, or the last line's end at the text's.
    pub(crate) fn whole_lines(&self, lines: LineSpan) -> Range {
        let (first, last) = (lines.first as usize, lines.last as usize);
        let end = if last + 1 < self.lines.len() {
            self.position(last + 1, 0)
        } else {
            self.position(last, self.len(last))
        };
        Range {
            start: self.position(first, 0),
            end,
        }
    }

    /// A point of the processor's tree (from 1) as a line and a column (from 0).
    pub(crate) const fn place(point: Point) -> (usize, usize) {
        (point.line.saturating_sub(1), point.column.saturating_sub(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_place_is_a_position_in_the_encoding() {
        let source = Source::new("a😀b\r\n  ok \nend", Encoding::Utf16);
        assert_eq!(source.line(0), "a😀b", "without its \\r");
        assert_eq!(source.line(9), "");
        assert_eq!(source.position(0, 2), Position::new(0, 3));
        assert_eq!(
            source.range(0, 1, 2),
            Range::new(Position::new(0, 1), Position::new(0, 3))
        );
        assert_eq!(
            source.to_end((1, 2), 2),
            Range::new(Position::new(1, 2), Position::new(2, 3))
        );
        assert_eq!(
            Source::place(Point {
                line: 2,
                column: 3,
                offset: 0
            }),
            (1, 2)
        );
    }

    #[test]
    fn a_line_is_read_by_its_characters() {
        let source = Source::new("a😀b\n  ok \nend", Encoding::Utf8);
        assert_eq!(source.slice(0, 1, 3), "😀b");
        assert_eq!(source.char_at(1, 2), Some('o'));
        assert_eq!(source.char_at(1, 9), None);
        assert_eq!((source.len(1), source.trimmed_len(1)), (5, 4));
        assert_eq!(source.skip_spaces(1, 0), 2);
    }

    #[test]
    fn whole_lines_end_at_the_next_lines_start_or_the_texts_end() {
        let source = Source::new("one\ntwo\nthree", Encoding::Utf32);
        assert_eq!(
            source.whole_lines(LineSpan::one(0)),
            Range::new(Position::new(0, 0), Position::new(1, 0))
        );
        assert_eq!(
            source.whole_lines(LineSpan::one(2)),
            Range::new(Position::new(2, 0), Position::new(2, 5))
        );
    }
}
