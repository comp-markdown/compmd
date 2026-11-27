//! CompMD's elements, with every primitive (decision 11).

use super::*;

#[test]
fn a_container_is_a_box_titled_with_its_name_and_params_and_a_slot_a_divider() {
    let text = "::: box {padding=m .wide}\nInside.\n:: #footer\nBelow.\n::: /box\n";
    assert_eq!(
        drawn(text, all()),
        [
            "hide 0:0-1:0 reveal 0",
            "line above 0 [\"┌─ \" compmd.container.border, \"box\" compmd.component, \" \" compmd.container.border, \"{padding=m .wide}\" compmd.param, \" \" compmd.container.border] fill \"─\" compmd.container.border reveal 0",
            "prefix 1 \"│ \" compmd.container.border",
            "hide 2:0-3:0 reveal 2",
            "line above 2 [\"├─ \" compmd.container.border, \"footer\" compmd.slot, \" \" compmd.container.border] fill \"─\" compmd.container.border reveal 2",
            "prefix 3 \"│ \" compmd.container.border",
            "hide 4:0-5:0 reveal 4",
            "line above 4 [\"└─\" compmd.container.border] fill \"─\" compmd.container.border reveal 4",
        ]
    );
}

#[test]
fn a_box_in_a_box_starts_its_lines_of_its_own_with_the_outer_side() {
    let text = "::: box\n::: box\nIn.\n::: /box\n```\ncode\n```\n::: /box\n";
    assert_eq!(
        drawn(text, all()),
        [
            "hide 0:0-1:0 reveal 0",
            "line above 0 [\"┌─ \" compmd.container.border, \"box\" compmd.component, \" \" compmd.container.border] fill \"─\" compmd.container.border reveal 0",
            "prefix 1-6 \"│ \" compmd.container.border",
            "hide 1:0-2:0 reveal 1",
            "line above 1 [\"│ \" compmd.container.border, \"┌─ \" compmd.container.border, \"box\" compmd.component, \" \" compmd.container.border] fill \"─\" compmd.container.border reveal 1",
            "prefix 2 \"│ \" compmd.container.border",
            "hide 3:0-4:0 reveal 3",
            "line above 3 [\"│ \" compmd.container.border, \"└─\" compmd.container.border] fill \"─\" compmd.container.border reveal 3",
            "style 4:0-6:3 compmd.code whole",
            "hide 4:0-4:3 reveal 4",
            "hide 6:0-6:3 reveal 6",
            "hide 7:0-8:0 reveal 7",
            "line above 7 [\"└─\" compmd.container.border] fill \"─\" compmd.container.border reveal 7",
        ]
    );
}

#[test]
fn a_box_further_in_than_a_lines_start_inserts_its_side_at_its_column() {
    let text = "- Item\n\n  ::: box\n  Inside.\n  ::: /box\n";
    assert_eq!(
        drawn(text, all()),
        [
            "replace 0:0-0:1 \"●\" compmd.bullet.1 reveal 0",
            "line above 2 [\"┌─ \" compmd.container.border, \"box\" compmd.component, \" \" compmd.container.border] fill \"─\" compmd.container.border reveal 2",
            "hide 2:2-2:9 reveal 2",
            "insert 3:2 \"│ \" compmd.container.border",
            "line above 4 [\"└─\" compmd.container.border] fill \"─\" compmd.container.border reveal 4",
            "hide 4:2-4:10 reveal 4",
        ]
    );
}

#[test]
fn a_container_recovery_closed_has_no_bottom_border() {
    let text = "::: box\nOpen.\n";
    assert_eq!(
        drawn(text, all()),
        [
            "hide 0:0-1:0 reveal 0",
            "line above 0 [\"┌─ \" compmd.container.border, \"box\" compmd.component, \" \" compmd.container.border] fill \"─\" compmd.container.border reveal 0",
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
        drawn(text, all()),
        [
            "style 2:0-2:2 compmd.frontmatter.md",
            "style 6:3-6:6 compmd.component",
            "style 6:15-6:26 compmd.param",
        ]
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
fn a_containers_continuation_lines_are_in_its_top_border() {
    let text = "::: box {padding=s}\n::: + gap=m\nIn.\n::: /box\n";
    let marks = drawn(text, all());
    assert_eq!(
        marks[..3],
        [
            "hide 0:0-2:0 reveal 0-1",
            "line above 0 [\"┌─ \" compmd.container.border, \"box\" compmd.component, \" \" compmd.container.border, \"{padding=s}\" compmd.param, \" \" compmd.container.border, \"gap=m\" compmd.param, \" \" compmd.container.border] fill \"─\" compmd.container.border reveal 0-1",
            "prefix 2 \"│ \" compmd.container.border",
        ]
    );
}
