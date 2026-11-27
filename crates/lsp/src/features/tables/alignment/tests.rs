use super::*;
use crate::features::tables::testing::table_of;

/// The lines of the first table of `text`, aligned.
fn aligned(text: &str) -> Vec<String> {
    aligned_lines(&table_of(text))
}

#[test]
fn cells_are_padded_to_their_columns_widest_and_the_delimiter_keeps_its_colons() {
    assert_eq!(
        aligned("| a | bb |\n|:-|-:|\n| ccc | d |\n"),
        ["| a   |  bb |", "| :-- | --: |", "| ccc |   d |"]
    );
    assert_eq!(
        aligned("|a|b|\n|:-:|---|\n|abcdef|x|\n"),
        ["|   a    | b   |", "| :----: | --- |", "| abcdef | x   |"],
        "centred, the odd space on the right"
    );
}

#[test]
fn an_aligned_table_is_left_as_it_is() {
    let text = "| a   |  bb |\n| :-- | --: |\n| ccc |   d |\n";
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(aligned(text), lines);
}

#[test]
fn an_escaped_pipe_is_a_cells_text_and_a_code_spans_pipe_splits_cells_as_gfm_reads_them() {
    assert_eq!(
        aligned("| a \\| b | c |\n|---|---|\n"),
        ["| a \\| b | c   |", "| ------ | --- |"]
    );
    assert_eq!(
        aligned("| `x|y` |\n|---|---|\n"),
        ["| `x  | y`  |", "| --- | --- |"]
    );
}

#[test]
fn a_wide_character_takes_two_columns() {
    assert_eq!(
        aligned("| 日本 | a |\n|-|-|\n| abcde | b |\n"),
        ["| 日本  | a   |", "| ----- | --- |", "| abcde | b   |"]
    );
}

#[test]
fn a_row_keeps_its_prefix_in_a_list_a_quote_or_a_container() {
    assert_eq!(
        aligned("- Items:\n\n  | a | b |\n  |-|-|\n  | 1 | 22222 |\n"),
        [
            "  | a   | b     |",
            "  | --- | ----- |",
            "  | 1   | 22222 |"
        ]
    );
    assert_eq!(
        aligned("> | a |\n>|-|\n> | 1234 |\n"),
        ["> | a    |", ">| ---- |", "> | 1234 |"]
    );
    assert_eq!(
        aligned("::: card\n\n| a |\n|-|\n\n::: /card\n"),
        ["| a   |", "| --- |"]
    );
}

#[test]
fn a_ragged_row_keeps_the_cells_it_has() {
    assert_eq!(
        aligned("| a | b |\n|---|---|\n| 1 |\n| 1 | 2 | 3 |\n"),
        [
            "| a   | b   |",
            "| --- | --- |",
            "| 1   |",
            "| 1   | 2   | 3   |"
        ]
    );
}

#[test]
fn a_table_without_outer_pipes_keeps_none() {
    assert_eq!(
        aligned("a | b\n--:|--\nccc|d\n"),
        ["a   | b", "--: | ---", "ccc | d"],
        "a first cell without a pipe before it isn't padded on its left"
    );
}
