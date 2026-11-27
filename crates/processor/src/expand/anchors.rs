//! Heading anchors (spec §12.3): the name a link reaches each heading of an expanded root by, GitHub's for the same
//! text. A heading's anchor is its expanded text as a reader sees it, in lower case, with every character but a
//! letter, a mark, a number, `-`, `_` or a space dropped, and each space written `-`; a repeat in the same root gets
//! the first of `-1`, `-2`, … that no heading before it has. An example's root (spec §7.8) is numbered on its own:
//! the walk doesn't enter it.

use super::text::plain_text;
use crate::expanded::{Content, Node};
use icu::properties::CodePointMapData;
use icu::properties::props::{GeneralCategory, GeneralCategoryGroup};
use std::collections::{HashMap, HashSet};

/// Gives every heading of a root's `children` its anchor, in the tree's order; the index of the last of `children`
/// whose anchors that changed, if any.
pub(super) fn give(children: &mut [Node]) -> Option<usize> {
    let mut taken = Taken::default();
    let mut changed = None;
    for (index, node) in children.iter_mut().enumerate() {
        if give_within(node, &mut taken) {
            changed = Some(index);
        }
    }
    changed
}

/// Gives the headings of `node` and of what it holds their anchors, none of `taken`, adding each to it; whether one
/// changed.
fn give_within(node: &mut Node, taken: &mut Taken) -> bool {
    let mut changed = false;
    if let Content::Heading(heading) = &mut node.content {
        let anchor = taken.anchor(slug(&plain_text(&heading.children)));
        changed = heading.anchor != anchor;
        heading.anchor = anchor;
    }
    for list in node.content.lists_mut() {
        for child in list {
            changed |= give_within(child, taken);
        }
    }
    changed
}

/// The anchors a root's headings have so far.
#[derive(Default)]
struct Taken {
    anchors: HashSet<String>,
    /// For each slug that repeats, the number its next repeat tries first: every number below it is taken, so many
    /// headings of the same text don't try them all again.
    next: HashMap<String, usize>,
}

impl Taken {
    /// `slug`, or for one a heading before has, the first of `slug-1`, `slug-2`, … that none has; taken from then on.
    fn anchor(&mut self, slug: String) -> String {
        let anchor = if self.anchors.contains(&slug) {
            let mut number = self.next.get(&slug).copied().unwrap_or(1);
            while self.anchors.contains(&format!("{slug}-{number}")) {
                number += 1;
            }
            self.next.insert(slug.clone(), number + 1);
            format!("{slug}-{number}")
        } else {
            slug
        };
        self.anchors.insert(anchor.clone());
        anchor
    }
}

