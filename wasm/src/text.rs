//! The editor's text as JavaScript holds it, with where each line starts, so a position in UTF-16 units, as a
//! JavaScript string counts it, becomes the code points the processor counts (spec §10.5) without counting from the
//! start of the text.

use compmd::TextEdit;

const BOM: char = '\u{feff}';

/// The text, the byte order mark included, and the start of each line.
pub(crate) struct Text {
    text: String,
    /// The first is after the byte order mark.
    starts: Vec<Place>,
}

/// A place in the text: in bytes, in UTF-16 units, and in code points after the byte order mark.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Place {
    byte: usize,
    utf16: usize,
    offset: usize,
}

impl Text {
    pub(crate) fn new(text: String) -> Self {
        let first = if text.starts_with(BOM) {
            Place {
                byte: BOM.len_utf8(),
                utf16: 1,
                offset: 0,
            }
        } else {
            Place::default()
        };
        let mut starts = vec![first];
        starts.extend(line_ends(&text[first.byte..], first));
        Self { text, starts }
    }

    /// The editor's edit of UTF-16 `start..end` to `inserted`, applied, and as the processor takes it. Positions past
    /// the end are at the end, and an end before the start is at the start. An edit that removes the byte order mark,
    /// or writes text before it or a new one, changes where the processor starts counting: it replaces the whole text.
    pub(crate) fn edit(&mut self, start: usize, end: usize, inserted: String) -> TextEdit {
        let from = self.place(start);
        let to = self.place(end.max(start));
        if from.byte < self.starts[0].byte || (from.byte == 0 && inserted.starts_with(BOM)) {
            return self.replace_all(from, to, &inserted);
        }
        self.splice_starts(from, to, &inserted);
        self.text.replace_range(from.byte..to.byte, &inserted);
        TextEdit {
            start: from.offset,
            end: to.offset,
            text: inserted,
        }
    }

    /// The code point UTF-16 position `utf16` is at, as [`place`](Self::place) finds it.
    pub(crate) fn offset(&self, utf16: usize) -> usize {
        self.place(utf16).offset
    }

    /// Where UTF-16 position `utf16` is: in its line, at the start of a character it is inside; past the end, at the
    /// end.
    fn place(&self, utf16: usize) -> Place {
        let Some(line) = self
            .starts
            .partition_point(|start| start.utf16 <= utf16)
            .checked_sub(1)
        else {
            return Place::default();
        };
        let mut place = self.starts[line];
        for c in self.text[place.byte..].chars() {
            let next = place.utf16 + c.len_utf16();
            if next > utf16 {
                break;
            }
            place = Place {
                byte: place.byte + c.len_utf8(),
                utf16: next,
                offset: place.offset + 1,
            };
        }
        place
    }

    /// The starts after `from..to` became `inserted`: those inside it go, its own lines come, and those after it move.
    fn splice_starts(&mut self, from: Place, to: Place, inserted: &str) {
        let kept = self.starts.partition_point(|start| start.byte <= from.byte);
        let moved = self.starts.partition_point(|start| start.byte <= to.byte);
        let after = self.starts.len() - moved;
        self.starts.splice(kept..moved, line_ends(inserted, from));
        let tail = self.starts.len() - after;
        let units = inserted.encode_utf16().count();
        let chars = inserted.chars().count();
        for start in &mut self.starts[tail..] {
            start.byte = start.byte + inserted.len() - (to.byte - from.byte);
            start.utf16 = start.utf16 + units - (to.utf16 - from.utf16);
            start.offset = start.offset + chars - (to.offset - from.offset);
        }
    }

    fn replace_all(&mut self, from: Place, to: Place, inserted: &str) -> TextEdit {
        let end = self.place(usize::MAX).offset;
        let mut text = std::mem::take(&mut self.text);
        text.replace_range(from.byte..to.byte, inserted);
        *self = Self::new(text);
        let counted = &self.text[self.starts[0].byte..];
        TextEdit {
            start: 0,
            end,
            text: counted.to_string(),
        }
    }
}

