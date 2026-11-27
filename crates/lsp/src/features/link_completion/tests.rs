//! Completion of links on a project on the disk.

use lsp_types::{CompletionItem, CompletionTextEdit, Position};

use super::link_completions;
use crate::features::testing::{asking_in_project, caret, project_folder};

const TIDES: &str = "# Tides\n\n## High water\n\nAt noon.\n\n## Low water\n";

/// The completions at the caret `|` of `marked`, the document `posts/harbour.md` of a project with `tides.md`,
/// `posts/log.md` and `posts/2026/high water.md`; `None` where no link is being typed.
fn completed(name: &str, marked: &str) -> Option<Vec<CompletionItem>> {
    let (text, at) = caret(marked);
    let before: String = text.chars().take(at).collect();
    let line = u32::try_from(before.matches('\n').count()).expect("a line");
    let character = before
        .rsplit('\n')
        .next()
        .unwrap_or_default()
        .chars()
        .count();
    let position = Position {
        line,
        character: u32::try_from(character).expect("a column"),
    };
    asking_in_project(
        &project_folder(name),
        &[
            ("tides.md", TIDES),
            ("posts/log.md", "A log.\n"),
            ("posts/2026/high water.md", "# High\n"),
        ],
        ("posts/harbour.md", &text),
        |asked| link_completions(position, asked),
    )
}

/// Each completion's label, and the columns its edit replaces.
fn written(items: &[CompletionItem]) -> Vec<(String, u32, u32)> {
    items
        .iter()
        .map(|item| {
            let Some(CompletionTextEdit::Edit(edit)) = &item.text_edit else {
                panic!("an edit");
            };
            (
                item.label.clone(),
                edit.range.start.character,
                edit.range.end.character,
            )
        })
        .collect()
}

fn labels(items: &[CompletionItem]) -> Vec<&str> {
    items.iter().map(|item| item.label.as_str()).collect()
}

#[test]
fn a_target_is_offered_the_projects_files_relative_to_the_document() {
    let items = completed("files", "# Harbour\n\nSee [the tides](../t|\n").expect("a link");
    assert_eq!(
        labels(&items),
        ["2026/high%20water.md", "log.md", "../tides.md"],
        "every file but the document itself"
    );
    assert_eq!(
        written(&items)[2],
        ("../tides.md".into(), 16, 20),
        "what is written of it replaced"
    );
    let defined = completed("definition-files", "[tides]: |\n").expect("a definition");
    assert_eq!(defined.len(), 3, "a definition's target too");
}

#[test]
fn after_a_hash_its_files_anchors_or_the_documents_own() {
    let items = completed("anchors", "# Harbour\n\nSee [high](../tides.md#hi|\n").expect("a link");
    assert_eq!(labels(&items), ["tides", "high-water", "low-water"]);
    assert_eq!(items[1].detail.as_deref(), Some("## High water"));
    assert_eq!(
        written(&items)[1],
        ("high-water".into(), 23, 25),
        "the anchor written replaced"
    );
    let own = completed("own-anchors", "# Harbour notes\n\n[top](#|\n").expect("a link");
    assert_eq!(labels(&own), ["harbour-notes"]);
    let gone = completed("gone-anchors", "[a](gone.md#|\n").expect("a link");
    assert!(gone.is_empty(), "a file that isn't there has none");
}

#[test]
fn a_references_label_is_offered_the_documents_definitions_once() {
    let text = "See [the tides][ti|\n\n[tides]: ../tides.md\n[Tides]: ./again.md\n[log]: log.md\n";
    let items = completed("labels", text).expect("a reference");
    assert_eq!(labels(&items), ["tides", "log"]);
    assert_eq!(items[0].detail.as_deref(), Some("../tides.md"));
    assert_eq!(written(&items)[0], ("tides".into(), 16, 18));
}

#[test]
fn outside_a_link_there_is_nothing_to_complete() {
    assert_eq!(completed("prose", "Plain |text.\n"), None);
}
