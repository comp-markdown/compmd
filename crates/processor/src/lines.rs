//! A document's lines, to find where a part of a line is: a component's name, a param's value, a path in a string.

use crate::diagnostic::Location;
use crate::line_index::LineIndex;
use crate::parse::params::interpolations;
use crate::tree::{Point, Position};

/// The lines of a document, without their line endings.
pub(crate) struct Lines<'a> {
    text: &'a str,
    starts: &'a LineIndex,
}

impl<'a> Lines<'a> {
    /// The lines of `text`, which start at `starts`.
    pub(crate) fn new(text: &'a str, starts: &'a LineIndex) -> Self {
        Self {
            text: text.strip_prefix('\u{feff}').unwrap_or(text),
            starts,
        }
    }

    /// Line `number`, counted from 1, without its line ending: LF, CRLF or CR.
    pub(crate) fn line(&self, number: usize) -> Option<&'a str> {
        let start = self.starts.line_start(number)?.byte;
        let end = self
            .starts
            .line_start(number + 1)
            .map_or(self.text.len(), |next| next.byte);
        let line = &self.text[start..end];
        let line = line.strip_suffix('\n').unwrap_or(line);
        Some(line.strip_suffix('\r').unwrap_or(line))
    }

    /// The code points of the line `at` is on, from `at` to the end of the line.
    pub(crate) fn from(&self, at: Point) -> Vec<char> {
        self.line(at.line)
            .map(|line| line.chars().skip(at.column.saturating_sub(1)).collect())
            .unwrap_or_default()
    }

    /// The code points of `position`, across lines, each with where it is; line endings are left out.
    pub(crate) fn span(&self, position: Position) -> Vec<(char, Location)> {
        let (start, end) = (position.start, position.end);
        (start.line..=end.line)
            .flat_map(|line| {
                let first = if line == start.line { start.column } else { 1 };
                let text = self.line(line).unwrap_or("");
                text.chars()
                    .enumerate()
                    .map(move |(index, c)| (c, Location::new(line, index + 1)))
                    .skip(first - 1)
                    .take_while(move |(_, at)| line < end.line || at.column < end.column)
            })
            .collect()
    }

    /// Where the path of each interpolation is, from `start` to the end of its line, in order.
    pub(crate) fn paths_from(&self, start: Point) -> Vec<Location> {
        interpolations(&self.from(start))
            .into_iter()
            .map(|(index, _)| Location::new(start.line, start.column + index))
            .collect()
    }

    /// Where the path of the interpolation that starts at `start` is; `start` when there is none there.
    pub(crate) fn path_at(&self, start: Point) -> Location {
        self.paths_from(start)
            .first()
            .copied()
            .unwrap_or_else(|| start.into())
    }

    /// Where the name of the component line or inline component that starts at `start` is: past its colons and
    /// the white space after them.
    pub(crate) fn name_at(&self, start: Point) -> Location {
        let rest = self.from(start);
        let colons = rest.iter().take_while(|c| **c == ':').count();
        let spaces = rest[colons..]
            .iter()
            .take_while(|c| c.is_whitespace())
            .count();
        Location::new(start.line, start.column + colons + spaces)
    }

    /// Where the language of the fenced code block that starts at `start` is: past its fence and the white space
    /// after it.
    pub(crate) fn language_at(&self, start: Point) -> Location {
        let rest = self.from(start);
        let fence = rest.iter().take_while(|c| matches!(c, '`' | '~')).count();
        let spaces = rest[fence..]
            .iter()
            .take_while(|c| c.is_whitespace())
            .count();
        Location::new(start.line, start.column + fence + spaces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(line: usize, column: usize) -> Point {
        Point {
            line,
            column,
            offset: 0,
        }
    }

    #[test]
    fn names_follow_the_colons_and_white_space() {
        let text = "\u{feff}::: grid\r\n  ::  chip[x]\nText :badge[b]";
        let starts = LineIndex::new(text);
        let lines = Lines::new(text, &starts);
        assert_eq!(lines.name_at(point(1, 1)), Location::new(1, 5));
        assert_eq!(lines.name_at(point(2, 3)), Location::new(2, 7));
        assert_eq!(lines.name_at(point(3, 6)), Location::new(3, 7));
        assert_eq!(lines.from(point(9, 1)), Vec::<char>::new());
        assert_eq!(lines.from(point(0, 1)), Vec::<char>::new());
    }

    #[test]
    fn a_code_blocks_language_follows_its_fence() {
        let text = "```mermaid\n  ~~~~ dot {a=1}";
        let starts = LineIndex::new(text);
        let lines = Lines::new(text, &starts);
        assert_eq!(lines.language_at(point(1, 1)), Location::new(1, 4));
        assert_eq!(lines.language_at(point(2, 3)), Location::new(2, 8));
    }

    #[test]
    fn paths_are_past_their_braces() {
        let text = "x {t=\"a {{ b }} {{c}}\"}\nnone";
        let starts = LineIndex::new(text);
        let lines = Lines::new(text, &starts);
        assert_eq!(
            lines.paths_from(point(1, 6)),
            [Location::new(1, 12), Location::new(1, 19)]
        );
        assert_eq!(lines.path_at(point(1, 9)), Location::new(1, 12));
        assert_eq!(lines.path_at(point(2, 1)), Location::new(2, 1));
    }

    #[test]
    fn a_span_runs_across_lines() {
        let text = "ab [x\ny](u) z";
        let starts = LineIndex::new(text);
        let lines = Lines::new(text, &starts);
        let span = lines.span(Position {
            start: point(1, 4),
            end: point(2, 6),
        });
        let text: String = span.iter().map(|(c, _)| c).collect();
        assert_eq!(text, "[xy](u)");
        assert_eq!(span[2].1, Location::new(2, 1));
    }

    #[test]
    fn a_line_is_without_its_line_ending_of_any_kind() {
        let text = "a\r\nb\rc\nd";
        let starts = LineIndex::new(text);
        let lines = Lines::new(text, &starts);
        let all: Vec<Option<&str>> = (1..=5).map(|line| lines.line(line)).collect();
        assert_eq!(all, [Some("a"), Some("b"), Some("c"), Some("d"), None]);
        assert_eq!(lines.name_at(point(3, 1)), Location::new(3, 1));
    }
}
