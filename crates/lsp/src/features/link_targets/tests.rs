//! Links and headings' features on a project on the disk: go to definition, references and hover.

use lsp_types::{HoverContents, Location, Position};

use super::super::link_definition::link_definition;
use super::super::link_hover::link_hover;
use super::super::references::references;
use crate::features::testing::{asking_in_project, project_folder};

/// The project's other files: a page with two sections, and a plain note.
const TIDES: &str = "# Tides\n\nThe sea comes in.\n\n## High water\n\nAt noon.\nAnd at midnight.\n\n### Spring tides\n\nHigher.\n\n## Low water\n\nAt six.\n";
const KNOTS: &str = "Knots, without a heading.\n";

/// The document: links to the other files, to its own heading, a reference and its definition.
const HARBOUR: &str = "# Harbour notes\n\nSee [high water](tides.md#high-water), [knots](knots.md), [the top](#harbour-notes), [nowhere](gone.md) and [the tables][tables].\n\n[tables]: ./tides.md#low-water\n";

fn at(line: u32, character: u32) -> Position {
    Position { line, character }
}

/// The line `location` starts on, and its file's name.
fn place(location: &Location) -> (String, u32) {
    let file = location
        .uri
        .as_str()
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_owned();
    (file, location.range.start.line)
}

fn in_project<R>(name: &str, answer: impl FnOnce(&crate::features::Asked) -> R) -> R {
    asking_in_project(
        &project_folder(name),
        &[
            ("tides.md", TIDES),
            ("knots.md", KNOTS),
            ("posts/log.md", "Tide at [noon](../tides.md#high-water).\n"),
        ],
        ("harbour.md", HARBOUR),
        answer,
    )
}

#[test]
fn a_link_goes_to_its_file_its_heading_or_its_definition() {
    in_project("definition", |asked| {
        let go =
            |character| link_definition(at(2, character), asked).map(|location| place(&location));
        assert_eq!(go(6), Some(("tides.md".into(), 4)), "the heading it names");
        assert_eq!(go(45), Some(("knots.md".into(), 0)), "the file");
        assert_eq!(
            go(66),
            Some(("harbour.md".into(), 0)),
            "the document's own heading"
        );
        assert_eq!(go(90), None, "a file that isn't there");
        assert_eq!(
            go(115),
            Some(("harbour.md".into(), 4)),
            "a reference's definition"
        );
        assert_eq!(link_definition(at(2, 1), asked), None, "off a link");
        assert_eq!(
            link_definition(at(4, 3), asked).map(|location| place(&location)),
            Some(("tides.md".into(), 13)),
            "a definition's destination"
        );
    });
}

#[test]
fn without_an_index_a_document_reaches_its_own_headings_and_the_files_on_the_disk() {
    let root = project_folder("no-index");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a folder");
    std::fs::write(root.join("tides.md"), TIDES).expect("written");
    let text = "# Notes\n\n[top](#notes) [tides](tides.md#high-water) [here](notes.md)\n";
    let processor = crate::features::testing::processed_file("notes.md", text);
    let uri = super::uri_of(&root.join("notes.md")).expect("a URI");
    let places = crate::places::Places::default();
    let path = root.join("notes.md");
    let asked = crate::features::Asked {
        processor: Some(&processor),
        uri: &uri,
        plain_markdown_file: true,
        path: &path,
        index: None,
        links: true,
        text,
        version: 1,
        encoding: crate::encoding::Encoding::Utf16,
        places: &places,
        pack_components: &[],
        unregistered: None,
    };
    assert_eq!(super::root(&asked), root, "the document's folder");
    let go = |character| link_definition(at(2, character), &asked).map(|location| place(&location));
    assert_eq!(go(2), Some(("notes.md".into(), 0)));
    assert_eq!(
        go(18),
        Some(("tides.md".into(), 0)),
        "the file, its headings not indexed"
    );
    assert_eq!(go(48), Some(("notes.md".into(), 0)), "itself by name");
    assert_eq!(references(at(0, 2), false, &asked).len(), 1);
}

#[test]
fn a_headings_references_are_its_links_across_the_project() {
    let found = asking_in_project(
        &project_folder("references"),
        &[
            ("harbour.md", HARBOUR),
            ("posts/log.md", "Tide at [noon](../tides.md#high-water).\n"),
        ],
        ("tides.md", TIDES),
        |asked| references(at(4, 3), false, asked),
    );
    let mut places: Vec<(String, u32)> = found.iter().map(place).collect();
    places.sort();
    assert_eq!(places, [("harbour.md".into(), 2), ("log.md".into(), 0)]);
}

#[test]
fn references_with_the_declaration_and_of_a_label() {
    in_project("labels", |asked| {
        let own: Vec<(String, u32)> = references(at(0, 2), true, asked)
            .iter()
            .map(place)
            .collect();
        assert_eq!(
            own,
            [("harbour.md".into(), 2), ("harbour.md".into(), 0)],
            "its link, then itself"
        );
        let uses = references(at(4, 2), false, asked);
        assert_eq!(
            uses.iter().map(place).collect::<Vec<_>>(),
            [("harbour.md".into(), 2)]
        );
        assert_eq!(
            references(at(4, 2), true, asked).len(),
            2,
            "and its definition"
        );
        assert_eq!(
            references(at(2, 1), true, asked),
            Vec::new(),
            "nothing there"
        );
    });
}

#[test]
fn a_hover_shows_the_targets_section_or_the_files_start() {
    in_project("hover", |asked| {
        let shown = |character| {
            link_hover(at(2, character), asked).map(|hover| match hover.contents {
                HoverContents::Markup(markup) => markup.value,
                _ => String::new(),
            })
        };
        assert_eq!(
            shown(6).as_deref(),
            Some("## High water\n\nAt noon.\nAnd at midnight.\n\n### Spring tides\n\nHigher."),
            "to the next heading of its level"
        );
        assert_eq!(shown(45).as_deref(), Some("Knots, without a heading."));
        assert_eq!(shown(90), None);
    });
}
