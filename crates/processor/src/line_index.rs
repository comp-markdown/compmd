//! Where each line of a text starts, in bytes and in code points (audit B4, R1, A8): the one count of lines, for every
//! line ending, LF, CRLF and CR alone, as CommonMark counts them and as the fork of markdown-rs places nodes. A document
//! keeps one with its text, an edit updates it without reading the rest of the text, and the parser, the frontmatter
//! and the data readers count lines with it too.

use crate::diagnostic::Location;

/// Where each line of a text starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LineIndex(Vec<Place>);

/// A place in a text, in bytes and in code points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Place {
    pub(crate) byte: usize,
    pub(crate) offset: usize,
}

/// The start of a text.
const START: Place = Place { byte: 0, offset: 0 };

impl LineIndex {
    /// The lines of a document's `text`, after its byte order mark.
    pub(crate) fn new(text: &str) -> Self {
        Self::of(text.strip_prefix('\u{feff}').unwrap_or(text))
    }

    /// The lines of `text` as it is, a byte order mark and all.
    pub(crate) fn of(text: &str) -> Self {
        let mut starts = vec![START];
        starts.extend(line_ends(text, START));
        Self(starts)
    }

    /// How many lines the text has: one more than its line endings.
    pub(crate) fn count(&self) -> usize {
        self.0.len()
    }

    /// Where line `line`, counted from 1, starts; `None` past the last line.
    pub(crate) fn line_start(&self, line: usize) -> Option<Place> {
        self.0.get(line.checked_sub(1)?).copied()
    }

    /// The line, counted from 1, that byte `byte` is on.
    pub(crate) fn line_of_byte(&self, byte: usize) -> usize {
        self.0.partition_point(|start| start.byte <= byte)
    }

    /// Where code point `offset` of `text` (the text the index was made of) is; its end past the end.
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

    /// The code point line `line`, counted from 1, starts at; `None` past the last line.
    pub(crate) fn start(&self, line: usize) -> Option<usize> {
        self.line_start(line).map(|start| start.offset)
    }

    /// The line and column of code point `offset`; past the end of the text, a column past its last line.
    pub(crate) fn location(&self, offset: usize) -> Location {
        let line = self.0.partition_point(|start| start.offset <= offset);
        Location::new(line, offset - self.0[line - 1].offset + 1)
    }

    /// The starts after the text from `start` to `end` became `inserted`; `text` is the new text, which the index was
    /// made of. A CR next to the edit may join a line ending with the text beside it, or part one (`\r` and `\n`
    /// becoming CRLF, or the other way round): then the index is made again, which is rare.
    pub(crate) fn edit(&mut self, text: &str, start: Place, end: Place, inserted: &str) {
        let before = text[..start.byte].ends_with('\r');
        let after = text[start.byte + inserted.len()..].starts_with('\n');
        if before || (after && inserted.ends_with('\r')) {
            *self = Self::of(text);
            return;
        }
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

/// Where the lines after each line ending of `text`, which starts at `at`, start: after an LF, a CRLF, or a CR alone.
fn line_ends(text: &str, at: Place) -> Vec<Place> {
    let mut ends = Vec::new();
    let mut chars = text.char_indices().enumerate().peekable();
    while let Some((count, (index, c))) = chars.next() {
        let ends_line =
            c == '\n' || (c == '\r' && chars.peek().is_none_or(|(_, (_, next))| *next != '\n'));
        if ends_line {
            ends.push(Place {
                byte: at.byte + index + 1,
                offset: at.offset + count + 1,
            });
        }
    }
    ends
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_place_is_found_in_its_line() {
        let text = "aé\n😀b\n";
        let index = LineIndex::new(text);
        let place = |offset| index.place(text, offset);
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
    fn an_offset_has_its_line_and_column() {
        let index = LineIndex::new("\u{feff}aé\n😀b\n");
        assert_eq!(index.location(0), Location::new(1, 1));
        assert_eq!(index.location(2), Location::new(1, 3), "the line's end");
        assert_eq!(index.location(4), Location::new(2, 2));
        assert_eq!(index.location(6), Location::new(3, 1));
        assert_eq!(index.location(9), Location::new(3, 4), "past the end");
    }

    #[test]
    fn every_line_ending_ends_a_line() {
        let index = LineIndex::of("a\r\nb\rc\nd\r");
        assert_eq!(index.count(), 5);
        let starts: Vec<usize> = (1..=5).filter_map(|line| index.start(line)).collect();
        assert_eq!(starts, [0, 3, 5, 7, 9]);
        assert_eq!(index.line_of_byte(4), 2);
        assert_eq!(index.line_of_byte(9), 5);
        assert_eq!(index.line_start(6), None);
        assert_eq!(index.line_start(0), None);
    }

    #[test]
    fn an_edit_moves_the_starts_after_it() {
        for text in ["one\ntwé\nthree\n", "one\r\ntwé\rthree\r\n"] {
            let length = text.chars().count();
            for (start, end, inserted) in [
                (0, 0, "x\n"),
                (4, 8, ""),
                (5, 6, "\n😀\r\n"),
                (3, 4, " "),
                (length, length, "four\r"),
                (0, length, "all"),
                (3, 3, "\r"),
                (4, 4, "\n"),
                (4, 5, ""),
                (2, 3, "e\r"),
            ] {
                let mut index = LineIndex::new(text);
                let (from, to) = (index.place(text, start), index.place(text, end));
                let new = [&text[..from.byte], inserted, &text[to.byte..]].concat();
                index.edit(&new, from, to, inserted);
                assert_eq!(
                    index,
                    LineIndex::new(&new),
                    "{text:?} {start}..{end} {inserted:?}"
                );
            }
        }
    }
}
