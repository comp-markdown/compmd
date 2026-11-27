//! Completion of wiki links on a project on the disk.

use lsp_types::{CompletionItem, Position};

use super::super::link_completion::link_completions;
use crate::features::testing::{asking_in_project, caret, project_folder};

const WIKI: &str = "---\nmd:\n  syntax: [wiki-links]\n---\n";

/// The labels completed at the caret `|` of the line after the frontmatter `front` of `posts/harbour.comp.md`, in a
/// project with `tides.md`, `posts/log.md` and `notes/log.md`; `None` where nothing is.
fn completed(name: &str, front: &str, marked: &str) -> Option<Vec<(String, Option<String>)>> {
    let (line, at) = caret(marked);
    let text = format!("{front}# Harbour\n\n{line}\n");
    let position = Position {
        line: u32::try_from(text.matches('\n').count() - 1).expect("a line"),
        character: u32::try_from(at).expect("a column"),
    };
    asking_in_project(
        &project_folder(name),
        &[
            ("tides.md", "# Tides\n\n## High water\n"),
            ("posts/log.md", "A log.\n"),
            ("notes/log.md", "Another log.\n"),
        ],
        ("posts/harbour.comp.md", &text),
        |asked| link_completions(position, asked),
    )
    .map(|items: Vec<CompletionItem>| {
        items
            .into_iter()
            .map(|item| (item.label, item.detail))
            .collect()
    })
}

#[test]
fn after_two_brackets_the_projects_pages_each_once() {
    assert_eq!(
        completed("wiki-pages", WIKI, "See [[ti|"),
        Some(vec![
            ("log".into(), Some("notes/log.md".into())),
            ("tides".into(), Some("tides.md".into())),
        ])
    );
}

#[test]
fn after_a_hash_the_pages_headings_or_the_documents_by_their_text() {
    let labels = |marked| {
        completed("wiki-headings", WIKI, marked)
            .expect("completions")
            .into_iter()
            .map(|(label, _)| label)
            .collect::<Vec<_>>()
    };
    assert_eq!(labels("[[Tides#|"), ["Tides", "High water"]);
    assert_eq!(labels("[[#H|"), ["Harbour"]);
    assert_eq!(labels("[[Gone#|"), Vec::<String>::new());
}

#[test]
fn a_document_without_wiki_links_completes_none() {
    assert_eq!(completed("wiki-none", "", "See [[ti|"), None);
}
