use lsp_types::{Position, Range};

use super::*;
use crate::encoding::Encoding;
use crate::features::testing::{asking, processed};

fn first_lines(text: &str) -> Vec<usize> {
    let processor = processed(text);
    let source = Source::new(text, Encoding::Utf16);
    tables_of(&processor.tree().children, &source)
        .iter()
        .map(|table| table.first_line)
        .collect()
}

#[test]
fn tables_are_found_in_containers_and_their_slots_not_in_definitions() {
    let text =
        "::: card\n\n| a |\n|-|\n\n:: #footer\n\n| b |\n|-|\n\n::: /card\n\n> | c |\n> |-|\n";
    assert_eq!(first_lines(text), [2, 7, 12]);
    assert_eq!(
        first_lines("```compmd component=note\n| a |\n|-|\n```\n"),
        Vec::<usize>::new()
    );
}

#[test]
fn alignment_edits_keep_a_lines_carriage_return() {
    let text = "| a |\r\n|-|\r\n";
    let edits = asking(text, &processed(text), &[], |asked| {
        alignment_edits(asked, None)
    });
    assert_eq!(
        edits,
        [
            TextEdit {
                range: Range::new(Position::new(0, 3), Position::new(0, 4)),
                new_text: "   ".into()
            },
            TextEdit {
                range: Range::new(Position::new(1, 1), Position::new(1, 2)),
                new_text: " --- ".into()
            }
        ]
    );
}
