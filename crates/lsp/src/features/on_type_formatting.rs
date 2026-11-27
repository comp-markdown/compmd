//! On-type formatting (`textDocument/onTypeFormatting`, [a document drawn in the editor], decision 7): a `|` or a
//! newline typed in a table aligns it, as markdown-mode and org-mode align theirs. The edits change only the lines
//! whose spaces change, each run of spaces on its own, so the cursor keeps its place in the text.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use lsp_types::{Position, TextEdit};

use super::Asked;
use super::tables::alignment_edits;

/// The character whose typing aligns a table.
pub(crate) const TRIGGER: &str = "|";
/// The other characters that do.
pub(crate) const MORE_TRIGGERS: [&str; 1] = ["\n"];

/// The edits after `typed` was typed at `position` (just after it) of the document `asked` is about: the table on its
/// line aligned, or, for a newline, the table on the line it ended or the new one.
pub(crate) fn on_type_formatting(position: Position, typed: &str, asked: &Asked) -> Vec<TextEdit> {
    let line = position.line as usize;
    let lines = match typed {
        TRIGGER => line..=line,
        "\n" => line.saturating_sub(1)..=line,
        _ => return Vec::new(),
    };
    alignment_edits(asked, Some(&lines))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::testing::{asking, processed};

    fn typed(text: &str, (line, character): (u32, u32), typed: &str) -> Vec<TextEdit> {
        let at = Position::new(line, character);
        asking(text, &processed(text), &[], |asked| {
            on_type_formatting(at, typed, asked)
        })
    }

    #[test]
    fn a_pipe_typed_in_a_table_aligns_it() {
        let text = "| name | n |\n|---|---|\n| harbour |\n";
        let edits = typed(text, (2, 11), "|");
        assert_eq!(
            edits
                .iter()
                .map(|edit| edit.range.start.line)
                .collect::<Vec<_>>(),
            [0, 0, 1],
            "the header and the delimiter, not the row typed in, which is as wide as it gets"
        );
        assert_eq!(edits[0].new_text, "    ");
        assert_eq!(typed("Text | more.\n", (0, 6), "|"), []);
    }

    #[test]
    fn a_newline_typed_after_a_table_aligns_it() {
        let text = "| a |\n|-|\n";
        assert_eq!(typed(text, (2, 0), "\n").len(), 2);
        assert_eq!(
            typed(text, (0, 0), "\n").len(),
            2,
            "a table on the new line, at the document's start"
        );
        assert_eq!(typed(text, (2, 0), "x"), [], "another character");
        assert_eq!(typed("a\n\n| b |\n|-|\n", (1, 0), "\n"), []);
    }
}
