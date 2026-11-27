//! Wiki links and passages on a project on the disk: go to definition, references and hover.

use lsp_types::{HoverContents, Location, Position};

use super::super::link_definition::link_definition;
use super::super::link_hover::link_hover;
use super::super::references::references;
use crate::features::Asked;
use crate::features::testing::{asking_in_project, project_folder};
use crate::settings::LinkSettings;

/// The frontmatter of a document with wiki links.
const WIKI: &str = "---\nmd:\n  syntax: [wiki-links]\n---\n";

const TIDES: &str = "# Tides\n\n## High water\n\nAt noon, the price: 12.\n";

fn at(line: u32, character: u32) -> Position {
    Position { line, character }
}

/// The file `location` is in, its line and its column.
fn place(location: &Location) -> (String, u32, u32) {
    let file = location
        .uri
        .as_str()
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_owned();
    (
        file,
        location.range.start.line,
        location.range.start.character,
    )
}

/// `answer` on the document `harbour.comp.md`, whose line 6 has wiki links and line 8 a passage link, in a project
/// with `tides.md` and a log linking to its heading.
fn in_project<R>(name: &str, answer: impl FnOnce(&Asked) -> R) -> R {
    let harbour = format!(
        "{WIKI}# Harbour notes\n\n[[Tides#High water|high water]] [[tides]] [[#Harbour notes]] [[Gone]]\n\n[noon](tides.md#:~:text=price:%2012)\n"
    );
    let log =
        format!("{WIKI}See [[harbour#harbour notes]], [[Harbour#Tides]] and [[#Harbour notes]].\n");
    asking_in_project(
        &project_folder(name),
        &[("tides.md", TIDES), ("posts/log.md", &log)],
        ("harbour.comp.md", &harbour),
        answer,
    )
}

#[test]
fn a_wiki_link_goes_to_its_page_and_heading_and_a_passage_to_its_match() {
    in_project("wiki-definition", |asked| {
        let go = |line, character| {
            link_definition(at(line, character), asked).map(|found| place(&found))
        };
        assert_eq!(go(6, 2), Some(("tides.md".into(), 2, 0)), "its heading");
        assert_eq!(
            go(6, 34),
            Some(("tides.md".into(), 0, 0)),
            "its page, any case"
        );
        assert_eq!(
            go(6, 45),
            Some(("harbour.comp.md".into(), 4, 0)),
            "the document's heading"
        );
        assert_eq!(go(6, 63), None, "a page that isn't there");
        assert_eq!(
            go(8, 2),
            Some(("tides.md".into(), 4, 13)),
            "the passage's first match"
        );
    });
}

#[test]
fn a_wiki_link_hovers_with_its_section_and_isnt_followed_with_links_wiki_off() {
    in_project("wiki-hover", |asked| {
        let hover = link_hover(at(6, 2), asked).expect("a hover");
        let HoverContents::Markup(markup) = hover.contents else {
            panic!("Markdown");
        };
        assert_eq!(markup.value, "## High water\n\nAt noon, the price: 12.");
        let off = Asked {
            links: LinkSettings {
                wiki: false,
                ..LinkSettings::default()
            },
            ..*asked
        };
        assert_eq!(link_definition(at(6, 2), &off), None);
        assert!(
            link_definition(at(8, 2), &off).is_some(),
            "a passage link is a link"
        );
    });
}

#[test]
fn a_headings_references_are_its_wiki_links_across_the_project() {
    in_project("wiki-references", |asked| {
        let mut found: Vec<(String, u32, u32)> = references(at(4, 3), false, asked)
            .iter()
            .map(place)
            .collect();
        found.sort();
        assert_eq!(
            found,
            [("harbour.comp.md".into(), 6, 42), ("log.md".into(), 4, 4),],
            "by its text any case, and not the log's own `[[#Harbour notes]]`"
        );
    });
}
