use lsp_types::CodeActionKind;

use super::super::tests::{actions_of, applied, at};
use super::*;

const TABLE: &str = "Notes.\n\n| port | depth |\n|:-|-:|\n| Oban | 4 |\n";

/// The table actions with the cursor at `line` and `character` of `text`.
fn offered(text: &str, line: u32, character: u32) -> Vec<CodeAction> {
    let cursor = Range::new(at(line, character), at(line, character));
    actions_of(CodeActionKind::REFACTOR_REWRITE.as_str(), text, cursor)
}

/// `text` with the action titled `title`, at `line` and `character`, made.
fn made(text: &str, (line, character): (u32, u32), title: &str) -> String {
    let actions = offered(text, line, character);
    let action = actions
        .iter()
        .find(|action| action.title == title)
        .unwrap_or_else(|| panic!("{title} offered"));
    applied(text, action)
}

#[test]
fn each_change_leaves_the_table_aligned() {
    assert_eq!(
        made(TABLE, (4, 3), "Insert a row below"),
        "Notes.\n\n| port | depth |\n| :--- | ----: |\n| Oban |     4 |\n|      |       |\n"
    );
    assert_eq!(
        made(TABLE, (4, 3), "Delete the row"),
        "Notes.\n\n| port | depth |\n| :--- | ----: |\n"
    );
    assert_eq!(
        made(TABLE, (2, 9), "Insert a column before"),
        "Notes.\n\n| port |     | depth |\n| :--- | --- | ----: |\n| Oban |     |     4 |\n"
    );
    assert_eq!(
        made(TABLE, (2, 9), "Delete the column"),
        "Notes.\n\n| port |\n| :--- |\n| Oban |\n"
    );
    assert_eq!(
        made(TABLE, (3, 1), "Move the column right"),
        "Notes.\n\n| depth | port |\n| ----: | :--- |\n|     4 | Oban |\n"
    );
    assert_eq!(
        made(TABLE, (4, 12), "Align the column center"),
        "Notes.\n\n| port | depth |\n| :--- | :---: |\n| Oban |   4   |\n"
    );
}

#[test]
fn row_changes_are_offered_on_rows_only_and_nothing_outside_a_table() {
    let titles = |line, character| {
        offered(TABLE, line, character)
            .into_iter()
            .map(|action| action.title)
            .collect::<Vec<_>>()
    };
    assert!(!titles(3, 2).iter().any(|title| title.contains("row")));
    assert!(titles(2, 0).contains(&"Insert a row below".to_owned()));
    assert!(!titles(2, 0).contains(&"Delete the row".to_owned()));
    assert_eq!(titles(0, 0), Vec::<String>::new());
}

#[test]
fn a_table_is_rewritten_with_the_documents_line_ends() {
    let text = "| a |\r\n|-|\r\n| 1 |\r\n";
    let actions = offered(text, 2, 1);
    let action = actions
        .iter()
        .find(|action| action.title == "Delete the row")
        .expect("offered");
    assert_eq!(
        applied(text, action),
        "| a   |\r\n| --- |\r\n",
        "the last line's end kept"
    );
}
