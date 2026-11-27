use super::*;
use crate::features::tables::aligned_lines;
use crate::features::tables::testing::table_of;

const TABLE: &str = "| a | b |\n|:-|-:|\n| 1 | 2 |\n| 3 |\n";

/// The lines of `TABLE` with `change` made at `row` and `column`, aligned.
fn changed(change: TableChange, row: usize, column: usize) -> Vec<String> {
    let mut table = table_of(TABLE);
    let cell = Cell { row, column };
    assert!(change.applies(&table, cell), "{change:?} applies");
    change.apply(&mut table, cell);
    aligned_lines(&table)
}

#[test]
fn a_row_is_inserted_above_or_below_a_body_row_and_below_the_header() {
    assert_eq!(
        changed(TableChange::NewRow { below: false }, 2, 0),
        [
            "| a   |   b |",
            "| :-- | --: |",
            "|     |     |",
            "| 1   |   2 |",
            "| 3   |"
        ]
    );
    assert_eq!(
        changed(TableChange::NewRow { below: true }, 2, 1)[3],
        "|     |     |"
    );
    assert_eq!(
        changed(TableChange::NewRow { below: true }, 0, 0)[2],
        "|     |     |",
        "below the header is the first body row"
    );
}

#[test]
fn a_body_row_is_deleted() {
    assert_eq!(
        changed(TableChange::RowRemoval, 3, 0),
        ["| a   |   b |", "| :-- | --: |", "| 1   |   2 |"]
    );
}

#[test]
fn a_column_is_inserted_before_or_after_and_every_row_gets_its_cells() {
    assert_eq!(
        changed(TableChange::NewColumn { after: false }, 2, 1),
        [
            "| a   |     |   b |",
            "| :-- | --- | --: |",
            "| 1   |     |   2 |",
            "| 3   |     |     |"
        ]
    );
    assert_eq!(
        changed(TableChange::NewColumn { after: true }, 0, 1)[1],
        "| :-- | --: | --- |"
    );
}

#[test]
fn a_column_is_deleted_or_moved() {
    assert_eq!(
        changed(TableChange::ColumnRemoval, 1, 0),
        ["|   b |", "| --: |", "|   2 |", "|     |"]
    );
    assert_eq!(
        changed(TableChange::ColumnMove { right: true }, 0, 0),
        [
            "|   b | a   |",
            "| --: | :-- |",
            "|   2 | 1   |",
            "|     | 3   |"
        ]
    );
    assert_eq!(
        changed(TableChange::ColumnMove { right: false }, 2, 1),
        changed(TableChange::ColumnMove { right: true }, 2, 0)
    );
}

#[test]
fn a_columns_alignment_is_set_or_removed() {
    assert_eq!(
        changed(TableChange::ColumnAlignment(Some(Align::Center)), 2, 0)[1],
        "| :-: | --: |"
    );
    assert_eq!(
        changed(TableChange::ColumnAlignment(None), 2, 1)[1],
        "| :-- | --- |"
    );
}

#[test]
fn a_change_is_offered_only_where_it_means_something() {
    let table = table_of(TABLE);
    let at = |row, column| Cell { row, column };
    let offered = |cell| {
        TableChange::ALL
            .into_iter()
            .filter(|change| change.applies(&table, cell))
            .map(TableChange::title)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        offered(at(0, 0)),
        [
            "Insert a row below",
            "Insert a column before",
            "Insert a column after",
            "Delete the column",
            "Move the column right",
            "Align the column center",
            "Align the column right",
            "Remove the column's alignment"
        ]
    );
    assert!(!offered(at(1, 1)).iter().any(|title| title.contains("row")));
    assert_eq!(
        offered(at(3, 0)).len(),
        10,
        "no move left, nor its own alignment"
    );
    assert!(
        offered(at(2, 2)).iter().all(|title| title.contains("row")),
        "past the columns"
    );
    let single = table_of("| a |\n|---|\n");
    assert!(!TableChange::ColumnRemoval.applies(&single, at(0, 0)));
}

#[test]
fn an_empty_cell_at_a_rows_end_keeps_its_pipe() {
    let mut table = table_of("a | b\n--|--\n");
    let cell = Cell { row: 0, column: 0 };
    TableChange::NewColumn { after: false }.apply(&mut table, cell);
    assert_eq!(
        aligned_lines(&table),
        ["|     | a   | b", "| --- | --- | ---"]
    );
    TableChange::NewRow { below: true }.apply(&mut table, cell);
    assert_eq!(aligned_lines(&table)[2], "|     |     |     |");
}
