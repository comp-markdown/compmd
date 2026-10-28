//! Origins (spec §12.3): the range of the processed document that made each expanded node. A node from a
//! definition's body has its invocation's; text a value or an interpolation produced has its parent's; any other
//! node is where it is written. Each one's occurrence counts the nodes before it in its list with the same range.

use super::env::Env;
use crate::expanded::{Content, Node, Origin};
use crate::tree::{self, Point};
use std::collections::HashMap;

/// The origin of what `node` makes in `env`: the invocation's in a body, else where `node` is written.
pub(super) fn of(node: &tree::Node, env: &Env) -> Option<Origin> {
    env.origin.or_else(|| range(node))
}

/// Where a node of the processed document is: a container from its opener to the end of its closer or, closed by
/// recovery, to the end of the last content it holds.
pub(super) fn range(node: &tree::Node) -> Option<Origin> {
    let position = node.position()?;
    let end = match node {
        tree::Node::ComponentContainer(container) => container
            .close_position
            .map_or_else(|| content_end(node), |close| close.end),
        _ => position.end,
    };
    Some(Origin {
        start: position.start,
        end,
        occurrence: 0,
    })
}

/// The furthest end among a node and everything in it.
fn content_end(node: &tree::Node) -> Point {
    let own = node
        .position()
        .map(|position| position.end)
        .unwrap_or_default();
    let slots = match node {
        tree::Node::ComponentContainer(container) => container.slots.as_slice(),
        _ => &[],
    };
    let inner = slots
        .iter()
        .flat_map(|slot| {
            slot.position
                .map(|position| position.end)
                .into_iter()
                .chain(slot.children.iter().map(content_end))
        })
        .chain(node.children().iter().map(content_end));
    inner.fold(own, |end, point| {
        if point.offset > end.offset {
            point
        } else {
            end
        }
    })
}

/// Origins finished: a node without one takes its parent's, and each gets its occurrence in its list; a template
/// slot's instances count as one list.
pub(super) fn finish(nodes: &mut [Node]) {
    list(nodes, None, &mut HashMap::new());
}

fn list(nodes: &mut [Node], parent: Option<Origin>, seen: &mut HashMap<(Point, Point), usize>) {
    for node in nodes {
        node.origin = numbered(node.origin.or(parent), seen);
        let inner = node.origin;
        if let Content::ComponentContainer(container) = &mut node.content {
            let mut slots = HashMap::new();
            for slot in &mut container.slots {
                slot.origin = numbered(slot.origin.or(inner), &mut slots);
                list(&mut slot.children, slot.origin, &mut HashMap::new());
                instances(slot.instances.as_deref_mut(), slot.origin);
            }
            instances(container.instances.as_deref_mut(), inner);
        }
        if let Some(children) = node.content.children_mut() {
            list(children, inner, &mut HashMap::new());
        }
    }
}

fn instances(instances: Option<&mut [crate::expanded::Instance]>, parent: Option<Origin>) {
    let mut seen = HashMap::new();
    for instance in instances.into_iter().flatten() {
        list(&mut instance.children, parent, &mut seen);
    }
}

/// `origin` with its occurrence among those `seen` before it.
fn numbered(origin: Option<Origin>, seen: &mut HashMap<(Point, Point), usize>) -> Option<Origin> {
    let mut origin = origin?;
    let count = seen.entry((origin.start, origin.end)).or_insert(0);
    origin.occurrence = *count;
    *count += 1;
    Some(origin)
}

/// Every origin removed, for a render without the `origins` feature.
pub(super) fn strip(nodes: &mut [Node]) {
    for node in nodes {
        node.origin = None;
        if let Content::ComponentContainer(container) = &mut node.content {
            for slot in &mut container.slots {
                slot.origin = None;
                strip(&mut slot.children);
                for instance in slot.instances.iter_mut().flatten() {
                    strip(&mut instance.children);
                }
            }
            for instance in container.instances.iter_mut().flatten() {
                strip(&mut instance.children);
            }
        }
        if let Some(children) = node.content.children_mut() {
            strip(children);
        }
    }
}
