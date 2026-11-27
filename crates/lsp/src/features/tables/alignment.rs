//! A table aligned ([a document drawn in the editor], decision 7): each cell padded with spaces to its column's
//! width, as wide as the column's widest cell on screen (an East Asian wide character takes two columns), and at least
//! three; the delimiter row's dashes to the width, its colons kept; the pipes lined up. A cell's text, a row's prefix
//! and its outer pipes stay as they are, and a row keeps the cells it has: a missing cell, which GFM shows empty, is
//! not added, since that would change more than spaces. A first cell without a pipe before it is never padded on its
//! left, so its row never starts further in than it was written.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::Align;
use unicode_width::UnicodeWidthStr;

use super::row_text::RowText;
use super::table_text::{DELIMITER, TableText};

/// The narrowest a column is: a delimiter cell's `:-:`.
const NARROWEST: usize = 3;

/// The lines of `table` aligned, each with its prefix.
pub(crate) fn aligned_lines(table: &TableText) -> Vec<String> {
    let widths = widths(table);
    table
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let cells = row.cells.iter().enumerate().map(|(column, cell)| {
                let (width, align) = (widths[column], align_of(table, column));
                if index == DELIMITER {
                    delimiter_cell(width, align)
                } else if column == 0 && !row.opens_with_pipe {
                    padded(cell, width, None)
                } else {
                    padded(cell, width, align)
                }
            });
            aligned_row(row, cells)
        })
        .collect()
}

/// The display width of each column: its widest cell's, the delimiter row's left out.
fn widths(table: &TableText) -> Vec<usize> {
    let columns = table
        .rows
        .iter()
        .map(|row| row.cells.len())
        .max()
        .unwrap_or(0);
    let mut widths = vec![NARROWEST; columns];
    for (_, row) in table
        .rows
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != DELIMITER)
    {
        for (width, cell) in widths.iter_mut().zip(&row.cells) {
            *width = (*width).max(cell.width());
        }
    }
    widths
}

/// The alignment of `column`; none past the delimiter row's columns.
fn align_of(table: &TableText, column: usize) -> Option<Align> {
    table.align.get(column).copied().flatten()
}

/// `row`'s line from its prefix and its `cells`, already padded: between pipes with a space on either side, its outer
/// pipes as it has them, and no spaces at its end.
fn aligned_row(row: &RowText, cells: impl Iterator<Item = String>) -> String {
    let mut line = row.prefix.clone();
    if row.opens_with_pipe {
        line.push_str("| ");
    }
    line.push_str(&cells.collect::<Vec<_>>().join(" | "));
    if row.closes_with_pipe {
        line.push_str(" |");
    }
    let end = line.trim_end().len().max(row.prefix.len());
    line.truncate(end);
    line
}

/// `text` padded with spaces to `width` on screen: on its left when the column is aligned right, on both sides when
/// centred (the odd space on the right), else on its right.
fn padded(text: &str, width: usize, align: Option<Align>) -> String {
    let room = width.saturating_sub(text.width());
    let (left, right) = match align {
        Some(Align::Right) => (room, 0),
        Some(Align::Center) => (room / 2, room - room / 2),
        Some(Align::Left) | None => (0, room),
    };
    format!("{}{text}{}", " ".repeat(left), " ".repeat(right))
}

/// A delimiter cell `width` wide, with the colons `align` has.
fn delimiter_cell(width: usize, align: Option<Align>) -> String {
    let (left, right) = match align {
        Some(Align::Left) => (":", ""),
        Some(Align::Right) => ("", ":"),
        Some(Align::Center) => (":", ":"),
        None => ("", ""),
    };
    let dashes = width - left.len() - right.len();
    format!("{left}{}{right}", "-".repeat(dashes))
}

#[cfg(test)]
mod tests;
