//! Formatting (`textDocument/formatting` and `rangeFormatting`): for now, the document's tables aligned
//! ([a document drawn in the editor], decision 7), those the range meets when there is one. The formatter of the whole
//! document (the language server's design, *Formatting*) comes with its own rules.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use lsp_types::{Range, TextEdit};

use super::Asked;
use super::tables::alignment_edits;

/// The edits that format the document `asked` is about: all of it, or the lines of `range`.
pub(crate) fn formatting(range: Option<Range>, asked: &Asked) -> Vec<TextEdit> {
    let lines = range.map(|range| range.start.line as usize..=range.end.line as usize);
    alignment_edits(asked, lines.as_ref())
}

#[cfg(test)]
mod tests {
    use lsp_types::Position;

    use super::*;
    use crate::features::testing::{asking, processed};

    fn formatted(text: &str, range: Option<Range>) -> Vec<TextEdit> {
        asking(text, &processed(text), &[], |asked| {
            formatting(range, asked)
        })
    }

    #[test]
    fn the_document_has_its_tables_aligned_or_those_its_range_meets() {
        let text = "| a | bb |\n|-|-|\n\nText.\n\n| c |\n|-|\n";
        let all = formatted(text, None);
        assert_eq!(
            all.iter()
                .map(|edit| edit.range.start.line)
                .collect::<Vec<_>>(),
            [0, 0, 1, 5, 6],
            "only the lines that change"
        );
        let lines = |first, last| Some(Range::new(Position::new(first, 0), Position::new(last, 0)));
        assert_eq!(formatted(text, lines(5, 5)).len(), 2);
        assert_eq!(formatted(text, lines(2, 4)), []);
        assert_eq!(formatted(text, lines(1, 6)).len(), all.len());
        assert_eq!(formatted("No table.\n", None), []);
    }
}
