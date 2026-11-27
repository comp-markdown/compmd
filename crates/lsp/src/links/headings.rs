//! A document's headings with their anchors ([the language server], *Links and headings*): the expanded tree's, each
//! at the place of the processed document that made it, so a link to `#harbour-notes` reaches it in a CompMD file and
//! in a plain `.md` file alike. The expansion keeps its origins for this (the `origins` feature).
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use compmd::Processor;
use compmd::expanded::{Content, Node};
use compmd::processor::Expanded;
use compmd::tree::Position;

/// A heading a link reaches by its anchor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Heading {
    /// The anchor (spec §12.3).
    pub(crate) anchor: String,
    /// Its level, 1 to 6, as expanded.
    pub(crate) depth: u8,
    /// Its text, expanded.
    pub(crate) text: String,
    /// The place of the processed document that made it.
    pub(crate) position: Position,
}

/// The headings of `processor`'s document, in their order; a heading without an origin (an expansion without the
/// `origins` feature) is left out.
pub(crate) fn headings(processor: &Processor<Expanded>) -> Vec<Heading> {
    let mut found = Vec::new();
    walk(&processor.expanded().children, &mut found);
    found
}

fn walk(nodes: &[Node], found: &mut Vec<Heading>) {
    for node in nodes {
        if let (Content::Heading(heading), Some(origin)) = (&node.content, &node.origin) {
            found.push(Heading {
                anchor: heading.anchor.clone(),
                depth: heading.depth,
                text: text_of(&heading.children),
                position: Position {
                    start: origin.start,
                    end: origin.end,
                },
            });
        }
        walk(node.content.children(), found);
        if let Content::ComponentContainer(container) = &node.content {
            for slot in &container.slots {
                walk(&slot.children, found);
            }
        }
    }
}

/// The text of `nodes`, as a heading's reads: its text and code, without markup.
fn text_of(nodes: &[Node]) -> String {
    nodes
        .iter()
        .map(|node| match &node.content {
            Content::Text(literal) | Content::InlineCode(literal) => literal.value.clone(),
            other => text_of(other.children()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::testing::processed_file;

    #[test]
    fn each_heading_has_its_anchor_text_and_place() {
        let text = "# Harbour *notes*\n\nText.\n\n## Tides `2026`\n\n## Tides 2026\n";
        let found = headings(&processed_file("notes.md", text));
        let summary: Vec<(&str, u8, &str, usize)> = found
            .iter()
            .map(|heading| {
                (
                    heading.anchor.as_str(),
                    heading.depth,
                    heading.text.as_str(),
                    heading.position.start.line,
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                ("harbour-notes", 1, "Harbour notes", 1),
                ("tides-2026", 2, "Tides 2026", 5),
                ("tides-2026-1", 2, "Tides 2026", 7),
            ]
        );
    }

    #[test]
    fn a_compmd_documents_headings_are_its_expanded_ones() {
        let text = "---\ntitle: Harbour\n---\n\n# {{md.doc.title}} notes\n\n::: box\n## In a box\n::: /box\n";
        let found = headings(&processed_file("notes.comp.md", text));
        let anchors: Vec<&str> = found
            .iter()
            .map(|heading| heading.anchor.as_str())
            .collect();
        assert_eq!(anchors, ["harbour-notes", "in-a-box"]);
        assert_eq!(found[1].position.start.line, 8);
    }
}