/// Where the lines after each line ending of `text`, which starts at `at`, start.
fn line_ends(text: &str, at: Place) -> impl Iterator<Item = Place> + '_ {
    text.char_indices()
        .scan(at, move |place, (index, c)| {
            *place = Place {
                byte: at.byte + index + c.len_utf8(),
                utf16: place.utf16 + c.len_utf16(),
                offset: place.offset + 1,
            };
            Some((c, *place))
        })
        .filter_map(|(c, place)| (c == '\n').then_some(place))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The code points of `text`'s UTF-16 positions, through a fresh index.
    fn offsets(text: &str, positions: &[usize]) -> Vec<usize> {
        let text = Text::new(text.to_string());
        positions.iter().map(|&p| text.place(p).offset).collect()
    }

    /// `edits` applied to `text`, each as UTF-16 `(start, end, inserted)`: what the processor is given for each, and
    /// the text and index after them, checked against a fresh index of the same text.
    fn edited(text: &str, edits: &[(usize, usize, &str)]) -> (Vec<(usize, usize, String)>, String) {
        let mut text = Text::new(text.to_string());
        let given = edits
            .iter()
            .map(|&(start, end, inserted)| {
                let edit = text.edit(start, end, inserted.to_string());
                (edit.start, edit.end, edit.text)
            })
            .collect();
        assert_eq!(
            text.starts,
            Text::new(text.text.clone()).starts,
            "the index after the edits"
        );
        (given, text.text)
    }

    #[test]
    fn each_character_counts_as_one_code_point() {
        // `a`, `é` as one character, `é` as `e` and a combining accent, then the line's end.
        assert_eq!(
            offsets("aé\u{65}\u{301}\n", &[0, 1, 2, 3, 4, 5, 6]),
            [0, 1, 2, 3, 4, 5, 5]
        );
        // 😀, then 👍🏽 (a thumb and a skin tone): two units each, and the position inside a pair is at its start.
        assert_eq!(
            offsets("😀👍🏽", &[0, 1, 2, 3, 4, 5, 6]),
            [0, 0, 1, 1, 2, 2, 3]
        );
        assert_eq!(
            offsets("😀 Hi", &[5]),
            [4],
            "typing at the end of the design doc's example"
        );
        assert_eq!(offsets("\ta", &[1]), [1]);
    }

    #[test]
    fn lines_are_found_by_their_starts() {
        let text = "one\r\n😀\r\nthree";
        assert_eq!(
            offsets(text, &[4, 5, 7, 8, 9, 10, 14, 99]),
            [4, 5, 6, 7, 8, 9, 13, 13]
        );
        assert_eq!(Text::new(text.to_string()).starts.len(), 3);
    }

    #[test]
    fn a_leading_byte_order_mark_isnt_counted() {
        assert_eq!(offsets("\u{feff}ab\nc", &[0, 1, 2, 4, 5]), [0, 0, 1, 3, 4]);
        let (given, text) = edited("\u{feff}ab", &[(3, 3, "!")]);
        assert_eq!(given, [(2, 2, "!".into())]);
        assert_eq!(text, "\u{feff}ab!");
    }

    #[test]
    fn edits_are_given_in_code_points_and_kept() {
        let (given, text) = edited(
            "😀 Hi\n\nTwo\n",
            &[
                (5, 5, "!"),         // after `Hi`
                (8, 8, "👍🏽\nNew\n"), // at the start of `Two`
                (0, 2, ""),          // the emoji
                (99, 50, "end"),     // past the end
            ],
        );
        assert_eq!(
            given,
            [
                (4, 4, "!".into()),
                (7, 7, "👍🏽\nNew\n".into()),
                (0, 1, String::new()),
                (17, 17, "end".into()),
            ]
        );
        assert_eq!(text, " Hi!\n\n👍🏽\nNew\nTwo\nend");
    }

    #[test]
    fn edits_across_lines_remove_their_starts() {
        let (given, text) = edited("a\r\nb\r\nc\r\n", &[(1, 7, "x\ny"), (0, 0, "\n")]);
        assert_eq!(given, [(1, 7, "x\ny".into()), (0, 0, "\n".into())]);
        assert_eq!(text, "\nax\ny\r\n");
    }

    #[test]
    fn changing_the_byte_order_mark_replaces_the_whole_text() {
        let (given, text) = edited("\u{feff}ab\nc", &[(0, 1, "")]);
        assert_eq!(given, [(0, 4, "ab\nc".into())], "removed");
        assert_eq!(text, "ab\nc");
        let (given, _) = edited("\u{feff}ab", &[(0, 0, "x")]);
        assert_eq!(given, [(0, 2, "x\u{feff}ab".into())], "text before it");
        let (given, _) = edited("ab", &[(0, 0, "\u{feff}x")]);
        assert_eq!(given, [(0, 2, "xab".into())], "a new one");
        let (given, _) = edited("ab", &[(0, 0, "x")]);
        assert_eq!(
            given,
            [(0, 0, "x".into())],
            "no byte order mark: an ordinary edit"
        );
    }
}
