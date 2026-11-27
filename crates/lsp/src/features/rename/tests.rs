//! Rename on a project on the disk.

use lsp_types::{Position, TextEdit, WorkspaceEdit};

use super::{prepare_rename, rename};
use crate::features::testing::{asking_in_project, project_folder};

/// The document: a heading two files link to, links to it and to another file's, and a label with its uses.
const HARBOUR: &str = "# Harbour notes\n\nBack to [the top](#harbour-notes), on to [the tides](tides.md#high-water).\n\nSee [the log][Log] and [log][] and [log].\n\n[log]: ./log.md\n";

fn at(line: u32, character: u32) -> Position {
    Position { line, character }
}

fn in_project<R>(name: &str, answer: impl FnOnce(&crate::features::Asked) -> R) -> R {
    asking_in_project(
        &project_folder(name),
        &[
            (
                "tides.md",
                "# Tides\n\n## High water\n\n[Notes](harbour.md#harbour-notes#x) [home](./harbour.md#harbour-notes)\n",
            ),
            (
                "posts/log.md",
                "[Harbour](../harbour.md#harbour-notes \"The notes\")\n",
            ),
        ],
        ("harbour.md", HARBOUR),
        answer,
    )
}

/// An edit as `(line, from, to, new text)`.
type Edited = (u32, u32, u32, String);

/// Each file's edits, by its name, the files in order.
fn edits_of(edit: &WorkspaceEdit) -> Vec<(String, Vec<Edited>)> {
    let mut files: Vec<(String, Vec<Edited>)> = edit
        .changes
        .as_ref()
        .expect("changes")
        .iter()
        .map(|(uri, edits)| {
            let name = uri
                .as_str()
                .rsplit_once("compmd-lsp-links-")
                .map_or("", |(_, rest)| rest);
            let name = name
                .split_once('/')
                .map_or(name, |(_, file)| file)
                .to_owned();
            let mut edits: Vec<(u32, u32, u32, String)> = edits
                .iter()
                .map(|TextEdit { range, new_text }| {
                    (
                        range.start.line,
                        range.start.character,
                        range.end.character,
                        new_text.clone(),
                    )
                })
                .collect();
            edits.sort();
            (name, edits)
        })
        .collect();
    files.sort();
    files
}

#[test]
fn a_headings_text_is_renamed_with_every_link_to_its_anchor() {
    in_project("rename-heading", |asked| {
        let range = prepare_rename(at(0, 4), asked).expect("its text");
        assert_eq!((range.start.character, range.end.character), (2, 15));
        let edit = rename(at(0, 4), "Harbour log", asked).expect("an edit");
        assert_eq!(
            edits_of(&edit),
            [
                (
                    "harbour.md".into(),
                    vec![
                        (0, 2, 15, "Harbour log".into()),
                        (2, 19, 32, "harbour-log".into())
                    ]
                ),
                (
                    "posts/log.md".into(),
                    vec![(0, 24, 37, "harbour-log".into())]
                ),
                ("tides.md".into(), vec![(4, 56, 69, "harbour-log".into())]),
            ],
            "the link with a passage-like second `#` isn't one to it"
        );
    });
}

#[test]
fn a_heading_whose_anchor_stays_has_only_its_text_edited() {
    in_project("rename-same-anchor", |asked| {
        let edit = rename(at(0, 4), "Harbour Notes", asked).expect("an edit");
        assert_eq!(
            edits_of(&edit),
            [(
                "harbour.md".into(),
                vec![(0, 2, 15, "Harbour Notes".into())]
            )]
        );
    });
}

#[test]
fn a_label_is_renamed_with_its_definition_and_every_reference() {
    in_project("rename-label", |asked| {
        let range = prepare_rename(at(4, 8), asked).expect("its label");
        assert_eq!((range.start.character, range.end.character), (14, 17));
        let edit = rename(at(4, 8), "notes", asked).expect("an edit");
        assert_eq!(
            edits_of(&edit),
            [(
                "harbour.md".into(),
                vec![
                    (4, 14, 17, "notes".into()),
                    (4, 24, 27, "notes".into()),
                    (4, 36, 39, "notes".into()),
                    (6, 1, 4, "notes".into()),
                ]
            )]
        );
        assert_eq!(
            rename(at(4, 8), "no]tes", asked),
            None,
            "a bracket can't be in a label"
        );
    });
}

#[test]
fn a_headings_text_is_without_its_closing_marks_and_a_setext_headings_is_its_line() {
    let text = "## Tides ##  \n\nHigh water\n==========\n\n[a][x] [b][y]\n\n[x]: a.md\n[y]: b.md\n";
    asking_in_project(
        &project_folder("rename-forms"),
        &[],
        ("notes.md", text),
        |asked| {
            let closed = prepare_rename(at(0, 4), asked).expect("its text");
            assert_eq!((closed.start.character, closed.end.character), (3, 8));
            let setext = prepare_rename(at(2, 1), asked).expect("its text");
            assert_eq!((setext.start.character, setext.end.character), (0, 10));
            let edit = rename(at(5, 4), "z", asked).expect("an edit");
            assert_eq!(
                edits_of(&edit)[0].1,
                [(5, 4, 5, "z".into()), (7, 1, 2, "z".into())],
                "the other label stays"
            );
        },
    );
}

#[test]
fn an_inline_link_has_no_label_to_rename() {
    in_project("rename-inline", |asked| {
        assert_eq!(prepare_rename(at(2, 12), asked), None);
        assert_eq!(rename(at(2, 12), "x", asked), None);
    });
}

#[test]
fn prose_and_a_new_name_on_two_lines_rename_nothing() {
    in_project("rename-nothing", |asked| {
        assert_eq!(prepare_rename(at(2, 1), asked), None);
        assert_eq!(rename(at(0, 4), "Two\nlines", asked), None);
        assert_eq!(rename(at(0, 4), "  ", asked), None);
    });
}
