//! Markdown's elements, with every primitive (decision 11).

use super::*;

#[test]
fn an_atx_headings_marker_is_its_levels_glyph_and_its_line_its_style() {
    assert_eq!(
        drawn("## Harbour notes\n", all()),
        [
            "replace 0:0-0:2 \"H2\" compmd.heading.marker reveal 0",
            "style 0:0-0:16 compmd.heading.2 whole",
        ]
    );
}

#[test]
fn a_setext_heading_has_its_glyph_inserted_and_its_underline_hidden() {
    assert_eq!(
        drawn("Harbour notes\n===\n\nTides\n---\n", all()),
        [
            "insert 0:0 \"H1 \" compmd.heading.marker reveal 0-1",
            "style 0:0-1:3 compmd.heading.1 whole",
            "hide 1:0-1:3 reveal 0-1",
            "insert 3:0 \"H2 \" compmd.heading.marker reveal 3-4",
            "style 3:0-4:3 compmd.heading.2 whole",
            "hide 4:0-4:3 reveal 3-4",
        ]
    );
}

#[test]
fn a_fenced_code_block_is_where_it_is_its_fence_marks_set_back_its_language_coloured_its_lines_left_alone()
 {
    assert_eq!(
        drawn("```rust {title=x}\nlet tide = 3;\n  ```  \n", all()),
        [
            "style 0:0-0:3 compmd.code.fence",
            "style 0:3-0:7 compmd.code.language",
            "style 2:2-2:5 compmd.code.fence",
        ]
    );
    assert_eq!(
        drawn("~~~~\nplain\n", all()),
        ["style 0:0-0:4 compmd.code.fence"],
        "no language, no closing fence"
    );
}

#[test]
fn a_fenced_code_blocks_fences_are_hidden_and_its_language_is_on_a_line_above() {
    assert_eq!(
        drawn_above("```rust\nlet tide = 3;\n```\n", all()),
        [
            "hide 0:0-0:7 reveal 0",
            "line above 0 [\"rust\" compmd.code.language] reveal 0",
            "hide 2:0-2:3 reveal 2",
        ]
    );
}

#[test]
fn a_code_block_without_a_language_or_a_closing_fence_hides_what_it_has() {
    assert_eq!(
        drawn_above("~~~~\nplain\n", all()),
        ["hide 0:0-0:4 reveal 0"]
    );
}

#[test]
fn an_indented_code_block_is_left_as_written() {
    assert_eq!(drawn("    let tide = 3;\n", all()), Vec::<String>::new());
}

#[test]
fn inline_code_is_styled_and_a_rule_is_a_line_in_its_place() {
    assert_eq!(
        drawn("Run `make`.\n\n---\n", all()),
        [
            "style 0:4-0:10 compmd.code.inline",
            "hide 2:0-3:0 reveal 2",
            "line above 2 [] fill \"─\" compmd.rule reveal 2",
        ]
    );
}

#[test]
fn a_rule_further_in_starts_its_line_with_the_indent() {
    assert_eq!(
        drawn("- a\n\n  ***\n", all())[1..],
        [
            "line above 2 [\"  \" compmd.rule] fill \"─\" compmd.rule reveal 2",
            "hide 2:2-2:5 reveal 2",
        ]
    );
}

#[test]
fn bullets_are_glyphs_by_depth_and_checkboxes_by_state() {
    assert_eq!(
        drawn(
            "- [ ] ropes\n  * [x] sails\n    + tea\n      - cups\n        - mugs\n1. [ ] one\n",
            all()
        ),
        [
            "replace 0:0-0:1 \"●\" compmd.bullet.1 reveal 0",
            "replace 0:2-0:5 \"☐\" compmd.checkbox.unchecked reveal 0",
            "replace 1:2-1:3 \"○\" compmd.bullet.2 reveal 1",
            "replace 1:4-1:7 \"☑\" compmd.checkbox.checked reveal 1",
            "replace 2:4-2:5 \"◆\" compmd.bullet.3 reveal 2",
            "replace 3:6-3:7 \"◇\" compmd.bullet.4 reveal 3",
            "replace 4:8-4:9 \"●\" compmd.bullet.1 reveal 4",
            "replace 5:3-5:6 \"☐\" compmd.checkbox.unchecked reveal 5",
        ]
    );
}

#[test]
fn a_quotes_bars_are_by_depth_and_an_alerts_title_is_its_icon_and_name() {
    assert_eq!(
        drawn(
            "> [!WARNING]\n> High tide at six.\n>\n> > Inner\nlazy\n",
            all()
        ),
        [
            "replace 0:0-0:1 \"▋\" compmd.callout.warning reveal 0",
            "replace 0:2-0:12 \"⚠ Warning\" compmd.callout.warning reveal 0",
            "replace 1:0-1:1 \"▋\" compmd.callout.warning reveal 1",
            "replace 2:0-2:1 \"▋\" compmd.callout.warning reveal 2",
            "replace 3:0-3:1 \"▋\" compmd.callout.warning reveal 3",
            "replace 3:2-3:3 \"▋\" compmd.quote.2 reveal 3",
        ]
    );
}

#[test]
fn a_table_is_drawn_over_its_aligned_text() {
    assert_eq!(
        drawn(
            "| Boat | Crew |\n| ---- | ---: |\n| A\\|B |    3 |\n",
            all()
        ),
        [
            "line above 0 [\"┌──────┬──────┐\" compmd.table.border] reveal 0-2",
            "replace 0:0-0:1 \"│\" compmd.table.border reveal 0-2",
            "replace 0:7-0:8 \"│\" compmd.table.border reveal 0-2",
            "replace 0:14-0:15 \"│\" compmd.table.border reveal 0-2",
            "replace 1:0-1:15 \"├──────┼──────┤\" compmd.table.border reveal 0-2",
            "line below 2 [\"└──────┴──────┘\" compmd.table.border] reveal 0-2",
            "replace 2:0-2:1 \"│\" compmd.table.border reveal 0-2",
            "replace 2:7-2:8 \"│\" compmd.table.border reveal 0-2",
            "replace 2:14-2:15 \"│\" compmd.table.border reveal 0-2",
        ]
    );
}

#[test]
fn a_table_without_outer_pipes_and_in_a_list_is_drawn_where_it_is() {
    assert_eq!(
        drawn(
            "- Boats\n\n  Boat | Crew\n  ---- | ----\n  Ada  | 3\n",
            all()
        ),
        [
            "replace 0:0-0:1 \"●\" compmd.bullet.1 reveal 0",
            "line above 2 [\"  ─────┬─────\" compmd.table.border] reveal 2-4",
            "replace 2:7-2:8 \"│\" compmd.table.border reveal 2-4",
            "replace 3:2-3:13 \"─────┼─────\" compmd.table.border reveal 2-4",
            "line below 4 [\"  ─────┴──\" compmd.table.border] reveal 2-4",
            "replace 4:7-4:8 \"│\" compmd.table.border reveal 2-4",
        ]
    );
}
