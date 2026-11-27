use lsp_types::CodeAction;

use super::super::code_actions;
use super::super::tests::{applied, caret_at};
use crate::features::testing::{asking_file, processed_file};

/// The actions at the caret on line `line` of the file `name` with `text`.
fn actions_in(name: &str, text: &str, line: u32) -> Vec<CodeAction> {
    let processor = processed_file(name, text);
    asking_file(name, text, &processor, &[], |asked| {
        code_actions(caret_at(line, 0), None, asked)
    })
}

fn titles(actions: &[CodeAction]) -> Vec<&str> {
    actions.iter().map(|action| action.title.as_str()).collect()
}

/// `text` with the action *Read as CompMD* of the file `name` applied.
fn read_as_compmd(name: &str, text: &str) -> String {
    let actions = actions_in(name, text, 0);
    let read = actions
        .iter()
        .find(|action| action.title == "Read as CompMD")
        .unwrap_or_else(|| panic!("offered: {:?}", titles(&actions)));
    applied(text, read)
}

#[test]
fn a_plain_markdown_file_has_markdowns_actions_and_read_as_compmd() {
    let text = "---\ntitle: Notes\n---\n\n::: gird\nText.\n:::\n";
    let actions = actions_in("notes.md", text, 4);
    assert_eq!(titles(&actions), ["Read as CompMD", "Open preview"]);
    let table = "| a | b |\n|---|---|\n| 1 | 2 |\n";
    let in_table = actions_in("notes.md", table, 2);
    assert!(
        titles(&in_table).contains(&"Insert a row below"),
        "{:?}",
        titles(&in_table)
    );
}

#[test]
fn the_md_key_goes_at_the_end_of_the_frontmatter_in_its_syntax_or_in_a_new_one() {
    assert_eq!(
        read_as_compmd("notes.md", "---\ntitle: Notes\n---\n# A\n"),
        "---\ntitle: Notes\nmd:\n  version: 1\n---\n# A\n"
    );
    assert_eq!(
        read_as_compmd("notes.md", "+++\ntitle = \"Notes\"\n+++\n# A\n"),
        "+++\ntitle = \"Notes\"\n[md]\nversion = 1\n+++\n# A\n"
    );
    assert_eq!(
        read_as_compmd("notes.md", "# A\n"),
        "---\nmd:\n  version: 1\n---\n\n# A\n"
    );
}

#[test]
fn a_compmd_file_or_a_frontmatter_that_doesnt_read_isnt_offered_it() {
    let offered =
        |name: &str, text: &str| titles(&actions_in(name, text, 0)).contains(&"Read as CompMD");
    assert!(!offered("notes.comp.md", "# A\n"));
    assert!(!offered("notes.md", "---\nmd: {}\n---\n# A\n"));
    assert!(
        !offered("notes.md", "---\ntitle: [\n---\n# A\n"),
        "a frontmatter that doesn't read"
    );
    let text = "---\nmd: {}\n---\n::: gird\nText.\n:::\n";
    let processor = processed_file("notes.md", text);
    let compmd = asking_file("notes.md", text, &processor, &[], |asked| {
        code_actions(caret_at(3, 5), None, asked)
    });
    assert!(
        titles(&compmd).contains(&"Change to `grid`"),
        "{:?}",
        titles(&compmd)
    );
}
