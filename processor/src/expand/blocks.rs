//! The expanded tree block by block: what each top-level block of the processed document gave, so that an edit
//! expands again only the blocks it changes, and the blocks after them that then give something else (spec §10.5).

use super::env::Env;
use super::{Expanded, Expander, Options, Replaced, Report, expand, layout, origins, text};
use crate::context::RenderContext;
use crate::diagnostic::Diagnostic;
use crate::expanded::{self, Content, Root};
use crate::load::Loaded;
use crate::parse::Spliced;
use crate::resolve::{DocumentId, Resolved};
use crate::tree::{Moved, Node};
use crate::validate::Validated;
use std::collections::HashSet;

/// What a top-level block of the processed document gave, and how that depends on what came before it.
#[derive(Clone, Debug)]
pub(crate) struct Block {
    /// How many top-level nodes of the expanded tree it gave.
    nodes: usize,
    /// What expanding was at when it started, and when it ended.
    start: State,
    end: State,
    /// Whether what it gives depends on the depth of the heading before it, which a `compmd` component with relative
    /// headings follows.
    reads_heading: bool,
    /// Whether it leaves the depth of the last heading as it found it.
    keeps_heading: bool,
    /// What evaluating it reported, in order.
    reports: Vec<Report>,
    /// The diagnostics of the layout checks of what it gave.
    layout: Vec<Diagnostic>,
}

impl Block {
    /// How many nodes it made, against the limit.
    fn made(&self) -> usize {
        self.end.made - self.start.made
    }

    /// Whether it gives the same when it starts at `state`: it doesn't cross the limit, and it doesn't read the depth of
    /// the heading, or that is the same.
    fn gives_the_same(&self, state: State, limit: usize) -> bool {
        state.crossed.is_none()
            && self.end.crossed.is_none()
            && state.made + self.made() <= limit
            && (!self.reads_heading || state.heading == self.start.heading)
    }

    /// It, starting at `state` and giving the same.
    fn started_at(&mut self, state: State) {
        let made = self.made();
        self.end = State {
            heading: if self.keeps_heading {
                state.heading
            } else {
                self.end.heading
            },
            made: state.made + made,
            crossed: None,
        };
        self.start = state;
    }

    /// Its diagnostics moved, as an edit before it moved the processed document, the document `main`.
    fn shift(&mut self, moved: Moved, main: DocumentId) {
        for report in &mut self.reports {
            moved.diagnostic(&mut report.diagnostic);
            if let Some((_, document, at)) = &mut report.once
                && *document == main
            {
                moved.location(at);
            }
        }
        for diagnostic in &mut self.layout {
            moved.diagnostic(diagnostic);
        }
    }
}

/// Where expanding is between two top-level blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct State {
    /// The depth of the last heading expanded.
    heading: u8,
    /// The nodes made so far, against the limit.
    made: usize,
    /// Once the limit is crossed, how many expansions were under way then.
    crossed: Option<usize>,
}

impl State {
    const START: Self = Self {
        heading: 0,
        made: 0,
        crossed: None,
    };
}

impl<'r> Expander<'r> {
    /// Where expanding is, between two top-level blocks.
    fn state(&self) -> State {
        State {
            heading: self.last_heading,
            made: self.made,
            crossed: self.crossed,
        }
    }

    /// Goes on from `state`.
    fn resume(&mut self, state: State) {
        self.last_heading = state.heading;
        self.made = state.made;
        self.crossed = state.crossed;
    }

    /// A top-level block of the processed document expanded, its adjacent text joined, its origins finished and its
    /// layout checked; with what it gave.
    pub(super) fn block(&mut self, block: &'r Node, env: &Env<'r>) -> (Vec<expanded::Node>, Block) {
        let start = self.state();
        self.heading_kept = true;
        self.heading_read = false;
        let mut nodes = Vec::new();
        self.node(block, env, &mut nodes);
        let mut nodes = text::joined(nodes);
        origins::finish(&mut nodes);
        let layout = layout::check(&nodes);
        if !self.options.origins {
            origins::strip(&mut nodes);
        }
        let given = Block {
            nodes: nodes.len(),
            start,
            end: self.state(),
            reads_heading: self.heading_read,
            keeps_heading: self.heading_kept,
            reports: std::mem::take(&mut self.reports),
            layout,
        };
        (nodes, given)
    }
}

/// The expanded tree of `children`, which `blocks` gave, with `loaded`'s inputs, after an expansion that replaced
/// `replaced`. Text that two blocks gave next to each other is joined, as a list joins it (spec §8.1); an edit can't
/// take the blocks apart after that, so it expands the whole document again.
pub(super) fn finished(
    mut children: Vec<expanded::Node>,
    blocks: Vec<Block>,
    loaded: &Loaded,
    replaced: Option<Replaced>,
) -> Expanded {
    let diagnostics = diagnostics(&blocks);
    let joined = joins(&children, &blocks);
    if joined {
        children = text::joined(children);
    }
    Expanded {
        root: Root {
            children,
            inputs: loaded.inputs.clone(),
        },
        diagnostics,
        blocks: (!joined).then_some(blocks),
        replaced: replaced.filter(|_| !joined),
    }
}

