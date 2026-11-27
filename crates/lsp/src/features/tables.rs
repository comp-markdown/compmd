//! GFM tables aligned in the text ([a document drawn in the editor], decisions 7 and 9): read from the processor's
//! tree, padded to their columns' widths, and changed by row and column. Formatting, on-type formatting and the table
//! code actions are made of this, and so is the formatter of the whole document, which aligns its tables with it.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

mod alignment;
mod line_edits;
mod row_text;
mod table_change;
mod table_text;
#[cfg(test)]
mod testing;

pub(crate) use alignment::aligned_lines;
pub(crate) use table_change::{Cell, TableChange};
pub(crate) use table_text::TableText;

use std::ops::RangeInclusive;

use compmd::tree::Node;
use lsp_types::TextEdit;

use super::Asked;
use super::drawing::Source;
use line_edits::line_edits;

/// Every table of `nodes` and what they hold, read from `source`, in the document's order; a component definition's
/// body, which isn't the document's text, is left out.
pub(crate) fn tables_of(nodes: &[Node], source: &Source) -> Vec<TableText> {
    let mut tables = Vec::new();
    collect(nodes, source, &mut tables);
    tables
}

fn collect(nodes: &[Node], source: &Source, tables: &mut Vec<TableText>) {
    for node in nodes {
        match node {
            Node::Table(table) => tables.extend(TableText::read(table, source)),
            Node::ComponentDefinition(_) | Node::ComponentSignature(_) => {}
            Node::ComponentContainer(container) => {
                collect(&container.children, source, tables);
                for slot in &container.slots {
                    collect(&slot.children, source, tables);
                }
            }
            _ => collect(node.children(), source, tables),
        }
    }
}

/// The tables of the document `asked` is about, read from its processor's tree; none when it couldn't be processed.
pub(crate) fn tables(asked: &Asked) -> Vec<TableText> {
    let source = Source::new(asked.text, asked.encoding);
    asked
        .processor
        .map(|processor| tables_of(&processor.tree().children, &source))
        .unwrap_or_default()
}

/// The edits that align the tables of the document `asked` is about that have a line in `lines`, or every table when
/// `None`: only the lines that change, each edited as little as it can be. Only spaces change, and a delimiter row's
/// dashes.
pub(crate) fn alignment_edits(
    asked: &Asked,
    lines: Option<&RangeInclusive<usize>>,
) -> Vec<TextEdit> {
    let source = Source::new(asked.text, asked.encoding);
    let meets = |table: &TableText| {
        lines.is_none_or(|lines| {
            table.first_line <= *lines.end() && *lines.start() <= table.last_line()
        })
    };
    tables(asked)
        .iter()
        .filter(|table| meets(table))
        .flat_map(|table| {
            aligned_lines(table)
                .iter()
                .enumerate()
                .flat_map(|(index, line)| line_edits(&source, table.first_line + index, line))
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests;
