//! What tells whether a change moved marks beyond the lines it edited ([a document drawn in the editor], decision 4):
//! the document's elements that span lines (containers, fenced code blocks, tables, block quotes, setext headings,
//! the frontmatter), by their lines, and each table's columns by its header's pipes. The layout before a change,
//! moved by the lines the change adds or removes, is the layout after it unless the change moved what else is drawn:
//! a closer typed, a fence opened, a column widened. The server then asks the client to ask for the plan again
//! (`compmd/drawing/refresh`). It may ask more often than needed, never less.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::{Node, Root};
use lsp_types::TextDocumentContentChangeEvent;

use super::source::Source;
use super::table;
use crate::encoding::Encoding;

/// The lines of a document's elements that span lines, in the order they start.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Layout(Vec<Extent>);

/// An element's lines (from 0), and a table's header's pipes.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Extent {
    first: usize,
    last: usize,
    columns: Vec<usize>,
}

impl Layout {
    /// The layout of `tree`, the tree of `text`.
    pub(crate) fn of(tree: &Root, text: &str) -> Self {
        let source = Source::new(text, Encoding::Utf32);
        let mut extents = Vec::new();
        walk(&tree.children, &source, &mut extents);
        Self(extents)
    }

    /// The layout as it is once `change` moved its lines; `None` for a change of the whole text.
    pub(crate) fn moved(&self, change: &TextDocumentContentChangeEvent) -> Option<Self> {
        let range = change.range?;
        let (start, end) = (range.start.line as usize, range.end.line as usize);
        let added = change.text.matches('\n').count();
        let shift = |line: usize| (line + added).saturating_sub(end - start);
        let extents = self
            .0
            .iter()
            .map(|extent| Extent {
                first: if extent.first > end {
                    shift(extent.first)
                } else {
                    extent.first
                },
                last: if extent.last >= end {
                    shift(extent.last)
                } else {
                    extent.last
                },
                columns: extent.columns.clone(),
            })
            .collect();
        Some(Self(extents))
    }
}

fn walk(nodes: &[Node], source: &Source, extents: &mut Vec<Extent>) {
    for node in nodes {
        let lines = match node {
            Node::ComponentContainer(container) => container.position.map(|opener| {
                let closer = container.close_position.unwrap_or(opener);
                (opener.start, closer.end)
            }),
            Node::Code(_)
            | Node::Table(_)
            | Node::Blockquote(_)
            | Node::Heading(_)
            | Node::Yaml(_)
            | Node::Toml(_) => node.position().map(|at| (at.start, at.end)),
            _ => None,
        };
        if let Some((start, end)) = lines {
            let ((first, column), (last, _)) = (Source::place(start), Source::place(end));
            let columns = if matches!(node, Node::Table(_)) {
                table::pipes(source, (first, column))
            } else {
                Vec::new()
            };
            if last > first || !columns.is_empty() {
                extents.push(Extent {
                    first,
                    last,
                    columns,
                });
            }
        }
        walk(node.children(), source, extents);
        if let Node::ComponentContainer(container) = node {
            for slot in &container.slots {
                walk(&slot.children, source, extents);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use lsp_types::{Position, Range};

    use super::*;
    use crate::features::testing::processed;

    fn layout(text: &str) -> Layout {
        Layout::of(processed(text).tree(), text)
    }

    /// The change of the lines `start` to `end` (their starts) into `text`.
    fn change(start: u32, end: u32, text: &str) -> TextDocumentContentChangeEvent {
        TextDocumentContentChangeEvent {
            range: Some(Range::new(Position::new(start, 0), Position::new(end, 0))),
            range_length: None,
            text: text.into(),
        }
    }

    #[test]
    fn the_layout_is_the_lines_of_what_spans_lines_and_a_tables_columns() {
        let text = "---\nmd: {}\n---\n# One line\n\n| A | B |\n| - | - |\n\n::: box\n:: #s\n> q\n> r\n::: /box\n";
        let Layout(extents) = layout(text);
        let lines: Vec<(usize, usize)> = extents
            .iter()
            .map(|extent| (extent.first, extent.last))
            .collect();
        assert_eq!(lines, [(0, 2), (5, 6), (8, 12), (10, 11)]);
        assert_eq!(extents[1].columns, [0, 4, 8]);
    }

    #[test]
    fn a_change_inside_an_element_or_after_it_moves_it_as_expected() {
        let before = layout("::: box\nIn.\n::: /box\n\n```\nx\n```\n");
        let typed = change(1, 1, "More.\n");
        assert_eq!(
            before.moved(&typed),
            Some(layout("::: box\nMore.\nIn.\n::: /box\n\n```\nx\n```\n")),
            "a line added in the box"
        );
        let removed = change(1, 2, "");
        assert_eq!(
            before.moved(&removed),
            Some(layout("::: box\n::: /box\n\n```\nx\n```\n")),
            "a line removed"
        );
    }

    #[test]
    fn a_change_that_moves_what_else_is_drawn_is_another_layout() {
        let before = layout("::: box\nIn.\n::: /box\n");
        let opened = change(0, 0, "```\n");
        assert_ne!(
            before.moved(&opened),
            Some(layout("```\n::: box\nIn.\n::: /box\n"))
        );
        let whole = TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: String::new(),
        };
        assert_eq!(before.moved(&whole), None);
    }
}
