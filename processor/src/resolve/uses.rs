//! The component names the bodies use (spec §6.3): F002 for an unknown name, F023 for an ambiguous one, W113 for
//! one from an ambient pack, and F019 for a definition that invokes itself, directly or not.
//!
//! The processed document's body is checked, and the `compmd` definitions of every document, since their bodies
//! are what is used of an imported one.

use super::names::{Binding, Level};
use super::{Body, ComponentId, DocumentId, Resolved};
use crate::diagnostic::{Code, Diagnostic, Fix, Location, Stage};
use crate::nearest::nearest;
use crate::tree::{Node, Point};
use std::collections::HashSet;

/// A component a body invokes: its name, and where its line or inline component starts.
pub(crate) struct Invocation<'a> {
    pub(crate) name: &'a str,
    pub(crate) start: Point,
}

/// Every component the nodes invoke, in source order, in slots and fallbacks too.
pub(crate) fn invocations<'a>(nodes: &'a [Node], out: &mut Vec<Invocation<'a>>) {
    for node in nodes {
        let named = match node {
            Node::ComponentContainer(container) => Some((&container.name, container.position)),
            Node::ComponentLeaf(leaf) | Node::ComponentInline(leaf) => {
                Some((&leaf.name, leaf.position))
            }
            _ => None,
        };
        if let Some((name, position)) = named {
            out.push(Invocation {
                name,
                start: position.map(|position| position.start).unwrap_or_default(),
            });
        }
        invocations(node.children(), out);
        if let Node::ComponentContainer(container) = node {
            for slot in &container.slots {
                invocations(&slot.children, out);
            }
        }
    }
}

/// Reports the names the documents use, the processed one's by top-level block, and the cycles.
pub(super) fn report(resolved: &mut Resolved, report_ambient: bool) {
    let main = resolved.main;
    for id in resolved.ids() {
        let root = &resolved.document(id).root;
        if id == main {
            let blocks = root
                .children
                .iter()
                .map(|block| {
                    names_used(resolved, id, &[std::slice::from_ref(block)], report_ambient)
                })
                .collect();
            resolved.diagnostics.blocks = blocks;
            continue;
        }
        let bodies: Vec<&[Node]> = root
            .children
            .iter()
            .filter(|node| matches!(node, Node::ComponentDefinition(_)))
            .map(Node::children)
            .collect();
        let found = names_used(resolved, id, &bodies, report_ambient);
        let list = if id < main {
            &mut resolved.diagnostics.before
        } else {
            &mut resolved.diagnostics.after
        };
        list.extend(found);
    }
    let mut found = Vec::new();
    resolved.cycles = recursion(resolved, &mut found);
    resolved.diagnostics.after.extend(found);
}

/// The diagnostics of the names the top-level block `block` of the processed document uses.
pub(super) fn block(resolved: &Resolved, block: &Node, report_ambient: bool) -> Vec<Diagnostic> {
    names_used(
        resolved,
        resolved.main,
        &[std::slice::from_ref(block)],
        report_ambient,
    )
}

/// The diagnostics of the names `bodies`, of the document `id`, use.
fn names_used(
    resolved: &Resolved,
    id: DocumentId,
    bodies: &[&[Node]],
    report_ambient: bool,
) -> Vec<Diagnostic> {
    let document = resolved.document(id);
    let lines = document.lines();
    let file = resolved.file(id);
    let mut used = Vec::new();
    for body in bodies {
        invocations(body, &mut used);
    }
    let mut found = Vec::new();
    for invocation in used {
        let at = lines.name_at(invocation.start);
        let name = invocation.name;
        let diagnostic = match document.names.get(name) {
            None => {
                let known = nearest(name, document.names.all());
                let hint =
                    known.map_or_else(String::new, |known| format!("; did you mean `{known}`?"));
                Diagnostic::new(Code::F002, at, format!("unknown component `{name}`{hint}"))
                    .with_fix(known.map(|known| Fix::replacing(at, name, known)))
            }
            Some(Binding::Ambiguous(sources)) => Diagnostic::new(
                Code::F023,
                at,
                format!(
                    "`{name}` is provided by {}; list it in one import to choose",
                    sources.join(" and ")
                ),
            ),
            Some(Binding::Component(_, Level::Ambient)) if report_ambient => Diagnostic::new(
                Code::W113,
                at,
                format!(
                    "`{name}` comes from a pack this processor provides, so other processors show its fallback"
                ),
            ),
            _ => continue,
        };
        found.push(diagnostic.in_stage(Stage::Resolve).in_file(file.clone()));
    }
    found
}

/// An invocation of a `compmd` definition in another's body: the callee, where the name is, where the invocation
/// starts.
type Edge = (ComponentId, Location, Point);

/// F019: a `compmd` definition whose body leads back to it, reported at the invocation that closes the cycle. Gives
/// those invocations, by their document and where they start: each renders its fallback, which breaks every cycle.
fn recursion(resolved: &Resolved, found: &mut Vec<Diagnostic>) -> HashSet<(DocumentId, Point)> {
    let edges: Vec<Vec<Edge>> = resolved
        .components
        .iter()
        .map(|component| {
            let Body::Compmd(body) = &component.body else {
                return Vec::new();
            };
            let document = resolved.document(component.document);
            let lines = document.lines();
            let mut used = Vec::new();
            invocations(body, &mut used);
            used.into_iter()
                .filter_map(|invocation| match document.names.get(invocation.name) {
                    Some(Binding::Component(callee, _))
                        if matches!(resolved.component(*callee).body, Body::Compmd(_)) =>
                    {
                        Some((*callee, lines.name_at(invocation.start), invocation.start))
                    }
                    _ => None,
                })
                .collect()
        })
        .collect();
    let mut search = Search {
        resolved,
        edges: &edges,
        state: vec![State::New; edges.len()],
        found,
        cycles: HashSet::new(),
    };
    for component in 0..edges.len() {
        if search.state[component] == State::New {
            search.visit(ComponentId(component));
        }
    }
    search.cycles
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    New,
    /// On the path being followed: reaching it again closes a cycle.
    Open,
    Done,
}

/// A depth-first search for cycles among definitions.
struct Search<'a> {
    resolved: &'a Resolved,
    edges: &'a [Vec<Edge>],
    state: Vec<State>,
    found: &'a mut Vec<Diagnostic>,
    /// The invocations that close a cycle.
    cycles: HashSet<(DocumentId, Point)>,
}

impl Search<'_> {
    fn visit(&mut self, component: ComponentId) {
        self.state[component.0] = State::Open;
        for &(callee, at, start) in &self.edges[component.0] {
            match self.state[callee.0] {
                State::Open => {
                    let caller = self.resolved.component(component);
                    self.cycles.insert((caller.document, start));
                    let name = &self.resolved.component(callee).name;
                    self.found.push(
                        Diagnostic::new(
                            Code::F019,
                            at,
                            format!("`{}` invokes `{name}`, which leads back to it: definitions can't recurse", caller.name),
                        )
                        .in_stage(Stage::Resolve)
                        .in_file(self.resolved.file(caller.document)),
                    );
                }
                State::New => self.visit(callee),
                State::Done => {}
            }
        }
        self.state[component.0] = State::Done;
    }
}
