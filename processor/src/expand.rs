//! Stage 4b, evaluating (spec §8, §9, §11, §12.3): the expanded tree. Every `compmd` definition, `if`, `each`,
//! `value` and interpolation is replaced by what it produces; `box`, the input components and host-language
//! components stay as nodes, their params evaluated. Then the layout checks (W114, W119) run on the result.
//!
//! What validating refused is treated as absent (spec §10.3), and not reported again: a refused param is not given,
//! an unknown path is shown as written, a component that can't render renders its fallback.

mod components;
mod env;
mod flow;
mod layout;
mod origins;
mod params;
mod text;

use crate::cel::{Error, Program, Value, Values};
use crate::context::RenderContext;
use crate::diagnostic::{Code, Diagnostic, Location, Stage};
use crate::expanded::{self, Content, Origin, Root};
use crate::lines::Lines;
use crate::load::Loaded;
use crate::md::Md;
use crate::parse::Spliced;
use crate::resolve::{DocumentId, Renderer, Resolved};
use crate::tree::{self, Move, Moved, Node};
use crate::validate::Validated;
use crate::validate::scope::Scope;
use env::Env;
use std::collections::{HashMap, HashSet};
use std::ops;
use std::rc::Rc;

/// The size of the expanded tree every processor renders (spec §8.5).
pub const LIMIT: usize = 100_000;

/// What expanding needs besides the document, its data and the render context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// Whether every expanded node gets its `origin` (the `origins` feature, spec §12.3).
    pub origins: bool,
    /// The most nodes the expanded tree may have; beyond it, F034 (spec §8.5).
    pub limit: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            origins: false,
            limit: LIMIT,
        }
    }
}

/// The result of expanding: the expanded tree and the diagnostics of stage 4b.
#[derive(Clone, Debug)]
pub(crate) struct Expanded {
    /// The expanded tree.
    pub(crate) root: Root,
    /// The diagnostics of stage 4b.
    pub(crate) diagnostics: Vec<Diagnostic>,
    /// What each top-level block of the processed document gave, so that an edit expands only the blocks it
    /// changes; `None` when text that two blocks gave was joined, which an edit can't take apart.
    pub(crate) blocks: Option<Vec<Block>>,
    /// What the expansion replaced in the expanded tree before it: `None` when it replaced all of it.
    pub(crate) replaced: Option<Replaced>,
}

/// The top-level nodes of an expanded tree that an edit replaced, how many replace them, and how far the nodes after
/// them moved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Replaced {
    pub(crate) nodes: ops::Range<usize>,
    pub(crate) inserted: usize,
    pub(crate) by: Move,
}

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

/// A diagnostic evaluating found, with its code, document and place when it is reported once (spec §10.4).
#[derive(Clone, Debug)]
struct Report {
    once: Option<(Code, DocumentId, Location)>,
    diagnostic: Diagnostic,
}

/// Expands a document, stage 4b of spec §11: the expanded tree of `resolved`'s processed document, with what
/// `validated` refused left out, `loaded`'s inputs and data, and `context`'s time and zone.
pub(crate) fn expand(
    resolved: &Resolved,
    validated: &Validated,
    loaded: &Loaded,
    context: &RenderContext,
    options: &Options,
) -> Expanded {
    let mut expander = Expander::new(resolved, validated, loaded, context, options);
    let env = Env::document(&expander, resolved.main);
    let mut children = Vec::new();
    let mut blocks = Vec::new();
    for block in &resolved.document(resolved.main).root.children {
        let (nodes, given) = expander.block(block, &env);
        children.extend(nodes);
        blocks.push(given);
    }
    finished(children, blocks, loaded, None)
}

