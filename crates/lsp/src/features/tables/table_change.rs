//! A change to a table's rows or columns, at the cell a cursor is in ([a document drawn in the editor], decision 9):
//! a row inserted or deleted, a column inserted, deleted or moved, a column's alignment set. A column change first
//! gives every row a cell in each column, an empty one where it had none, which GFM shows the same; a cell left empty
//! at a row's end that has no pipe there gets one, so the row keeps it.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::Align;

use super::row_text::RowText;
use super::table_text::{DELIMITER, TableText};

/// The first body row's index in [`TableText::rows`].
const FIRST_BODY_ROW: usize = DELIMITER + 1;

/// A change to a table at a cell, its row and its column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TableChange {
    /// An empty row above the cell's (a body row's), or below it (the header's, too).
    NewRow { below: bool },
    /// The cell's body row deleted.
    RowRemoval,
    /// An empty column before the cell's, or after it.
    NewColumn { after: bool },
    /// The cell's column deleted, when the table has another.
    ColumnRemoval,
    /// The cell's column swapped with the one on its left, or on its right.
    ColumnMove { right: bool },
    /// The cell's column aligned so, or not at all.
    ColumnAlignment(Option<Align>),
}

/// Where a change is made: the index of a row in [`TableText::rows`] and a column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Cell {
    pub(crate) row: usize,
    pub(crate) column: usize,
}

impl TableChange {
    /// Every change, in the order they are offered.
    pub(crate) const ALL: [Self; 12] = [
        Self::NewRow { below: false },
        Self::NewRow { below: true },
        Self::RowRemoval,
        Self::NewColumn { after: false },
        Self::NewColumn { after: true },
        Self::ColumnRemoval,
        Self::ColumnMove { right: false },
        Self::ColumnMove { right: true },
        Self::ColumnAlignment(Some(Align::Left)),
        Self::ColumnAlignment(Some(Align::Center)),
        Self::ColumnAlignment(Some(Align::Right)),
        Self::ColumnAlignment(None),
    ];

    /// How the change is offered.
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::NewRow { below: false } => "Insert a row above",
            Self::NewRow { below: true } => "Insert a row below",
            Self::RowRemoval => "Delete the row",
            Self::NewColumn { after: false } => "Insert a column before",
            Self::NewColumn { after: true } => "Insert a column after",
            Self::ColumnRemoval => "Delete the column",
            Self::ColumnMove { right: false } => "Move the column left",
            Self::ColumnMove { right: true } => "Move the column right",
            Self::ColumnAlignment(Some(Align::Left)) => "Align the column left",
            Self::ColumnAlignment(Some(Align::Center)) => "Align the column center",
            Self::ColumnAlignment(Some(Align::Right)) => "Align the column right",
            Self::ColumnAlignment(None) => "Remove the column's alignment",
        }
    }

    /// Whether the change means something at `cell` of `table`: a row change on a body row (a row below the header,
    /// too), a column change in one of the delimiter row's columns, a move where there is a column to swap with, an
    /// alignment the column doesn't have.
    pub(crate) fn applies(self, table: &TableText, cell: Cell) -> bool {
        let (row, column, columns) = (cell.row, cell.column, table.columns());
        let in_body = row >= FIRST_BODY_ROW;
        match self {
            Self::NewRow { below } => in_body || (below && row == 0),
            Self::RowRemoval => in_body,
            _ if column >= columns => false,
            Self::NewColumn { .. } => true,
            Self::ColumnRemoval => columns > 1,
            Self::ColumnMove { right: false } => column > 0,
            Self::ColumnMove { right: true } => column + 1 < columns,
            Self::ColumnAlignment(align) => table.align.get(column).copied().flatten() != align,
        }
    }

    /// `table` with the change made at `cell`, where it [applies](Self::applies).
    pub(crate) fn apply(self, table: &mut TableText, cell: Cell) {
        let Cell { row, column } = cell;
        match self {
            Self::NewRow { below } => {
                let at = if below {
                    (row + 1).max(FIRST_BODY_ROW)
                } else {
                    row
                };
                let new = empty_row(&table.rows[row.max(DELIMITER)], table.columns());
                table.rows.insert(at, new);
            }
            Self::RowRemoval => {
                table.rows.remove(row);
            }
            Self::NewColumn { after } => {
                let at = column + usize::from(after);
                fill_columns(table);
                for row in &mut table.rows {
                    row.cells.insert(at, String::new());
                }
                table.align.insert(at, None);
            }
            Self::ColumnRemoval => {
                fill_columns(table);
                for row in &mut table.rows {
                    row.cells.remove(column);
                }
                table.align.remove(column);
            }
            Self::ColumnMove { right } => {
                let other = if right { column + 1 } else { column - 1 };
                fill_columns(table);
                for row in &mut table.rows {
                    row.cells.swap(column, other);
                }
                table.align.swap(column, other);
            }
            Self::ColumnAlignment(align) => table.align[column] = align,
        }
        for row in &mut table.rows {
            keep_empty_ends(row);
        }
    }
}

/// A row of `columns` empty cells, with `like`'s prefix and outer pipes.
fn empty_row(like: &RowText, columns: usize) -> RowText {
    RowText {
        prefix: like.prefix.clone(),
        cells: vec![String::new(); columns],
        starts: Vec::new(),
        opens_with_pipe: like.opens_with_pipe,
        closes_with_pipe: like.closes_with_pipe,
    }
}

/// Every row of `table` with a cell in each of the delimiter row's columns.
fn fill_columns(table: &mut TableText) {
    let columns = table.columns();
    for row in &mut table.rows {
        if row.cells.len() < columns {
            row.cells.resize(columns, String::new());
        }
    }
}

/// `row` with a pipe at an end where its cell is empty, which the row would lose without it.
fn keep_empty_ends(row: &mut RowText) {
    row.opens_with_pipe |= row.cells.first().is_some_and(String::is_empty);
    row.closes_with_pipe |= row.cells.last().is_some_and(String::is_empty);
}

#[cfg(test)]
mod tests;
