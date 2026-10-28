//! The layout checks of the expanded tree (spec §9.2, §9.3), at each breakpoint: a box that places blocks by
//! position needs a height (W119), and a box whose blocks use breakpoints needs a width where its width would come
//! from its content (W114).
//!
//! A block's nearest box is the nearest `box` or host-language component around it in the expanded tree; `each`,
//! `if` and `display=contents` components are gone by then, and list items, quotes and table cells don't count.

use super::params::BREAKPOINTS;
use crate::diagnostic::{Code, Diagnostic, Location, Stage};
use crate::expanded::{Content, Node, Param};
use serde_json::Value;

/// The insets, which place a block by position.
const INSETS: [&str; 4] = [
    "inset-inline-start",
    "inset-inline-end",
    "inset-block-start",
    "inset-block-end",
];

/// What a block is laid out in.
#[derive(Clone, Copy)]
enum Around<'a> {
    /// The area the processor renders into.
    Top,
    /// A `box`, with its params.
    Box(&'a [Param]),
    /// An item of a host-language component's slot, laid out by the component.
    Item,
}

/// A block with a box: a component node, with its params, where it is, and what it holds.
struct Block<'a> {
    node: &'a Node,
    params: &'a [Param],
}

/// The diagnostics of the layout checks on `nodes`, the top level of an expanded tree.
pub(super) fn check(nodes: &[Node]) -> Vec<Diagnostic> {
    let mut found = Vec::new();
    let mut blocks = Vec::new();
    collect(nodes, &mut blocks);
    for block in &blocks {
        if (0..=BREAKPOINTS.len()).any(|at| positioned(block.params, at)) {
            report(
                &mut found,
                Code::W119,
                block.node,
                "a block placed by position at the top level needs an area with a height; it is placed against the page",
            );
        }
        visit(block, Around::Top, &mut found);
    }
    found
}

/// The blocks whose nearest box is the one `nodes` are in: component nodes, found through any other nodes.
fn collect<'a>(nodes: &'a [Node], blocks: &mut Vec<Block<'a>>) {
    for node in nodes {
        match &node.content {
            Content::ComponentContainer(container) => blocks.push(Block {
                node,
                params: &container.params,
            }),
            Content::ComponentLeaf(leaf) => blocks.push(Block {
                node,
                params: &leaf.params,
            }),
            other => collect(other.children(), blocks),
        }
    }
}

/// Checks a block in what it is laid out in, then what it holds.
fn visit(block: &Block, around: Around, found: &mut Vec<Diagnostic>) {
    let Content::ComponentContainer(container) = &block.node.content else {
        return;
    };
    let mut inside = Vec::new();
    for list in block.node.content.lists() {
        collect(list, &mut inside);
    }
    let held = if container.name == "box" {
        Around::Box(&container.params)
    } else {
        Around::Item
    };
    let followed = inside.iter().any(|inner| uses_breakpoints(inner.params));
    let levels = 0..=BREAKPOINTS.len();
    if followed
        && levels
            .clone()
            .any(|at| needs_width(block.params, around, at))
    {
        report(
            found,
            Code::W114,
            block.node,
            "its blocks use breakpoints, so it follows no width of its content; give it a `width`",
        );
    }
    if let Around::Box(params) = held
        && levels
            .into_iter()
            .any(|at| !sized(params, at) && inside.iter().any(|inner| positioned(inner.params, at)))
    {
        report(
            found,
            Code::W119,
            block.node,
            "it places blocks by position, so it needs a `height`, an `aspect-ratio` or `fill`",
        );
    }
    for inner in &inside {
        visit(inner, held, found);
    }
}

fn report(found: &mut Vec<Diagnostic>, code: Code, node: &Node, message: &str) {
    if let Some(origin) = node.origin {
        found.push(
            Diagnostic::new(code, Location::from(origin.start), message).in_stage(Stage::Evaluate),
        );
    }
}

/// The value of `key` at the breakpoint `at` (0 below `sm`, then `sm`, `md`, `lg`): the one written for the widest
/// breakpoint up to it.
fn value<'a>(params: &'a [Param], key: &str, at: usize) -> Option<&'a Value> {
    params
        .iter()
        .filter(|param| param.key == key && level(param.breakpoint.as_deref()) <= at)
        .max_by_key(|param| level(param.breakpoint.as_deref()))
        .map(|param| &param.value)
}

/// The text of `key` at `at`, or `default`.
fn text<'a>(params: &'a [Param], key: &str, at: usize, default: &'a str) -> &'a str {
    value(params, key, at)
        .and_then(Value::as_str)
        .unwrap_or(default)
}

fn level(breakpoint: Option<&str>) -> usize {
    breakpoint
        .and_then(|breakpoint| BREAKPOINTS.iter().position(|known| *known == breakpoint))
        .map_or(0, |index| index + 1)
}

/// Whether a block uses breakpoints: a breakpoint on any of its params, or `show` or `hide`.
fn uses_breakpoints(params: &[Param]) -> bool {
    params
        .iter()
        .any(|param| param.breakpoint.is_some() || param.key == "show" || param.key == "hide")
}

/// Whether a block is placed by position at `at`: an inset that isn't `auto`.
fn positioned(params: &[Param], at: usize) -> bool {
    INSETS
        .iter()
        .any(|inset| value(params, inset, at).is_some_and(|value| value != "auto"))
}

/// Whether a box has a height at `at`: from `height`, `aspect-ratio` or `fill`.
fn sized(params: &[Param], at: usize) -> bool {
    value(params, "height", at).is_some_and(|height| height != "auto")
        || value(params, "aspect-ratio", at).is_some_and(|ratio| ratio != "none")
        || value(params, "fill", 0).is_some()
}

/// Whether a block in `around` needs a `width` at `at`, having none: its width would come from its content.
fn needs_width(params: &[Param], around: Around, at: usize) -> bool {
    if value(params, "width", at).is_some_and(|width| width != "auto") {
        return false;
    }
    if positioned(params, at) {
        let inset = |key| value(params, key, at).is_some_and(|value| value != "auto");
        return !(inset("inset-inline-start") && inset("inset-inline-end"));
    }
    let Around::Box(parent) = around else {
        return false;
    };
    match text(parent, "display", at, "block") {
        "flex" if text(parent, "flex-direction", at, "row") == "row" => true,
        "flex" => {
            let align = text(
                params,
                "align-self",
                at,
                text(parent, "align-items", at, "stretch"),
            );
            align != "stretch" || matches!(text(params, "justify-self", at, ""), "center" | "end")
        }
        "grid" => value(params, "justify-self", at).is_some(),
        _ => false,
    }
}
