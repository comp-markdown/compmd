//! The edits that make a line read as it is aligned, as small as they can be, so that the client keeps a cursor where
//! it was in the text around them: when the two lines have the same characters but spaces, one for each run of spaces
//! between two of them that changes; else one for what lies between what the lines start and end with in common (a
//! delimiter row's dashes).

use std::ops::Range;

use lsp_types::TextEdit;

use super::row_text::is_space;
use crate::features::drawing::Source;

/// The edits that make `line` of `source` read `aligned`; none when it does.
pub(crate) fn line_edits(source: &Source, line: usize, aligned: &str) -> Vec<TextEdit> {
    let written: Vec<char> = source.line(line).chars().collect();
    let aligned: Vec<char> = aligned.chars().collect();
    let (written_spaced, aligned_spaced) = (Spaced::of(&written), Spaced::of(&aligned));
    let changes: Vec<(Range<usize>, Range<usize>)> =
        if written_spaced.characters == aligned_spaced.characters {
            written_spaced
                .gaps
                .into_iter()
                .zip(aligned_spaced.gaps)
                .filter(|(from, to)| written[from.clone()] != aligned[to.clone()])
                .collect()
        } else {
            Vec::from_iter(differing(&written, &aligned))
        };
    changes
        .into_iter()
        .map(|(from, to)| TextEdit {
            range: source.range(line, from.start, from.end),
            new_text: aligned[to].iter().collect(),
        })
        .collect()
}

/// A line's characters that aren't spaces, and the runs of spaces around them: one before each, and one after the
/// last, each maybe empty.
struct Spaced {
    characters: Vec<char>,
    gaps: Vec<Range<usize>>,
}

impl Spaced {
    fn of(line: &[char]) -> Self {
        let mut spaced = Self {
            characters: Vec::new(),
            gaps: Vec::new(),
        };
        let mut start = 0;
        for (at, &c) in line.iter().enumerate() {
            if !is_space(c) {
                spaced.characters.push(c);
                spaced.gaps.push(start..at);
                start = at + 1;
            }
        }
        spaced.gaps.push(start..line.len());
        spaced
    }
}

/// The part of `written` that differs from `aligned`, and the part of `aligned` that takes its place: what lies between
/// what they start and end with in common; `None` when they are the same.
fn differing(written: &[char], aligned: &[char]) -> Option<(Range<usize>, Range<usize>)> {
    if written == aligned {
        return None;
    }
    let start = written
        .iter()
        .zip(aligned)
        .take_while(|(a, b)| a == b)
        .count();
    let shorter = written.len().min(aligned.len()) - start;
    let end = written
        .iter()
        .rev()
        .zip(aligned.iter().rev())
        .take_while(|(a, b)| a == b)
        .count()
        .min(shorter);
    Some((start..written.len() - end, start..aligned.len() - end))
}

#[cfg(test)]
mod tests {
    use lsp_types::{Position, Range as LspRange};

    use super::*;
    use crate::encoding::Encoding;

    fn edit(line: u32, from: u32, to: u32, text: &str) -> TextEdit {
        TextEdit {
            range: LspRange::new(Position::new(line, from), Position::new(line, to)),
            new_text: text.into(),
        }
    }

    #[test]
    fn only_the_runs_of_spaces_that_change_are_edited() {
        let source = Source::new("x\n| a |bb|  ", Encoding::Utf16);
        assert_eq!(
            line_edits(&source, 1, "| a  | bb |"),
            [
                edit(1, 3, 4, "  "),
                edit(1, 5, 5, " "),
                edit(1, 7, 7, " "),
                edit(1, 8, 10, "")
            ]
        );
        assert_eq!(line_edits(&source, 0, "x"), []);
    }

    #[test]
    fn a_line_whose_characters_change_is_edited_between_what_it_keeps() {
        let source = Source::new("|-|:-|", Encoding::Utf16);
        assert_eq!(
            line_edits(&source, 0, "| --- | :-- |"),
            [edit(0, 1, 5, " --- | :-- ")]
        );
        assert_eq!(
            differing(&['a', 'a'], &['a', 'a', 'a']),
            Some((2..2, 2..3)),
            "what they share at either end overlaps"
        );
        assert_eq!(differing(&['a'], &['a']), None);
    }

    #[test]
    fn positions_are_in_the_clients_encoding() {
        let source = Source::new("| 😀|", Encoding::Utf16);
        assert_eq!(line_edits(&source, 0, "| 😀 |"), [edit(0, 4, 4, " ")]);
    }
}
