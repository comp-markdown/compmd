//! What a client that lacks a primitive gets (decision 3): the element drawn another way, or that part left out,
//! never a primitive it didn't declare.

use super::*;

/// VS Code's primitives (decision 10): no `line`.
fn without_lines() -> Primitives {
    all_but(&[Primitive::Line])
}

#[test]
fn without_line_a_code_blocks_language_goes_on_its_opening_fence() {
    assert_eq!(
        drawn_above("```rust\nlet tide = 3;\n```\n", without_lines()),
        [
            "replace 0:0-0:7 \"rust\" compmd.code.language reveal 0",
            "hide 2:0-2:3 reveal 2",
        ]
    );
}

#[test]
fn without_line_or_replace_a_code_blocks_fence_marks_are_hidden_and_its_language_stays() {
    let hiding = [Primitive::Hide, Primitive::Style].into_iter().collect();
    assert_eq!(
        drawn_above("```rust\nlet tide = 3;\n```\n", hiding),
        ["hide 0:0-0:3 reveal 0", "hide 2:0-2:3 reveal 2",]
    );
}

#[test]
fn without_line_a_container_is_drawn_the_same() {
    let text = "::: box {padding=m}\nInside.\n:: #footer\nBelow.\n::: /box\n";
    assert_eq!(drawn(text, without_lines()), drawn(text, all()));
}

#[test]
fn without_prefix_a_boxs_gutter_is_inserted_on_each_line() {
    let text = "::: box\nOne.\nTwo.\n::: /box\n";
    let inserting = all_but(&[Primitive::Prefix]);
    let marks = drawn(text, inserting);
    assert_eq!(
        marks[2..4],
        [
            "insert 1:0 \"│ \" compmd.container.border",
            "insert 2:0 \"│ \" compmd.container.border",
        ]
    );
}

#[test]
fn without_line_a_rule_is_its_text_replaced_and_its_line_styled() {
    assert_eq!(
        drawn("***\n", without_lines()),
        [
            "replace 0:0-0:3 \"───\" compmd.rule reveal 0",
            "style 0:0-0:3 compmd.rule whole",
        ]
    );
}

#[test]
fn without_line_a_table_has_no_borders_and_without_replace_its_pipes_are_styled() {
    let text = "| A | B |\n| - | - |\n";
    assert_eq!(
        drawn(text, without_lines()),
        [
            "replace 0:0-0:1 \"│\" compmd.table.border reveal 0-1",
            "replace 0:4-0:5 \"│\" compmd.table.border reveal 0-1",
            "replace 0:8-0:9 \"│\" compmd.table.border reveal 0-1",
            "replace 1:0-1:9 \"├───┼───┤\" compmd.table.border reveal 0-1",
        ]
    );
    let styling = [Primitive::Style].into_iter().collect();
    assert_eq!(
        drawn(text, styling),
        [
            "style 0:0-0:1 compmd.table.border",
            "style 0:4-0:5 compmd.table.border",
            "style 0:8-0:9 compmd.table.border",
            "style 1:0-1:9 compmd.table.border",
        ]
    );
}

#[test]
fn without_replace_markers_are_styled_where_they_are() {
    let text = "## Tides\n\n- [x] ropes\n\n> [!TIP]\n> Early.\n";
    let styling = all_but(&[Primitive::Replace]);
    assert_eq!(
        drawn(text, styling),
        [
            "style 0:0-0:8 compmd.heading.2 whole",
            "style 2:0-2:1 compmd.bullet.1",
            "style 2:2-2:5 compmd.checkbox.checked",
            "style 4:0-4:1 compmd.callout.tip",
            "style 4:2-4:8 compmd.callout.tip",
            "style 5:0-5:1 compmd.callout.tip",
        ]
    );
}

#[test]
fn a_client_that_draws_nothing_gets_no_marks() {
    let text = "# A\n\n::: box\nIn.\n::: /box\n";
    assert_eq!(drawn(text, Primitives::default()), Vec::<String>::new());
}