/// `text` as GitHub writes a heading's anchor: lower-cased, with every character but a letter, a mark, a number, `-`,
/// `_` or a space dropped, and each space written `-`.
fn slug(text: &str) -> String {
    let categories = CodePointMapData::<GeneralCategory>::new();
    let kept = GeneralCategoryGroup::Letter
        .union(GeneralCategoryGroup::Mark)
        .union(GeneralCategoryGroup::Number);
    text.to_lowercase()
        .chars()
        .filter_map(|character| match character {
            ' ' => Some('-'),
            '-' | '_' => Some(character),
            _ => kept
                .contains(categories.get(character))
                .then_some(character),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expanded::{self, Container};
    use crate::tree::{Literal, Parent};
    use proptest::prelude::*;

    fn node(content: Content) -> Node {
        Node::new(content)
    }

    fn text(value: &str) -> Node {
        node(Content::Text(Literal {
            value: value.into(),
            position: None,
        }))
    }

    fn heading(children: Vec<Node>) -> Node {
        node(Content::Heading(expanded::Heading {
            depth: 2,
            anchor: String::new(),
            children,
        }))
    }

    fn anchors(nodes: &[Node]) -> Vec<String> {
        let mut found = Vec::new();
        for node in nodes {
            if let Content::Heading(heading) = &node.content {
                found.push(heading.anchor.clone());
            }
            for list in node.content.lists() {
                found.extend(anchors(list));
            }
        }
        found
    }

    #[test]
    fn punctuation_is_dropped_and_spaces_are_hyphens() {
        assert_eq!(slug("Harbour notes, 2026!"), "harbour-notes-2026");
        assert_eq!(slug("Café & bar"), "café--bar");
        assert_eq!(slug("What's new? (v2.0)"), "whats-new-v20");
        assert_eq!(slug("  two  spaces "), "--two--spaces-");
    }

    #[test]
    fn hyphens_and_underscores_are_kept() {
        assert_eq!(
            slug("snake_case and kebab-case"),
            "snake_case-and-kebab-case"
        );
        assert_eq!(
            slug("a—b–c"),
            "abc",
            "dashes other than `-` are punctuation"
        );
    }

    #[test]
    fn letters_marks_and_numbers_of_every_script_are_kept() {
        assert_eq!(slug("Über Größe"), "über-größe");
        assert_eq!(
            slug("Cafe\u{301}"),
            "cafe\u{301}",
            "a combining accent is a mark"
        );
        assert_eq!(slug("日本語の見出し"), "日本語の見出し");
        assert_eq!(slug("Ελληνικά ΚΕΦΑΛΙΔΑ"), "ελληνικά-κεφαλιδα");
        assert_eq!(slug("हिन्दी"), "हिन्दी");
        assert_eq!(slug("Chapter Ⅻ ½ ٣"), "chapter-ⅻ-½-٣");
    }

    #[test]
    fn emoji_and_symbols_are_dropped() {
        assert_eq!(slug("Launch 🚀 day"), "launch--day");
        assert_eq!(slug("€5 + $3 = ⓐ"), "5--3--");
        assert_eq!(slug("🎉"), "");
    }

    #[test]
    fn repeats_get_the_first_free_number() {
        let mut taken = Taken::default();
        let slugs = [
            "notes", "notes", "notes-1", "notes", "notes-3", "notes", "", "",
        ];
        let given: Vec<String> = slugs
            .into_iter()
            .map(|slug| taken.anchor(slug.to_string()))
            .collect();
        let expected = [
            "notes",
            "notes-1",
            "notes-1-1",
            "notes-2",
            "notes-3",
            "notes-4",
            "",
            "-1",
        ];
        assert_eq!(given, expected);
    }

    #[test]
    fn headings_are_numbered_in_the_tree_s_order() {
        let boxed = node(Content::ComponentContainer(Container {
            name: "box".into(),
            children: vec![heading(vec![text("Notes")])],
            ..Container::default()
        }));
        let mut children = vec![
            heading(vec![text("Notes")]),
            boxed,
            node(Content::Paragraph(Parent {
                children: vec![text("Notes")],
                position: None,
            })),
            heading(vec![text("Notes")]),
        ];
        assert_eq!(give(&mut children), Some(3));
        assert_eq!(anchors(&children), ["notes", "notes-1", "notes-2"]);
        assert_eq!(give(&mut children), None, "nothing changes a second time");
    }

    #[test]
    fn a_heading_reads_its_text_and_code_but_not_images_or_html() {
        let image = node(Content::Image(crate::tree::Image {
            alt: "Logo".into(),
            ..crate::tree::Image::default()
        }));
        let code = node(Content::InlineCode(Literal {
            value: "md.doc".into(),
            position: None,
        }));
        let html = node(Content::Html(Literal {
            value: "<br>".into(),
            position: None,
        }));
        let emphasis = node(Content::Emphasis(Parent {
            children: vec![text(" values")],
            position: None,
        }));
        let mut children = vec![heading(vec![image, text("The "), code, html, emphasis])];
        give(&mut children);
        assert_eq!(anchors(&children), ["the-mddoc-values"]);
    }

    #[test]
    fn a_wiki_link_reads_as_its_label() {
        let link = node(Content::WikiLink(crate::tree::WikiLink {
            target: "harbour-guide".into(),
            label: "The guide".into(),
            position: None,
        }));
        let mut children = vec![heading(vec![text("See "), link])];
        give(&mut children);
        assert_eq!(anchors(&children), ["see-the-guide"]);
    }

    #[test]
    fn an_example_s_headings_are_its_own() {
        let example = node(Content::Code(expanded::Code {
            document: Some(Box::new(expanded::Root {
                children: vec![heading(vec![text("Notes")])],
                ..expanded::Root::default()
            })),
            ..expanded::Code::default()
        }));
        let mut children = vec![example, heading(vec![text("Notes")])];
        give(&mut children);
        assert_eq!(anchors(&children), ["notes"]);
    }

    proptest! {
        #[test]
        fn an_anchor_has_no_space_and_no_upper_case(text in "\\PC*") {
            let anchor = slug(&text);
            prop_assert!(!anchor.contains(char::is_whitespace));
            prop_assert_eq!(anchor.to_lowercase(), anchor.clone());
            prop_assert_eq!(slug(&anchor), anchor, "an anchor is its own slug");
        }

        #[test]
        fn a_repeat_gets_the_first_number_no_anchor_has(slugs in prop::collection::vec("a|a-1|a-2|a-1-1|b", 0..16)) {
            let mut taken = Taken::default();
            let mut seen: HashSet<String> = HashSet::new();
            for slug in slugs {
                let first_free = (1..=seen.len() + 1)
                    .map(|number| format!("{slug}-{number}"))
                    .find(|anchor| !seen.contains(anchor))
                    .unwrap();
                let expected = if seen.contains(&slug) { first_free } else { slug.clone() };
                seen.insert(expected.clone());
                prop_assert_eq!(taken.anchor(slug), expected);
            }
        }

        #[test]
        fn anchors_in_a_root_are_unique(texts in prop::collection::vec("[a-c -]{0,3}", 0..12)) {
            let mut children: Vec<Node> = texts.iter().map(|text_| heading(vec![text(text_)])).collect();
            give(&mut children);
            let given = anchors(&children);
            let distinct: HashSet<&String> = given.iter().collect();
            prop_assert_eq!(distinct.len(), given.len());
        }
    }
}
