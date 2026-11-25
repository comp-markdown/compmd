//! Folding ranges ([the language server], *Blocks, symbols, folding*): the frontmatter, each container from its opener
//! to its closer, each definition and signature fence, and each fenced code block, the rendered ones among them.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#blocks-symbols-folding

use compmd::tree::{Node, Position};
use lsp_types::FoldingRange;

/// The folding ranges of the blocks `nodes`, by their lines.
pub(crate) fn folding(nodes: &[Node]) -> Vec<FoldingRange> {
    let mut ranges = Vec::new();
    walk(nodes, &mut ranges);
    ranges
}

fn walk(nodes: &[Node], ranges: &mut Vec<FoldingRange>) {
    for node in nodes {
        let folded = match node {
            Node::ComponentContainer(container) => container.position.map(|opener| {
                let end = container
                    .close_position
                    .map_or(opener.end, |closer| closer.end);
                Position { end, ..opener }
            }),
            Node::Yaml(_)
            | Node::Toml(_)
            | Node::ComponentDefinition(_)
            | Node::ComponentSignature(_)
            | Node::Code(_) => node.position(),
            _ => None,
        };
        if let Some(at) = folded
            && at.end.line > at.start.line
        {
            ranges.push(FoldingRange {
                start_line: line(at.start.line),
                end_line: line(at.end.line),
                ..FoldingRange::default()
            });
        }
        // The default slot, then the named ones, in source order.
        walk(node.children(), ranges);
        if let Node::ComponentContainer(container) = node {
            for slot in &container.slots {
                walk(&slot.children, ranges);
            }
        }
    }
}

/// The processor's line, from 1, as LSP's, from 0.
fn line(line: usize) -> u32 {
    u32::try_from(line.saturating_sub(1)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
