//! A document's lines, to find where a part of a line is: a component's name, a param's value, a path in a string.

use crate::diagnostic::Location;
use crate::parse::params::interpolations;
use crate::tree::{Point, Position};

/// Where each line of a text starts, in bytes and in code points after its byte order mark. A document keeps them with
/// its text, and an edit updates them without reading the rest of the text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LineStarts(Vec<Place>);

/// A place in a text, in bytes and in code points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Place {
    pub(crate) byte: usize,
    pub(crate) offset: usize,
}

impl LineStarts {
    pub(crate) fn new(text: &str) -> Self {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut starts = vec![Place { byte: 0, offset: 0 }];
        starts.extend(line_ends(text, Place { byte: 0, offset: 0 }));
        Self(starts)
    }

    /// Where code point `offset` of `text`, the text after its byte order mark, is; its end past the end.
    pub(crate) fn place(&self, text: &str, offset: usize) -> Place {
        let line = self.0[self.0.partition_point(|start| start.offset <= offset) - 1];
        text[line.byte..]
            .char_indices()
            .nth(offset - line.offset)
            .map_or_else(
                || Place {
                    byte: text.len(),
                    offset: line.offset + text[line.byte..].chars().count(),
                },
                |(index, _)| Place {
                    byte: line.byte + index,
                    offset,
                },
            )
    }

    /// The starts after the text from `start` to `end` became `inserted`; `text` is the new text, after its byte order
    /// mark.
    pub(crate) fn edit(&mut self, start: Place, end: Place, inserted: &str) {
        let kept = self.0.partition_point(|line| line.byte <= start.byte);
        let moved = self.0.partition_point(|line| line.byte <= end.byte);
        let count = self.0.len() - moved;
        self.0.splice(kept..moved, line_ends(inserted, start));
        let tail = self.0.len() - count;
        let added = inserted.chars().count();
        for line in &mut self.0[tail..] {
            line.byte = line.byte + inserted.len() - (end.byte - start.byte);
            line.offset = line.offset + added - (end.offset - start.offset);
        }
    }
}

/// Where the lines after each line ending of `text`, which starts at `at`, start.
fn line_ends(text: &str, at: Place) -> impl Iterator<Item = Place> + '_ {
    text.char_indices()
        .enumerate()
        .filter(|(_, (_, c))| *c == '\n')
        .map(move |(count, (index, _))| Place {
            byte: at.byte + index + 1,
            offset: at.offset + count + 1,
        })
}

/// The lines of a document, without their line endings.
pub(crate) struct Lines<'a> {
    text: &'a str,
    starts: &'a LineStarts,
}

impl<'a> Lines<'a> {
    /// The lines of `text`, which start at `starts`.
    pub(crate) fn new(text: &'a str, starts: &'a LineStarts) -> Self {
        Self {
            text: text.strip_prefix('\u{feff}').unwrap_or(text),
            starts,
        }
    }

    /// Line `number`, counted from 1.
    fn line(&self, number: usize) -> Option<&'a str> {
        let index = number.checked_sub(1)?;
        let start = self.starts.0.get(index)?.byte;
        let end = self
            .starts
            .0
            .get(index + 1)
            .map_or(self.text.len(), |next| next.byte - 1);
        let line = &self.text[start..end];
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
        let starts = LineStarts::new(text);
        let lines = Lines::new(text, &starts);
        assert_eq!(lines.name_at(point(1, 1)), Location::new(1, 5));
        assert_eq!(lines.name_at(point(2, 3)), Location::new(2, 7));
        assert_eq!(lines.name_at(point(3, 6)), Location::new(3, 7));
        assert_eq!(lines.from(point(9, 1)), Vec::<char>::new());
        assert_eq!(lines.from(point(0, 1)), Vec::<char>::new());
    }

    #[test]
    fn paths_are_past_their_braces() {
        let text = "x {t=\"a {{ b }} {{c}}\"}\nnone";
        let starts = LineStarts::new(text);
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
        let starts = LineStarts::new(text);
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
    fn a_place_is_found_in_its_line() {
        let text = "aé\n😀b\n";
        let starts = LineStarts::new(text);
        let place = |offset| starts.place(text, offset);
        assert_eq!(place(1), Place { byte: 1, offset: 1 });
        assert_eq!(place(3), Place { byte: 4, offset: 3 });
        assert_eq!(place(4), Place { byte: 8, offset: 4 });
        assert_eq!(
            place(6),
            Place {
                byte: 10,
                offset: 6
            },
            "the end"
        );
        assert_eq!(
            place(99),
            Place {
                byte: 10,
                offset: 6
            },
            "past the end"
        );
    }

    #[test]
    fn an_edit_moves_the_starts_after_it() {
        let text = "one\ntwé\nthree\n";
        for (start, end, inserted) in [
            (0, 0, "x\n"),
            (4, 8, ""),
            (5, 6, "\n😀\n"),
            (3, 4, " "),
            (14, 14, "four\n"),
            (0, 14, "all"),
        ] {
            let mut starts = LineStarts::new(text);
            let (from, to) = (starts.place(text, start), starts.place(text, end));
            let new = [&text[..from.byte], inserted, &text[to.byte..]].concat();
            starts.edit(from, to, inserted);
            assert_eq!(starts, LineStarts::new(&new), "{start}..{end} {inserted:?}");
        }
    }
}
