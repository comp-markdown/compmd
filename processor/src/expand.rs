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
use crate::context::Context;
use crate::diagnostic::{Code, Diagnostic, Location, Stage};
use crate::expanded::{self, Content, Origin, Root};
use crate::lines::Lines;
use crate::load::Loaded;
use crate::md::Md;
use crate::resolve::{DocumentId, Resolved};
use crate::tree::{self, Node};
use crate::validate::Validated;
use crate::validate::scope::Scope;
use env::Env;
use std::collections::{HashMap, HashSet};
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
}

/// Expands a document, stage 4b of spec §11: the expanded tree of `resolved`'s processed document, with what
/// `validated` refused left out, `loaded`'s inputs and data, and `context`'s time and zone.
pub(crate) fn expand(
    resolved: &Resolved,
    validated: &Validated,
    loaded: &Loaded,
    context: &Context,
    options: &Options,
) -> Expanded {
    let mut expander = Expander::new(resolved, validated, loaded, context, options);
    let main = resolved.main;
    let env = Env::document(&expander, main);
    let mut root = Root {
        children: expander.list(&resolved.document(main).root.children, &env),
        inputs: loaded.inputs.clone(),
    };
    origins::finish(&mut root.children);
    expander.diagnostics.extend(layout::check(&root.children));
    if !options.origins {
        origins::strip(&mut root.children);
    }
    Expanded {
        root,
        diagnostics: expander.diagnostics,
    }
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
    /// The nodes made so far, against the limit.
    made: usize,
    /// The expansions under way (`each`, template slots, invocations), innermost last, with where each is; and,
    /// once the limit is crossed, how many were under way then.
    expanding: Vec<(DocumentId, Location)>,
    crossed: Option<usize>,
    diagnostics: Vec<Diagnostic>,
    /// The evaluation errors reported, by code, document and place, so each is reported once (spec §10.4).
    reported: HashSet<(Code, DocumentId, Location)>,
}

impl<'r> Expander<'r> {
    fn new(
        resolved: &'r Resolved,
        validated: &'r Validated,
        loaded: &'r Loaded,
        context: &'r Context<'r>,
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
            .map(|id| (id, Lines::new(&resolved.document(id).text)))
            .collect();
        Self {
            resolved,
            validated,
            options,
            md,
            lines,
            programs: HashMap::new(),
            last_heading: 0,
            made: 0,
            expanding: Vec::new(),
            crossed: None,
            diagnostics: Vec::new(),
            reported: HashSet::new(),
        }
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
        if self.reported.insert((code, env.document, at)) {
            self.diagnostics.push(
                Diagnostic::new(code, at, message)
                    .in_stage(Stage::Evaluate)
                    .in_file(self.resolved.file(env.document)),
            );
        }
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
            Node::Code(code) => Content::Code(tree::Code {
                position: None,
                ..code.clone()
            }),
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
            self.diagnostics.push(
                Diagnostic::new(
                    Code::F034,
                    at,
                    format!(
                        "the expanded document would have more than {} nodes; this renders its fallback",
                        self.options.limit
                    ),
                )
                .in_stage(Stage::Evaluate)
                .in_file(self.resolved.file(id)),
            );
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
