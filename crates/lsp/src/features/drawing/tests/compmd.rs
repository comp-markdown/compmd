//! CompMD's elements, with every primitive (decision 11).

use super::*;

#[test]
fn a_container_has_a_gutter_inside_and_its_opener_slots_and_closer_styled_where_they_are() {
    let text = "::: box {padding=m .wide}\nInside.\n:: #footer\nBelow.\n::: /box\n";
    assert_eq!(
        drawn(text, all()),
        [
            "style 0:0-0:3 compmd.container.border",
            "style 0:4-0:7 compmd.component",
            "style 0:8-0:25 compmd.param",
            "prefix 1-3 \"│ \" compmd.container.border",
            "style 2:0-2:4 compmd.container.border",
            "style 2:4-2:10 compmd.slot",
            "style 4:0-4:8 compmd.container.border",
        ]
    );
}

#[test]
fn a_box_in_a_box_has_the_outer_gutter_before_its_own() {
    let text = "::: box\n::: box\nIn.\n::: /box\n```\ncode\n```\n::: /box\n";
    assert_eq!(
        drawn(text, all()),
        [
            "style 0:0-0:3 compmd.container.border",
            "style 0:4-0:7 compmd.component",
            "prefix 1-6 \"│ \" compmd.container.border",
            "style 1:0-1:3 compmd.container.border",
            "style 1:4-1:7 compmd.component",
            "prefix 2 \"│ \" compmd.container.border",
            "style 3:0-3:8 compmd.container.border",
            "style 4:0-4:3 compmd.code.fence",
            "style 6:0-6:3 compmd.code.fence",
            "style 7:0-7:8 compmd.container.border",
        ]
    );
}

#[test]
fn a_box_further_in_than_a_lines_start_inserts_its_gutter_at_its_column() {
    let text = "- Item\n\n  ::: box\n  Inside.\n  ::: /box\n";
    assert_eq!(
        drawn(text, all()),
        [
            "replace 0:0-0:1 \"●\" compmd.bullet.1 reveal 0",
            "style 2:2-2:5 compmd.container.border",
            "style 2:6-2:9 compmd.component",
            "insert 3:2 \"│ \" compmd.container.border",
            "style 4:2-4:10 compmd.container.border",
        ]
    );
}

#[test]
fn a_container_recovery_closed_has_its_gutter_to_its_last_line() {
    let text = "::: box\nOpen.\n";
    assert_eq!(
        drawn(text, all()),
        [
            "style 0:0-0:3 compmd.container.border",
            "style 0:4-0:7 compmd.component",
            "prefix 1 \"│ \" compmd.container.border",
        ]
    );
}

#[test]
fn a_component_definition_is_left_as_written() {
    let text = "```compmd component=note\n---\n# Note\n```\n";
    assert_eq!(drawn(text, all()), Vec::<String>::new());
}

#[test]
fn a_leafs_name_is_in_the_component_style_and_its_params_and_the_md_key_are_set_back() {
    let text = "---\ntitle: Notes\nmd:\n  lang: en\n---\n\n:: box[A note] {padding=s}\n";
    assert_eq!(
        drawn(text, all())[3..],
        [
            "style 2:0-2:2 compmd.frontmatter.md",
            "hide 4:0-5:0 reveal 4",
            "line above 4 [] fill \"─\" compmd.rule reveal 4",
            "style 6:3-6:6 compmd.component",
            "style 6:15-6:26 compmd.param",
        ]
    );
}

#[test]
fn a_frontmatters_fences_are_rules_giving_way_one_by_one_and_its_lines_set_back_until_the_cursor_is_in_it()
 {
    let text = "---\ntitle: Notes\n---\n\nText.\n";
    assert_eq!(
        drawn(text, all()),
        [
            "hide 0:0-1:0 reveal 0",
            "line above 0 [] fill \"─\" compmd.rule reveal 0",
            "style 1:0-2:0 compmd.frontmatter whole reveal 0-2",
            "hide 2:0-3:0 reveal 2",
            "line above 2 [] fill \"─\" compmd.rule reveal 2",
        ]
    );
    let toml = drawn("+++\ntitle = \"Notes\"\n+++\n", all());
    assert_eq!(toml[0], "hide 0:0-1:0 reveal 0", "TOML's fences too");
    assert_eq!(
        drawn("---\n---\n", all()).len(),
        4,
        "an empty frontmatter has its fences only"
    );
}

#[test]
fn a_leaf_with_a_param_list_broken_over_lines_sets_back_each_line() {
    let text = ":: box {padding=s}\n::: + gap=m\n";
    assert_eq!(
        drawn(text, all()),
        [
            "style 0:3-0:6 compmd.component",
            "style 0:7-0:18 compmd.param",
            "style 1:6-1:11 compmd.param",
        ]
    );
}

#[test]
fn a_containers_continuation_lines_are_styled_as_its_opener_and_outside_its_gutter() {
    let text = "::: box {padding=s}\n::: + gap=m\nIn.\n::: /box\n";
    assert_eq!(
        drawn(text, all()),
        [
            "style 0:0-0:3 compmd.container.border",
            "style 0:4-0:7 compmd.component",
            "style 0:8-0:19 compmd.param",
            "style 1:0-1:5 compmd.container.border",
            "style 1:6-1:11 compmd.param",
            "prefix 2 \"│ \" compmd.container.border",
            "style 3:0-3:8 compmd.container.border",
        ]
    );
}
