//! What holds a place of a document's text: text whose component syntax counts, the frontmatter, or a block that holds
//! other text. Positional hints and syntax tokens both read a line again as the parser reads it, once they know it is
//! text.

use crate::tree::{Code, DefinitionBody, Node};

/// Whether a place is in text whose component syntax counts, in the frontmatter, or in a block that holds other
/// text: code, math, raw HTML, a signature, a definition's header or host-language code.
#[derive(Debug, PartialEq)]
pub(crate) enum Surrounding<'a> {
    Text,
    /// A YAML frontmatter.
    Frontmatter,
    /// A TOML frontmatter.
    Toml,
    /// A code block, which a code renderer may render (spec §7.7).
    Code(&'a Code),
    Opaque,
}

/// What surrounds code point `offset` of the blocks `nodes`. The blocks are in order, so the one a place is in is the
/// last that starts before it.
pub(crate) fn surrounding(nodes: &[Node], offset: usize) -> Surrounding<'_> {
    let starts_before = nodes.partition_point(|node| {
        node.position()
            .is_none_or(|position| position.start.offset <= offset)
    });
    let Some(block) = starts_before.checked_sub(1).map(|index| &nodes[index]) else {
        return Surrounding::Text;
    };
    let inside = block
        .position()
        .is_some_and(|position| position.start.offset <= offset && offset <= position.end.offset);
    match block {
        Node::Yaml(_) if inside => Surrounding::Frontmatter,
        Node::Toml(_) if inside => Surrounding::Toml,
        Node::Code(code) if inside => Surrounding::Code(code),
        Node::Math(_)
        | Node::Html(_)
        | Node::InlineCode(_)
        | Node::InlineMath(_)
        | Node::ComponentSignature(_)
            if inside =>
        {
            Surrounding::Opaque
        }
        Node::ComponentDefinition(definition) if inside => match &definition.body {
            DefinitionBody::Compmd { children }
                if children
                    .first()
                    .and_then(Node::position)
                    .is_some_and(|first| first.start.offset <= offset) =>
            {
                surrounding(children, offset)
            }
            _ => Surrounding::Opaque,
        },
        Node::ComponentContainer(container) => {
            let in_slot = container.slots.iter().rev().find(|slot| {
                slot.position
                    .is_some_and(|position| position.start.offset <= offset)
            });
            match in_slot {
                Some(slot) => surrounding(&slot.children, offset),
                None => surrounding(&container.children, offset),
            }
        }
        _ => surrounding(block.children(), offset),
    }
}

/// How many code points of `line` come before what a component line starts with: indentation, and the markers of
/// block quotes and list items.
pub(crate) fn indent(line: &[char]) -> usize {
    let space = |c: Option<&char>| matches!(c, Some(' ' | '\t'));
    let mut at = 0;
    loop {
        while space(line.get(at)) {
            at += 1;
        }
        let digits = line[at..].iter().take_while(|c| c.is_ascii_digit()).count();
        match line.get(at) {
            Some('>') => at += 1,
            Some('-' | '*' | '+') if space(line.get(at + 1)) => at += 1,
            Some(_)
                if digits > 0
                    && matches!(line.get(at + digits), Some('.' | ')'))
                    && space(line.get(at + digits + 1)) =>
            {
                at += digits + 1;
            }
            _ => return at,
        }
    }
}
