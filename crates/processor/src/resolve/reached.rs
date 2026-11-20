//! The components the processed document reaches (G10 of `docs/first-components.md`): those its body invokes or
//! renders a code block with, in every branch and template whatever the inputs, and those the `compmd` definitions
//! it reaches invoke in turn, each name read in the document that writes it. A component it could name but never
//! uses, such as most of an ambient pack's, isn't reached, so a host loads no code for it. An example (spec §7.8) is
//! resolved on its own, so what it reaches is its own.

use super::names::{Binding, Renderer};
use super::uses::{code_blocks, invocations};
use super::{Body, ComponentId, DocumentId, Resolved};
use crate::tree::Node;
use std::collections::HashSet;

/// Every component the processed document reaches.
pub(super) fn reached(resolved: &Resolved) -> HashSet<ComponentId> {
    let mut found = HashSet::new();
    let body: Vec<&Node> = resolved
        .document(resolved.main)
        .root
        .children
        .iter()
        .filter(|node| !matches!(node, Node::ComponentDefinition(_)))
        .collect();
    let mut to_visit: Vec<(DocumentId, &[Node])> = body
        .iter()
        .map(|node| (resolved.main, std::slice::from_ref(*node)))
        .collect();
    while let Some((id, nodes)) = to_visit.pop() {
        for callee in callees(resolved, id, nodes) {
            if !found.insert(callee) {
                continue;
            }
            let component = resolved.component(callee);
            if let Body::Compmd(body) = &component.body {
                to_visit.push((component.document, body));
            }
        }
    }
    found
}

/// The components `nodes`, in the document `id`, invoke or render a code block with.
fn callees(resolved: &Resolved, id: DocumentId, nodes: &[Node]) -> Vec<ComponentId> {
    let document = resolved.document(id);
    let component = |name: &str| match document.names.get(name) {
        Some(Binding::Component(callee, _)) => Some(*callee),
        _ => None,
    };
    let mut used = Vec::new();
    invocations(nodes, &mut used);
    let mut blocks = Vec::new();
    code_blocks(nodes, &mut blocks);
    let rendered = blocks.into_iter().filter_map(|code| {
        match document.renderers.get(code.lang.as_deref()?)? {
            Renderer::Component(name, _) => component(name),
            Renderer::Ambiguous(_) => None,
        }
    });
    used.iter()
        .filter_map(|invocation| component(invocation.name))
        .chain(rendered)
        .collect()
}
