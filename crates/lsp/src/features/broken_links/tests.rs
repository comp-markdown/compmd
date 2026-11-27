//! The broken-link warning on a project on the disk.

use lsp_types::{Diagnostic, NumberOrString};

use super::{CODE, broken_links, is_broken_link};
use crate::features::testing::{asking_in_project, project_folder};

const TIDES: &str = "# Tides\n\n## High water\n";

/// The document's broken-link warnings, each its line, its first column and its message.
fn warned(name: &str, text: &str) -> Vec<(u32, u32, String)> {
    asking_in_project(
        &project_folder(name),
        &[("tides.md", TIDES), ("maps/harbour.png", "")],
        ("posts/harbour.md", text),
        |asked| {
            broken_links(asked)
                .into_iter()
                .map(|diagnostic| {
                    assert_is_the_servers(&diagnostic);
                    let start = diagnostic.range.start;
                    (start.line, start.character, diagnostic.message)
                })
                .collect()
        },
    )
}

fn assert_is_the_servers(diagnostic: &Diagnostic) {
    assert!(is_broken_link(diagnostic));
    assert_eq!(diagnostic.code, Some(NumberOrString::String(CODE.into())));
    assert_eq!(diagnostic.source.as_deref(), Some("compmd"));
}

#[test]
fn a_link_to_a_file_or_a_heading_that_isnt_there_is_warned_about() {
    let text = "# Harbour\n\n[a](../tides.md) [b](../gone.md) [c](../tides.md#high-water) [d](../tides.md#low-water) [e](#harbour) [f](#nowhere) [g](../maps/harbour.png) [h](https://example.com/x.md)\n";
    assert_eq!(
        warned("files-and-anchors", text),
        [
            (2, 17, "no file `../gone.md` is in the project".into()),
            (2, 61, "`../tides.md` has no heading `#low-water`".into()),
            (2, 102, "the document has no heading `#nowhere`".into()),
        ]
    );
}

#[test]
fn a_definitions_target_and_a_label_no_definition_defines() {
    let text = "See [the tides][tides] and [the tables][tables].\n\n[tables]: ../tables.md\n";
    assert_eq!(
        warned("references", text),
        [
            (2, 0, "no file `../tables.md` is in the project".into()),
            (0, 16, "no definition defines the label `tides`".into()),
        ]
    );
}

#[test]
fn a_passage_that_matches_nothing() {
    let text = "[a](../tides.md#:~:text=High) [b](../tides.md#:~:text=dawn) [c](#:~:text=nowhere%20at%20all) [d](#:~:text=%5Ba%5D)\n";
    assert_eq!(
        warned("passages", text),
        [
            (
                0,
                30,
                "nothing in `../tides.md` matches the passage `dawn`".into()
            ),
            (
                0,
                60,
                "nothing in the document matches the passage `nowhere at all`".into()
            ),
        ]
    );
}

#[test]
fn a_wiki_link_to_a_page_or_a_heading_that_isnt_there() {
    let text = "---\nmd:\n  syntax: [wiki-links]\n---\n# Harbour\n\n[[Tides]] [[Gone]] [[tides#high water]] [[Tides#Low water]] [[#Harbour]] [[#Nowhere]] [[Tides|tides]]\n";
    assert_eq!(
        warned("wiki", text),
        [
            (6, 10, "no page `Gone` is in the project".into()),
            (6, 40, "`Tides` has no heading `Low water`".into()),
            (6, 73, "the document has no heading `Nowhere`".into()),
        ]
    );
}

#[test]
fn without_wiki_links_double_brackets_are_a_reference_in_brackets() {
    assert_eq!(
        warned("no-wiki", "[[Gone]]\n"),
        [(0, 2, "no definition defines the label `Gone`".into())],
        "not the outer brackets', which no definition can define"
    );
}

#[test]
fn a_wiki_links_page_waits_for_the_index_and_isnt_checked_with_links_wiki_off() {
    let text = "---\nmd:\n  syntax: [wiki-links]\n---\n[[Gone]]\n";
    let processor = crate::features::testing::processed_file("notes.comp.md", text);
    let without_index =
        crate::features::testing::asking_file("notes.comp.md", text, &processor, &[], broken_links);
    assert_eq!(without_index, [], "a page isn't known missing");
    let off = asking_in_project(
        &project_folder("wiki-off"),
        &[],
        ("notes.comp.md", text),
        |asked| {
            broken_links(&crate::features::Asked {
                links: crate::settings::LinkSettings {
                    wiki: false,
                    ..crate::settings::LinkSettings::default()
                },
                ..*asked
            })
        },
    );
    assert_eq!(off, []);
}
