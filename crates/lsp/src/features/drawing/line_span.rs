//! A run of a document's lines, from 0 as LSP counts them: the lines a mark gives way on, or a prefix's lines.

use lsp_types::Range;
use serde::Serialize;

/// The lines `first` to `last`, both included.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LineSpan {
    pub(crate) first: u32,
    pub(crate) last: u32,
}

/// `line` as LSP's.
pub(crate) fn lsp_line(line: usize) -> u32 {
    u32::try_from(line).unwrap_or(u32::MAX)
}

impl LineSpan {
    /// The lines `first` to `last`.
    pub(crate) fn of(first: usize, last: usize) -> Self {
        Self {
            first: lsp_line(first),
            last: lsp_line(last.max(first)),
        }
    }

    /// The line `line` alone.
    pub(crate) fn one(line: usize) -> Self {
        Self::of(line, line)
    }

    /// The lines `range` is on: a range that ends at the start of a later line ends on the line before.
    pub(crate) fn of_range(range: &Range) -> Self {
        let last = if range.end.character == 0 && range.end.line > range.start.line {
            range.end.line - 1
        } else {
            range.end.line
        };
        Self {
            first: range.start.line,
            last: last.max(range.start.line),
        }
    }

    /// Whether `line` is one of them.
    pub(crate) const fn contains(self, line: u32) -> bool {
        self.first <= line && line <= self.last
    }

    /// Whether the two share a line.
    pub(crate) const fn meets(self, other: Self) -> bool {
        self.first <= other.last && other.first <= self.last
    }
}

#[cfg(test)]
mod tests {
    use lsp_types::Position;

    use super::*;

    #[test]
    fn a_span_holds_its_lines_and_meets_those_it_shares_one_with() {
        let span = LineSpan::of(2, 4);
        assert!(span.contains(2) && span.contains(4) && !span.contains(5));
        assert!(span.meets(LineSpan::one(4)) && !span.meets(LineSpan::one(1)));
        assert_eq!(LineSpan::of(3, 1), LineSpan::one(3), "never backwards");
        assert_eq!(lsp_line(usize::MAX), u32::MAX);
    }

    #[test]
    fn a_range_ending_at_a_lines_start_ends_on_the_line_before() {
        let range = |start: (u32, u32), end: (u32, u32)| Range {
            start: Position::new(start.0, start.1),
            end: Position::new(end.0, end.1),
        };
        assert_eq!(
            LineSpan::of_range(&range((1, 0), (3, 0))),
            LineSpan::of(1, 2)
        );
        assert_eq!(
            LineSpan::of_range(&range((1, 0), (3, 2))),
            LineSpan::of(1, 3)
        );
        assert_eq!(LineSpan::of_range(&range((1, 0), (1, 0))), LineSpan::one(1));
    }
}
