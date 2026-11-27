//! `refactor.rewrite` on a table ([a document drawn in the editor], decision 9): a row inserted above or below the
//! cursor's, or deleted; a column inserted before or after the cursor's, deleted, moved left or right; the column's
//! alignment set or removed. Each leaves the table aligned. Only the changes that mean something at the cursor's cell
//! are offered: none of a row's on the delimiter row, none that inserts a row above the header or deletes it.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use lsp_types::{CodeAction, CodeActionKind, Position, Range, TextEdit};

use super::{Asked, editing};
use crate::features::drawing::Source;
use crate::features::tables::{Cell, TableChange, TableText, aligned_lines, tables};

/// The changes offered to the table whose line `range` starts on, at the cell it starts in.
pub(super) fn table_actions(range: Range, asked: &Asked) -> Vec<CodeAction> {
    let line = range.start.line as usize;
    let Some(table) = tables(asked).into_iter().find(|table| table.holds(line)) else {
        return Vec::new();
    };
    let row = line - table.first_line;
    let column = asked.offset(range.start) - asked.offset(Position::new(range.start.line, 0));
    let cell = Cell {
        row,
        column: table.rows[row].cell_at(column),
    };
    TableChange::ALL
        .into_iter()
        .filter(|change| change.applies(&table, cell))
        .map(|change| {
            let mut changed = table.clone();
            change.apply(&mut changed, cell);
            CodeAction {
                title: change.title().to_owned(),
                kind: Some(CodeActionKind::REFACTOR_REWRITE),
                edit: Some(editing(asked, vec![rewritten(&table, &changed, asked)])),
                ..CodeAction::default()
            }
        })
        .collect()
}

/// The edit that writes `changed`, aligned, in place of `table`'s lines, with the document's line ends.
fn rewritten(table: &TableText, changed: &TableText, asked: &Asked) -> TextEdit {
    let source = Source::new(asked.text, asked.encoding);
    let line_end = if asked.text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    TextEdit {
        range: source.to_end((table.first_line, 0), table.last_line()),
        new_text: aligned_lines(changed).join(line_end),
    }
}

#[cfg(test)]
mod tests;
