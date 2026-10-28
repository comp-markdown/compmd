//! Positions in code points (spec §10.5) for text the fork of markdown-rs parsed, whose positions count bytes.

use crate::tree::{Point, Position};

/// Text being parsed, and where it starts in its document: a fallback or a definition's body is parsed on its own
/// and placed where it is written.
pub(crate) struct Source<'a> {
    text: &'a str,
    origin: Point,
    /// The byte index where each line starts.
    line_starts: Vec<usize>,
    /// The code points before each line.
    line_offsets: Vec<usize>,
}

impl<'a> Source<'a> {
    /// `text` placed at `origin`: its first line starts at `origin`, later lines at column 1.
    pub(crate) fn new(text: &'a str, origin: Point) -> Self {
        let mut line_starts = vec![0];
        let mut line_offsets = vec![0];
        let mut chars = text.char_indices().enumerate().peekable();
        while let Some((offset, (index, c))) = chars.next() {
            let crlf = c == '\r' && chars.peek().is_some_and(|(_, (_, next))| *next == '\n');
            if crlf {
                chars.next();
            }
            if c == '\n' || c == '\r' {
                line_starts.push(index + if crlf { 2 } else { 1 });
                line_offsets.push(offset + if crlf { 2 } else { 1 });
            }
        }
        Self {
            text,
            origin,
            line_starts,
            line_offsets,
        }
    }

    /// The text.
    pub(crate) fn text(&self) -> &'a str {
        self.text
    }

    /// The point at byte `index` of the text.
    pub(crate) fn point(&self, index: usize) -> Point {
        let line = self.line_starts.partition_point(|start| *start <= index) - 1;
        let line_start = self.line_starts[line];
        let column = self.text[line_start..index].chars().count();
        Point {
            line: self.origin.line + line,
            column: if line == 0 {
                self.origin.column + column
            } else {
                1 + column
            },
            offset: self.origin.offset + self.line_offsets[line] + column,
        }
    }

    /// The position of a node the fork made.
    pub(crate) fn position(
        &self,
        position: Option<&markdown::unist::Position>,
    ) -> Option<Position> {
        position.map(|position| Position {
            start: self.point(position.start.offset),
            end: self.point(position.end.offset),
        })
    }

    /// Line `line` of the document, without its line ending, and the point where it starts; `None` if the text
    /// doesn't hold it.
    pub(crate) fn line(&self, line: usize) -> Option<(&'a str, Point)> {
        let index = line.checked_sub(self.origin.line)?;
        let start = *self.line_starts.get(index)?;
        let end = self
            .line_starts
            .get(index + 1)
            .copied()
            .unwrap_or(self.text.len());
        let text = self.text[start..end].trim_end_matches(['\n', '\r']);
        Some((text, self.point(start)))
    }
}

/// The point `chars` code points further on the same line.
pub(crate) fn right(point: Point, chars: usize) -> Point {
    Point {
        column: point.column + chars,
        offset: point.offset + chars,
        ..point
    }
}

/// The start of a document.
pub(crate) const START: Point = Point {
    line: 1,
    column: 1,
    offset: 0,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn at(line: usize, column: usize, offset: usize) -> Point {
        Point {
            line,
            column,
            offset,
        }
    }

    #[test]
    fn counts_code_points_on_every_kind_of_line_ending() {
        let source = Source::new("aé\r\nb\rc\nd", START);
        assert_eq!(source.point(3), at(1, 3, 2));
        assert_eq!(source.point(5), at(2, 1, 4));
        assert_eq!(source.point(7), at(3, 1, 6));
        assert_eq!(source.point(9), at(4, 1, 8));
    }

    #[test]
    fn places_text_at_its_origin() {
        let source = Source::new("ab\ncd", at(4, 7, 30));
        assert_eq!(source.point(1), at(4, 8, 31));
        assert_eq!(source.point(4), at(5, 2, 34));
    }

    #[test]
    fn gives_a_line_and_where_it_starts() {
        let source = Source::new("ab\r\ncd\n", at(3, 1, 10));
        assert_eq!(source.line(4), Some(("cd", at(4, 1, 14))));
        assert_eq!(source.line(2), None);
        assert_eq!(source.line(9), None);
    }

    #[test]
    fn moves_right_on_a_line() {
        assert_eq!(right(at(2, 3, 9), 4), at(2, 7, 13));
    }
}
