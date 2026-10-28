//! A document's lines, to find where a part of a line is: a component's name, a param's value, a path in a string.

use crate::diagnostic::Location;
use crate::tree::{Point, Position};

/// The lines of a document, without their line endings.
pub(crate) struct Lines<'a>(Vec<&'a str>);

impl<'a> Lines<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        Self(
            text.split('\n')
                .map(|line| line.strip_suffix('\r').unwrap_or(line))
                .collect(),
        )
    }

    /// The code points of the line `at` is on, from `at` to the end of the line.
    pub(crate) fn from(&self, at: Point) -> Vec<char> {
        self.0
            .get(at.line.wrapping_sub(1))
            .map(|line| line.chars().skip(at.column.saturating_sub(1)).collect())
            .unwrap_or_default()
    }

    /// The code points of `position`, across lines, each with where it is; line endings are left out.
    pub(crate) fn span(&self, position: Position) -> Vec<(char, Location)> {
        let (start, end) = (position.start, position.end);
        (start.line..=end.line)
            .flat_map(|line| {
                let first = if line == start.line { start.column } else { 1 };
                let text = self.0.get(line.wrapping_sub(1)).copied().unwrap_or("");
                text.chars()
                    .enumerate()
                    .map(move |(index, c)| (c, Location::new(line, index + 1)))
                    .skip(first - 1)
                    .take_while(move |(_, at)| line < end.line || at.column < end.column)
            })
            .collect()
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
        let lines = Lines::new("\u{feff}::: grid\r\n  ::  chip[x]\nText :badge[b]");
        assert_eq!(lines.name_at(point(1, 1)), Location::new(1, 5));
        assert_eq!(lines.name_at(point(2, 3)), Location::new(2, 7));
        assert_eq!(lines.name_at(point(3, 6)), Location::new(3, 7));
        assert_eq!(lines.from(point(9, 1)), Vec::<char>::new());
    }

    #[test]
    fn a_span_runs_across_lines() {
        let lines = Lines::new("ab [x\ny](u) z");
        let span = lines.span(Position {
            start: point(1, 4),
            end: point(2, 6),
        });
        let text: String = span.iter().map(|(c, _)| c).collect();
        assert_eq!(text, "[xy](u)");
        assert_eq!(span[2].1, Location::new(2, 1));
    }
}