/// The expanded tree of `children`, which `blocks` gave, with `loaded`'s inputs, after an expansion that replaced
/// `replaced`. Text that two blocks gave next to each other is joined, as a list joins it (spec §8.1); an edit can't
/// take the blocks apart after that, so it expands the whole document again.
fn finished(
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

/// Expands one document.
struct Expander<'r> {
    resolved: &'r Resolved,
    validated: &'r Validated,
    options: &'r Options,
    /// Each document's `md`, without `loop`, and its `md.` functions.
    md: HashMap<DocumentId, (Value, Md<'r>)>,
    /// Each document's lines, for where in it an interpolation's path is.
    lines: HashMap<DocumentId, Lines<'r>>,
    /// The programs of the paths and defaults this render evaluates; validating keeps those of expressions.
    programs: HashMap<String, Result<Rc<Program>, Error>>,
    /// The depth of the last heading expanded, in output order, which relative headings follow (spec §7.2).
    last_heading: u8,
    /// Whether `last_heading` is still the one the block being expanded started with, and whether the block read it
    /// while it was.
    heading_kept: bool,
    heading_read: bool,
    /// The nodes made so far, against the limit.
    made: usize,
    /// The expansions under way (`each`, template slots, invocations), innermost last, with where each is; and,
    /// once the limit is crossed, how many were under way then.
    expanding: Vec<(DocumentId, Location)>,
    crossed: Option<usize>,
    /// What evaluating the block being expanded reported.
    reports: Vec<Report>,
}

impl<'r> Expander<'r> {
    fn new(
        resolved: &'r Resolved,
        validated: &'r Validated,
        loaded: &'r Loaded,
        context: &'r RenderContext,
        options: &'r Options,
    ) -> Self {
        let md = resolved
            .ids()
            .map(|id| {
                let settings = &resolved.document(id).settings;
                let mut md = context.md(settings).data(loaded.data(id));
                if id == resolved.main {
                    md = md.input(&loaded.inputs);
                }
                (id, (md.into(), context.functions(settings)))
            })
            .collect();
        let lines = resolved
            .ids()
            .map(|id| (id, resolved.document(id).lines()))
            .collect();
        Self {
            resolved,
            validated,
            options,
            md,
            lines,
            programs: HashMap::new(),
            last_heading: 0,
            heading_kept: true,
            heading_read: false,
            made: 0,
            expanding: Vec::new(),
            crossed: None,
            reports: Vec::new(),
        }
    }

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
    fn block(&mut self, block: &'r Node, env: &Env<'r>) -> (Vec<expanded::Node>, Block) {
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

    /// The depth of the last heading, which a body with relative headings reads.
    fn heading_depth(&mut self) -> u8 {
        self.heading_read |= self.heading_kept;
        self.last_heading
    }

    /// `md` of the document `id`.
    fn md(&self, id: DocumentId) -> Value {
        self.md[&id].0.clone()
    }

    /// The variables a document's body starts with: its `md`.
    fn values(&self, id: DocumentId) -> Values {
        Values::default().with("md", self.md(id))
    }

    /// What a document's body sees.
    fn scope(&self, id: DocumentId) -> Scope<'r> {
        Scope::body(self.resolved, id)
    }

    /// Evaluates an expression written in `env`'s document, with the program validating parsed; interpolation paths
    /// and declared defaults, which validating doesn't parse, are parsed here once.
    fn evaluate(&mut self, source: &str, env: &Env) -> Result<Value, Error> {
        let program = match self.validated.program(source) {
            Some(program) => Rc::clone(program),
            None => self
                .programs
                .entry(source.to_string())
                .or_insert_with(|| Program::parse(source).map(Rc::new))
                .clone()?,
        };
        program.evaluate(&env.values, &self.md[&env.document].1)
    }

    /// Reports a diagnostic found in `env`'s document, once per code and place.
    fn report(&mut self, code: Code, env: &Env, at: Location, message: impl Into<String>) {
        self.reports.push(Report {
            once: Some((code, env.document, at)),
            diagnostic: Diagnostic::new(code, at, message)
                .in_stage(Stage::Evaluate)
                .in_file(self.resolved.file(env.document)),
        });
    }

    /// A list of nodes expanded, adjacent text joined.
    fn list(&mut self, nodes: &'r [Node], env: &Env<'r>) -> Vec<expanded::Node> {
        let mut out = Vec::new();
        for node in nodes {
            self.node(node, env, &mut out);
        }
        text::joined(out)
    }

    /// Inline nodes expanded without joining text, as a fallback keeps them.
    fn inline(&mut self, nodes: &'r [Node], env: &Env<'r>) -> Vec<expanded::Node> {
        let mut out = Vec::new();
        for node in nodes {
            self.node(node, env, &mut out);
        }
        out
    }

    /// A node expanded into `out`: what it produces, or for an mdast node, itself with its children expanded and a
    /// heading at its computed depth.
    fn node(&mut self, node: &'r Node, env: &Env<'r>, out: &mut Vec<expanded::Node>) {
        let content = match node {
            Node::Yaml(_)
            | Node::ComponentDefinition(_)
            | Node::ComponentSignature(_)
            | Node::ComponentSlot(_) => return,
            Node::Interpolation(interpolation) => {
                return self.interpolation(interpolation, env, out);
            }
            Node::ComponentContainer(container) => {
                let invocation = components::Invocation::container(node, container);
                return self.component(invocation, env, out);
            }
            Node::ComponentLeaf(leaf) => {
                return self.component(components::Invocation::leaf(node, leaf), env, out);
            }
            Node::ComponentInline(leaf) => {
                return self.component(components::Invocation::inline(node, leaf), env, out);
            }
            Node::Link(link) => return self.link(link, node, env, out),
            Node::Image(image) => return self.image(image, node, env, out),
            Node::Blockquote(quote) => Content::Blockquote(tree::Blockquote {
                children: self.list(&quote.children, env),
                alert: quote.alert,
                position: None,
            }),
            Node::Break(_) => Content::Break(tree::Void::default()),
            Node::Code(code) => {
                if let Some(name) = self.renderer(code, env) {
                    let invocation = components::Invocation::code(node, code, name);
                    return self.component(invocation, env, out);
                }
                Content::Code(plain_code(code))
            }
            Node::Definition(definition) => {
                Content::Definition(self.definition(definition, node, env))
            }
            Node::Delete(delete) => Content::Delete(self.parent(&delete.children, env)),
            Node::Emphasis(emphasis) => Content::Emphasis(self.parent(&emphasis.children, env)),
            Node::FootnoteDefinition(footnote) => {
                Content::FootnoteDefinition(tree::FootnoteDefinition {
                    identifier: footnote.identifier.clone(),
                    label: footnote.label.clone(),
                    children: self.list(&footnote.children, env),
                    position: None,
                })
            }
            Node::FootnoteReference(reference) => Content::FootnoteReference(tree::Reference {
                position: None,
                ..reference.clone()
            }),
            Node::Heading(heading) => Content::Heading(self.heading(heading, env)),
            Node::Html(html) => Content::Html(literal(&html.value)),
            Node::ImageReference(image) => Content::ImageReference(tree::ImageReference {
                position: None,
                ..image.clone()
            }),
            Node::InlineCode(code) => Content::InlineCode(literal(&code.value)),
            Node::InlineMath(math) => Content::InlineMath(literal(&math.value)),
            Node::LinkReference(link) => Content::LinkReference(tree::LinkReference {
                children: self.list(&link.children, env),
                reference_type: link.reference_type,
                identifier: link.identifier.clone(),
                label: link.label.clone(),
                position: None,
            }),
            Node::List(list) => Content::List(tree::List {
                ordered: list.ordered,
                start: list.start,
                spread: list.spread,
                children: self.list(&list.children, env),
                position: None,
            }),
            Node::ListItem(item) => Content::ListItem(tree::ListItem {
                spread: item.spread,
                checked: item.checked,
                children: self.list(&item.children, env),
                position: None,
            }),
            Node::Math(math) => Content::Math(tree::Math {
                position: None,
                ..math.clone()
            }),
            Node::Paragraph(paragraph) => Content::Paragraph(self.parent(&paragraph.children, env)),
            Node::Strong(strong) => Content::Strong(self.parent(&strong.children, env)),
            Node::Table(table) => Content::Table(tree::Table {
                align: table.align.clone(),
                children: self.list(&table.children, env),
                position: None,
            }),
            Node::TableCell(cell) => Content::TableCell(self.parent(&cell.children, env)),
            Node::TableRow(row) => Content::TableRow(self.parent(&row.children, env)),
            Node::Text(text) => Content::Text(literal(&text.value)),
            Node::ThematicBreak(_) => Content::ThematicBreak(tree::Void::default()),
            Node::WikiLink(link) => Content::WikiLink(tree::WikiLink {
                position: None,
                ..link.clone()
            }),
        };
        let made = self.made(content, origins::of(node, env));
        out.push(made);
    }

    /// The name of the code renderer that renders `code` in `env`'s document, when one alone does (spec §7.7).
    fn renderer(&self, code: &tree::Code, env: &Env<'r>) -> Option<&'r str> {
        code.info.as_ref()?;
        let renderers = &self.resolved.document(env.document).renderers;
        match renderers.get(code.lang.as_deref()?)? {
            Renderer::Component(name, _) => Some(name),
            Renderer::Ambiguous(_) => None,
        }
    }

    /// A link reference definition, its URL's interpolations filled in; empty when they would give it a scheme it
    /// may not have (W109).
    fn definition(
        &mut self,
        definition: &tree::LinkDefinition,
        node: &Node,
        env: &Env<'r>,
    ) -> tree::LinkDefinition {
        tree::LinkDefinition {
            url: self
                .checked_url(&definition.url, definition.url_parts.as_deref(), node, env)
                .unwrap_or_default(),
            url_parts: None,
            position: None,
            ..definition.clone()
        }
    }

    /// A node with children and nothing else, its children expanded.
    fn parent(&mut self, children: &'r [Node], env: &Env<'r>) -> tree::Parent<expanded::Node> {
        tree::Parent {
            children: self.list(children, env),
            position: None,
        }
    }

    /// A heading at its computed depth: in a body with relative headings, below the heading the invocation is under
    /// (spec §7.2). Relative headings after it follow it.
    fn heading(
        &mut self,
        heading: &'r tree::Heading,
        env: &Env<'r>,
    ) -> tree::Heading<expanded::Node> {
        let depth = env
            .shift
            .map_or(heading.depth, |base| (base + heading.depth).min(6));
        let children = self.list(&heading.children, env);
        self.last_heading = depth;
        self.heading_kept = false;
        tree::Heading {
            depth,
            children,
            position: None,
        }
    }

    /// A node made, counted against the limit (spec §8.5): the first that crosses it is reported at the expansion
    /// under way, F034.
    fn made(&mut self, content: Content, origin: Option<Origin>) -> expanded::Node {
        self.made += 1;
        if self.made > self.options.limit
            && self.crossed.is_none()
            && let Some((id, at)) = self.expanding.last().copied()
        {
            self.crossed = Some(self.expanding.len());
            self.reports.push(Report {
                once: None,
                diagnostic: Diagnostic::new(
                    Code::F034,
                    at,
                    format!(
                        "the expanded document would have more than {} nodes; this renders its fallback",
                        self.options.limit
                    ),
                )
                .in_stage(Stage::Evaluate)
                .in_file(self.resolved.file(id)),
            });
        }
        expanded::Node { content, origin }
    }

    /// Whether the limit has been crossed.
    fn over_limit(&self) -> bool {
        self.crossed.is_some()
    }

    /// Runs `expansion`, an `each`, a template slot or an invocation written at `at` in `env`'s document. `None`
    /// when it renders its fallback instead (spec §10.3): it crossed the limit itself, or starts once the limit is
    /// crossed. One under way when an inner one crossed it keeps what it made.
    fn expanding<T>(
        &mut self,
        env: &Env,
        at: Location,
        expansion: impl FnOnce(&mut Self) -> T,
    ) -> Option<T> {
        if self.over_limit() {
            return None;
        }
        self.expanding.push((env.document, at));
        let depth = self.expanding.len();
        let result = expansion(self);
        self.expanding.pop();
        (self.crossed != Some(depth)).then_some(result)
    }
}

/// A text-only node.
fn literal(value: &str) -> tree::Literal {
    tree::Literal {
        value: value.to_string(),
        position: None,
    }
}

/// A code block as content, as no code renderer renders it.
fn plain_code(code: &tree::Code) -> expanded::Code {
    expanded::Code {
        lang: code.lang.clone(),
        meta: code.meta.clone(),
        value: code.value.clone(),
        component: None,
        alt: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
