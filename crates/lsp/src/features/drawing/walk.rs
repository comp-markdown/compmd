//! The walk over the processor's tree that draws each element decision 11 names, when its switch is on, and what it
//! holds. Images, math, HTML and component definitions are left as written.

use compmd::tree::Node;

use super::planner::Planner;
use super::{
    code_block, container, frontmatter, heading, inline_code, leaf, list, quote, rule, table,
};

/// How deep an element is: in how many bulleted lists, and in how many block quotes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Nesting {
    pub(crate) lists: usize,
    pub(crate) quotes: usize,
}

/// Draws the elements `nodes`, and what they hold, at `nesting`.
pub(crate) fn draw(nodes: &[Node], planner: &mut Planner, nesting: Nesting) {
    for node in nodes {
        draw_node(node, planner, nesting);
    }
}

fn draw_node(node: &Node, planner: &mut Planner, nesting: Nesting) {
    let settings = planner.settings();
    match node {
        Node::Heading(heading) if settings.heading.on => heading::draw(heading, planner),
        Node::Code(code) if settings.code.on => code_block::draw(code, planner),
        Node::InlineCode(code) if settings.code.on => inline_code::draw(code, planner),
        Node::ThematicBreak(rule) if settings.rule.on => rule::draw(rule, planner),
        Node::List(list) => return list::draw(list, planner, nesting),
        Node::Blockquote(quote) => return quote::draw(quote, planner, nesting),
        Node::Table(table) if settings.table.on => table::draw(table, planner),
        Node::ComponentContainer(container) => {
            if settings.container.on {
                container::draw(container, planner);
            }
            draw(&container.children, planner, nesting);
            for slot in &container.slots {
                draw(&slot.children, planner, nesting);
            }
            return;
        }
        Node::ComponentLeaf(leaf) => leaf::draw(leaf, planner),
        Node::Yaml(frontmatter) | Node::Toml(frontmatter) if settings.param.on => {
            frontmatter::draw(frontmatter, planner);
        }
        Node::ComponentDefinition(_) | Node::ComponentSignature(_) => return,
        _ => {}
    }
    draw(node.children(), planner, nesting);
}
