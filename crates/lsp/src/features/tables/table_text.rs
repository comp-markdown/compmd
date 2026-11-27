//! A table as it is written, from the processor's tree and the document's text: its rows' cells, where the tree
//! puts them (so an escaped pipe, `\|`, is a cell's text, and a pipe in a code span splits cells, as GFM reads it), the
//! delimiter row, and each column's alignment.

use compmd::tree::{Align, Node, Table};

use super::row_text::RowText;
use crate::features::drawing::Source;

/// A table's lines, the delimiter row second.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TableText {
    /// The line the header row is on.
    pub(crate) first_line: usize,
    /// The header row, the delimiter row, then the body rows, one a line.
    pub(crate) rows: Vec<RowText>,
    /// Each column's alignment, as the delimiter row has it.
    pub(crate) align: Vec<Option<Align>>,
}

/// The index of the delimiter row in [`TableText::rows`].
pub(crate) const DELIMITER: usize = 1;

impl TableText {
    /// `table` of the tree, read from `source`; `None` for a table without positions.
    pub(crate) fn read(table: &Table, source: &Source) -> Option<Self> {
        let first_line = Source::place(table.position?.start).0;
        let mut rows = Vec::with_capacity(table.children.len() + 1);
        for (index, row) in table.children.iter().enumerate() {
            let Node::TableRow(row) = row else {
                return None;
            };
            let line = Source::place(row.position?.start).0;
            if index == DELIMITER {
                rows.push(RowText::of_delimiter(source, line - 1));
            }
            rows.push(RowText::of_cells(source, line, &row.children)?);
        }
        if rows.len() == DELIMITER {
            rows.push(RowText::of_delimiter(source, first_line + DELIMITER));
        }
        Some(Self {
            first_line,
            rows,
            align: table.align.clone(),
        })
    }

    /// The line the last row is on.
    pub(crate) fn last_line(&self) -> usize {
        self.first_line + self.rows.len() - 1
    }

    /// Whether `line` is one of the table's.
    pub(crate) fn holds(&self, line: usize) -> bool {
        (self.first_line..=self.last_line()).contains(&line)
    }

    /// How many columns the delimiter row makes.
    pub(crate) fn columns(&self) -> usize {
        self.rows[DELIMITER].cells.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::tables::testing::table_of;

    #[test]
    fn a_tables_rows_are_read_from_the_tree_with_the_delimiter_second() {
        let table = table_of("Before.\n\n| a | b \\| c | `d|e` |\n|:--|--:|---|---|\n| 1 |\n");
        assert_eq!(table.first_line, 2);
        assert_eq!(table.last_line(), 4);
        assert!(table.holds(3) && !table.holds(5) && !table.holds(1));
        assert_eq!(table.columns(), 4);
        assert_eq!(table.rows[0].cells, ["a", "b \\| c", "`d", "e`"]);
        assert_eq!(table.rows[DELIMITER].cells, [":--", "--:", "---", "---"]);
        assert_eq!(table.rows[2].cells, ["1"]);
        assert_eq!(
            table.align,
            [Some(Align::Left), Some(Align::Right), None, None]
        );
    }

    #[test]
    fn a_table_of_a_header_alone_has_its_delimiter() {
        let table = table_of("a | b\n--|--\n");
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0].cells, ["a", "b"]);
        assert!(!table.rows[0].opens_with_pipe && !table.rows[0].closes_with_pipe);
        assert_eq!(table.rows[DELIMITER].cells, ["--", "--"]);
    }

    #[test]
    fn a_rows_prefix_is_what_is_before_it_on_its_line() {
        let table = table_of("> | a |\n> |---|\n> | 1 |   \n");
        assert_eq!(
            table
                .rows
                .iter()
                .map(|row| row.prefix.as_str())
                .collect::<Vec<_>>(),
            ["> ", "> ", "> "]
        );
        assert!(table.rows[2].closes_with_pipe);
        let listed = table_of("- | a |\n  |---|\n");
        assert_eq!(listed.rows[0].prefix, "- ");
        assert_eq!(listed.rows[DELIMITER].prefix, "  ");
    }
}
