//! The links a document's parse tree holds ([the language server], *Links and headings*): inline links and images
//! with their destination, references with their label, reference definitions with both, and wiki links with their
//! target, where the document has them.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use compmd::tree::{Node, Position};

/// What a link is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LinkKind {
    /// `[text](url)` or `![alt](url)`.
    Inline { url: String },
    /// `[text][label]`, `[label][]`, `[label]`, or an image's: by its normalized label.
    Reference { identifier: String },
    /// `[label]: url`.
    Definition { identifier: String, url: String },
    /// `[[target]]` or `[[target|label]]`: by its target, `Page#Heading`.
    Wiki { target: String },
}

/// A link of a document, where it is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentLink {
    pub(crate) kind: LinkKind,
    pub(crate) position: Position,
}

impl DocumentLink {
    /// The destination it writes: an inline link's or a definition's.
    pub(crate) fn url(&self) -> Option<&str> {
        match &self.kind {
            LinkKind::Inline { url } | LinkKind::Definition { url, .. } => Some(url),
            LinkKind::Reference { .. } | LinkKind::Wiki { .. } => None,
        }
    }
}

/// The links among `nodes` and inside them, in the document's order.
pub(crate) fn document_links(nodes: &[Node]) -> Vec<DocumentLink> {
    let mut found = Vec::new();
    walk(nodes, &mut found);
    found
}

fn walk(nodes: &[Node], found: &mut Vec<DocumentLink>) {
    for node in nodes {
        let kind = match node {
            Node::Link(link) => Some(LinkKind::Inline {
                url: link.url.clone(),
            }),
            Node::Image(image) => Some(LinkKind::Inline {
                url: image.url.clone(),
            }),
            Node::LinkReference(reference) => Some(LinkKind::Reference {
                identifier: reference.identifier.clone(),
            }),
            Node::ImageReference(reference) => Some(LinkKind::Reference {
                identifier: reference.identifier.clone(),
            }),
            Node::Definition(definition) => Some(LinkKind::Definition {
                identifier: definition.identifier.clone(),
                url: definition.url.clone(),
            }),
            Node::WikiLink(link) => Some(LinkKind::Wiki {
                target: link.target.clone(),
            }),
            _ => None,
        };
        if let (Some(kind), Some(position)) = (kind, node.position()) {
            found.push(DocumentLink { kind, position });
        }
        walk(node.children(), found);
        if let Node::ComponentContainer(container) = node {
            for slot in &container.slots {
                walk(&slot.children, found);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::testing::processed_file;

    #[test]
    fn links_images_references_and_definitions_in_order() {
        let text = "See [the tides](tides.md#high) and ![a map](map.png).\n\n- [the tables][tables]\n\n[tables]: ./tables.md\n";
        let processor = processed_file("notes.md", text);
        let found = document_links(&processor.tree().children);
        let kinds: Vec<&LinkKind> = found.iter().map(|link| &link.kind).collect();
        assert_eq!(
            kinds,
            [
                &LinkKind::Inline {
                    url: "tides.md#high".into()
                },
                &LinkKind::Inline {
                    url: "map.png".into()
                },
                &LinkKind::Reference {
                    identifier: "tables".into()
                },
                &LinkKind::Definition {
                    identifier: "tables".into(),
                    url: "./tables.md".into()
                },
            ]
        );
        assert_eq!(
            (found[0].position.start.line, found[0].position.start.column),
            (1, 5)
        );
        assert_eq!(found[2].url(), None);
        assert_eq!(found[3].url(), Some("./tables.md"));
    }

    #[test]
    fn links_inside_a_containers_slots() {
        let text = "::: card\n[a](a.md)\n\n:: #footer\n[b](b.md)\n::: /card\n";
        let processor = processed_file("notes.comp.md", text);
        let found = document_links(&processor.tree().children);
        let urls: Vec<Option<&str>> = found.iter().map(DocumentLink::url).collect();
        assert_eq!(urls, [Some("a.md"), Some("b.md")]);
    }

    #[test]
    fn wiki_links_where_the_document_has_them() {
        let text = "---\nmd:\n  syntax: [wiki-links]\n---\nSee [[Tides#High water|the tides]].\n";
        let found = document_links(&processed_file("notes.comp.md", text).tree().children);
        assert_eq!(
            found[0].kind,
            LinkKind::Wiki {
                target: "Tides#High water".into()
            }
        );
        assert_eq!(found[0].url(), None);
        let plain = document_links(&processed_file("notes.md", "[[Tides]]\n").tree().children);
        assert_eq!(plain, [], "a `.md` file without them");
    }
}