/// The expanded tree after an edit that changed the processed document's top-level blocks `spliced.blocks`, from
/// `previous`, the expanded tree before it. Those blocks are expanded again, and so is each block after them that
/// gives something else after them: one that follows the depth of the heading before it, when that changed, or one
/// that crosses the limit. The other blocks keep what they gave, moved.
pub(crate) fn splice(
    previous: Expanded,
    resolved: &Resolved,
    validated: &Validated,
    loaded: &Loaded,
    context: &RenderContext,
    options: &Options,
    spliced: &Spliced,
) -> Expanded {
    let Some(mut blocks) = previous.blocks else {
        return expand(resolved, validated, loaded, context, options);
    };
    let mut children = previous.root.children;
    let (main, moved) = (resolved.main, spliced.moved);
    let start = spliced.blocks.start;
    let nodes = |blocks: &[Block]| blocks.iter().map(|block| block.nodes).sum::<usize>();
    let first = nodes(&blocks[..start]);
    let removed = first..first + nodes(&blocks[spliced.blocks.clone()]);
    for block in &mut blocks[spliced.blocks.end..] {
        block.shift(moved, main);
    }
    for block in &mut blocks[..start] {
        block.shift(moved, main);
    }
    if options.origins {
        origins::shift(&mut children[removed.end..], moved);
    }
    let old_length = children.len();
    let mut expander = Expander::new(resolved, validated, loaded, context, options);
    let env = Env::document(&expander, main);
    let tree = &resolved.document(main).root.children;
    expander.resume(
        blocks[..start]
            .last()
            .map_or(State::START, |block| block.end),
    );
    let mut given = Vec::new();
    let mut records = Vec::new();
    for block in &tree[start..start + spliced.inserted] {
        let (nodes, record) = expander.block(block, &env);
        given.extend(nodes);
        records.push(record);
    }
    let mut end = first + given.len();
    children.splice(removed, given);
    blocks.splice(spliced.blocks.clone(), records);
    let mut state = expander.state();
    let mut at = end;
    for (index, block) in blocks.iter_mut().enumerate().skip(start + spliced.inserted) {
        if block.gives_the_same(state, options.limit) {
            block.started_at(state);
        } else {
            expander.resume(state);
            let (nodes, record) = expander.block(&tree[index], &env);
            let count = nodes.len();
            children.splice(at..at + block.nodes, nodes);
            *block = record;
            end = at + count;
        }
        state = block.end;
        at += block.nodes;
    }
    let replaced = Replaced {
        nodes: first..end + old_length - children.len(),
        inserted: end - first,
        by: moved.by,
    };
    finished(children, blocks, loaded, Some(replaced))
}

/// The diagnostics of stage 4b, from what the blocks gave: what evaluating reported, each once that is reported once,
/// then the layout checks.
fn diagnostics(blocks: &[Block]) -> Vec<Diagnostic> {
    let mut reported = HashSet::new();
    let reports = blocks.iter().flat_map(|block| &block.reports);
    let mut found: Vec<Diagnostic> = reports
        .filter(|report| report.once.is_none_or(|once| reported.insert(once)))
        .map(|report| report.diagnostic.clone())
        .collect();
    found.extend(blocks.iter().flat_map(|block| block.layout.iter().cloned()));
    found
}

/// Whether two blocks gave text next to each other, which a list joins (spec §8.1).
fn joins(children: &[expanded::Node], blocks: &[Block]) -> bool {
    let is_text = |index: usize| {
        children
            .get(index)
            .is_some_and(|node| matches!(node.content, Content::Text(_)))
    };
    blocks
        .iter()
        .scan(0, |at, block| {
            *at += block.nodes;
            Some(*at)
        })
        .any(|at| at > 0 && is_text(at - 1) && is_text(at))
}

#[cfg(test)]
mod tests {
    use super::super::literal;
    use super::*;
    use crate::tree::{self, Move};

    fn text(value: &str) -> expanded::Node {
        expanded::Node {
            content: Content::Text(literal(value)),
            origin: None,
        }
    }

    /// A block that gave `nodes` nodes.
    fn block(nodes: usize) -> Block {
        Block {
            nodes,
            start: State::START,
            end: State::START,
            reads_heading: false,
            keeps_heading: true,
            reports: Vec::new(),
            layout: Vec::new(),
        }
    }

    #[test]
    fn text_two_blocks_give_next_to_each_other_is_joined() {
        let replaced = Replaced {
            nodes: 0..1,
            inserted: 1,
            by: Move::default(),
        };
        let loaded = Loaded::default();
        let joined = finished(
            vec![text("a"), text("b")],
            vec![block(1), block(0), block(1)],
            &loaded,
            Some(replaced.clone()),
        );
        assert_eq!(joined.root.children.len(), 1);
        assert!(joined.blocks.is_none(), "the blocks can't be taken apart");
        assert_eq!(joined.replaced, None, "it replaced the whole tree");
        let paragraph = expanded::Node {
            content: Content::Paragraph(tree::Parent {
                children: Vec::new(),
                position: None,
            }),
            origin: None,
        };
        let apart = finished(
            vec![text("a"), paragraph],
            vec![block(1), block(1)],
            &loaded,
            Some(replaced.clone()),
        );
        assert_eq!(apart.root.children.len(), 2);
        assert!(apart.blocks.is_some());
        assert_eq!(apart.replaced, Some(replaced));
    }
}
