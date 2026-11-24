//! How deep blocks nest (spec §8.5): quotes, list items and footnote definitions nested deeper than [`NESTING`] are
//! cut from the fork's mdast as soon as it is parsed, each the paragraph of its text, before anything walks it; and
//! the converter cuts blocks the same way where component containers make them too deep. Each cut is F038, at the
//! block that crossed the limit. Nothing here recurses, so a document nested thousands deep is refused, not a stack
//! overflow.

use crate::nesting::NESTING;
use markdown::mdast;
use std::mem;

/// Whether `node` nests the blocks it holds one level deeper.
pub(super) fn nests(node: &mdast::Node) -> bool {
    matches!(
        node,
        mdast::Node::Blockquote(_) | mdast::Node::ListItem(_) | mdast::Node::FootnoteDefinition(_)
    )
}

/// Whether `node`, among blocks nested `depth` deep, crosses the limit: it, or for a list its items, would be nested
/// deeper than [`NESTING`]. A list is cut whole, since its items can't be paragraphs.
pub(super) fn too_deep(node: &mdast::Node, depth: usize) -> bool {
    (nests(node) || matches!(node, mdast::Node::List(_))) && depth >= NESTING
}

/// Cuts each block of `children`, a root's, that [`too_deep`] crosses the limit: it becomes the paragraph of its text.
/// Returns where each cut block starts, in bytes.
pub(super) fn cut_deep(children: &mut Vec<mdast::Node>) -> Vec<usize> {
    let mut cut = Vec::new();
    let mut parents = vec![(children, 0)];
    while let Some((nodes, depth)) = parents.pop() {
        for node in nodes {
            if too_deep(node, depth) {
                cut.extend(node.position().map(|position| position.start.offset));
                *node = plain(mem::replace(node, empty()));
            } else {
                let nested = depth + usize::from(nests(node));
                if let Some(children) = node.children_mut() {
                    parents.push((children, nested));
                }
            }
        }
    }
    cut.sort_unstable();
    cut
}

/// `node` as the paragraph of its text, at its place, taken apart without recursion.
pub(super) fn plain(node: mdast::Node) -> mdast::Node {
    let position = node.position().cloned();
    let mut text = String::new();
    let mut pending = vec![node];
    while let Some(mut node) = pending.pop() {
        if let Some(value) = text_of(&node) {
            text.push_str(value);
        } else if matches!(node, mdast::Node::Paragraph(_) | mdast::Node::Heading(_))
            && !text.is_empty()
        {
            text.push(' ');
        }
        if let Some(children) = node.children_mut() {
            pending.extend(mem::take(children).into_iter().rev());
        }
    }
    mdast::Node::Paragraph(mdast::Paragraph {
        children: vec![mdast::Node::Text(mdast::Text {
            value: text.trim().to_string(),
            position: position.clone(),
        })],
        position,
    })
}

/// The text a leaf holds, as it is written.
fn text_of(node: &mdast::Node) -> Option<&str> {
    match node {
        mdast::Node::Text(mdast::Text { value, .. })
        | mdast::Node::InlineCode(mdast::InlineCode { value, .. })
        | mdast::Node::Code(mdast::Code { value, .. })
        | mdast::Node::Math(mdast::Math { value, .. })
        | mdast::Node::InlineMath(mdast::InlineMath { value, .. })
        | mdast::Node::Html(mdast::Html { value, .. })
        | mdast::Node::CompmdRaw(mdast::CompmdRaw { value, .. }) => Some(value),
        _ => None,
    }
}

/// A node with nothing in it, in place of one being taken apart.
fn empty() -> mdast::Node {
    mdast::Node::Break(mdast::Break { position: None })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(text: &str) -> Vec<mdast::Node> {
        match markdown::to_mdast(text, &markdown::ParseOptions::gfm()) {
            Ok(mdast::Node::Root(root)) => root.children,
            _ => Vec::new(),
        }
    }

    /// How deep the quotes of `nodes` nest.
    fn depth(nodes: &[mdast::Node]) -> usize {
        let mut deepest = 0;
        let mut pending: Vec<(&mdast::Node, usize)> = nodes.iter().map(|node| (node, 0)).collect();
        while let Some((node, depth)) = pending.pop() {
            let nested = depth + usize::from(nests(node));
            deepest = deepest.max(nested);
            pending.extend(
                node.children()
                    .into_iter()
                    .flatten()
                    .map(|child| (child, nested)),
            );
        }
        deepest
    }

    #[test]
    fn blocks_past_the_limit_become_their_text() {
        let mut nodes = parsed(&format!(
            "{}Deep *text*.\n\nAfter.\n",
            "> ".repeat(NESTING + 2)
        ));
        let cut = cut_deep(&mut nodes);
        assert_eq!(cut.len(), 1, "one cut, at the block that crossed the limit");
        assert_eq!(depth(&nodes), NESTING);
        let mut innermost = &nodes[0];
        while let Some([child]) = innermost.children().map(Vec::as_slice) {
            innermost = child;
        }
        assert_eq!(innermost.to_string(), "Deep text.");
        // At the limit, nothing is cut.
        let mut nodes = parsed(&format!("{}At the limit.\n", "> ".repeat(NESTING)));
        assert_eq!(cut_deep(&mut nodes), Vec::<usize>::new());
        assert_eq!(depth(&nodes), NESTING);
    }

    #[test]
    fn a_block_s_text_keeps_its_paragraphs_apart() {
        let node = parsed("> One `code`.\n>\n> # Two\n").remove(0);
        assert_eq!(plain(node).to_string(), "One code. Two");
    }
}
