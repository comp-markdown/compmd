//! Document symbols ([the language server], *Blocks, symbols, folding*): the headings, containers, leaves, slots and
//! definitions of a document, nested as it nests them: a heading holds what follows it up to the next heading of its
//! level or above; a container its default slot and its named slots; a `compmd` definition its body.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#blocks-symbols-folding

use compmd::tree::{DefinitionBody, Node, Position};
use lsp_types::{DocumentSymbol, Range, SymbolKind};

use crate::encoding::Encoding;

/// The symbols of the blocks `nodes` of the document whose text is `text`.
pub(crate) fn symbols(nodes: &[Node], text: &str, encoding: Encoding) -> Vec<DocumentSymbol> {
    Outline { text, encoding }.level(nodes)
}

/// How a document's symbols are found.
struct Outline<'t> {
    text: &'t str,
    encoding: Encoding,
}

/// A heading not closed yet, with its depth.
struct Open {
    depth: u8,
    symbol: DocumentSymbol,
}

impl Outline<'_> {
    fn range(&self, position: &Position) -> Range {
        self.encoding.span(self.text, position)
    }

    /// The symbols of the sibling blocks `nodes`, each heading holding what follows it.
    fn level(&self, nodes: &[Node]) -> Vec<DocumentSymbol> {
        let mut done: Vec<DocumentSymbol> = Vec::new();
        let mut open: Vec<Open> = Vec::new();
        for node in nodes {
            if let Node::Heading(heading) = node
                && let Some(position) = heading.position
            {
                while open.last().is_some_and(|last| last.depth >= heading.depth) {
                    close(&mut open, &mut done);
                }
                let range = self.range(&position);
                open.push(Open {
                    depth: heading.depth,
                    symbol: symbol(
                        text_of(&heading.children),
                        None,
                        SymbolKind::STRING,
                        range,
                        range,
                        Vec::new(),
                    ),
                });
                continue;
            }
            let found = self.of(node);
            match open.last_mut() {
                Some(last) => add(&mut last.symbol, found),
                None => done.extend(found),
            }
        }
        while !open.is_empty() {
            close(&mut open, &mut done);
        }
        done
    }

    /// The symbols `node` is, or holds.
    fn of(&self, node: &Node) -> Vec<DocumentSymbol> {
        let Some(position) = node.position() else {
            return self.level(node.children());
        };
        let selection = self.range(&position);
        match node {
            Node::ComponentContainer(container) => {
                let mut children = self.level(&container.children);
                for slot in &container.slots {
                    let Some(at) = slot.position else { continue };
                    let range = self.range(&at);
                    let inside = self.level(&slot.children);
                    let end = inside.last().map_or(range.end, |last| last.range.end);
                    children.push(symbol(
                        format!("#{}", slot.name),
                        None,
                        SymbolKind::FIELD,
                        Range { end, ..range },
                        range,
                        inside,
                    ));
                }
                let end = container
                    .close_position
                    .map_or(selection.end, |close| self.range(&close).end);
                let range = Range {
                    start: selection.start,
                    end,
                };
                vec![symbol(
                    container.name.clone(),
                    Some("container"),
                    SymbolKind::OBJECT,
                    range,
                    selection,
                    children,
                )]
            }
            Node::ComponentLeaf(leaf) => {
                vec![symbol(
                    leaf.name.clone(),
                    Some("leaf"),
                    SymbolKind::OBJECT,
                    selection,
                    selection,
                    Vec::new(),
                )]
            }
            Node::ComponentDefinition(definition) => {
                let body = match &definition.body {
                    DefinitionBody::Compmd { children } => self.level(children),
                    DefinitionBody::Host { .. } => Vec::new(),
                };
                let detail = format!("definition, {}", definition.language);
                vec![symbol(
                    definition.name.clone(),
                    Some(&detail),
                    SymbolKind::CLASS,
                    selection,
                    selection,
                    body,
                )]
            }
            Node::ComponentSignature(signature) => {
                vec![symbol(
                    signature.name.clone(),
                    Some("signature"),
                    SymbolKind::INTERFACE,
                    selection,
                    selection,
                    Vec::new(),
                )]
            }
            _ => self.level(node.children()),
        }
    }
}

/// Closes the innermost open heading: into the one around it, or among `done`.
fn close(open: &mut Vec<Open>, done: &mut Vec<DocumentSymbol>) {
    let Some(Open { symbol, .. }) = open.pop() else {
        return;
    };
    match open.last_mut() {
        Some(parent) => add(&mut parent.symbol, vec![symbol]),
        None => done.push(symbol),
    }
}

/// `found` under `parent`, whose range then runs to the last of them.
fn add(parent: &mut DocumentSymbol, found: Vec<DocumentSymbol>) {
    if let Some(last) = found.last() {
        parent.range.end = parent.range.end.max(last.range.end);
    }
    parent.children.get_or_insert_with(Vec::new).extend(found);
}

#[expect(
    deprecated,
    reason = "`deprecated` is a field LSP keeps, which `tags` replaces"
)]
fn symbol(
    name: String,
    detail: Option<&str>,
    kind: SymbolKind,
    range: Range,
    selection_range: Range,
    children: Vec<DocumentSymbol>,
) -> DocumentSymbol {
    DocumentSymbol {
        name,
        detail: detail.map(str::to_owned),
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range,
        children: (!children.is_empty()).then_some(children),
    }
}

/// The text of inline nodes: what a heading says.
fn text_of(nodes: &[Node]) -> String {
    nodes
        .iter()
        .map(|node| match node {
            Node::Text(text) | Node::InlineCode(text) | Node::InlineMath(text) => {
                text.value.clone()
            }
            _ => text_of(node.children()),
        })
        .collect()
}

#[cfg(test)]
mod tests;
