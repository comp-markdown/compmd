//! A list ([a document drawn in the editor], decision 11): each bulleted item's `-`, `*` or `+` replaced by the glyph of
//! its depth (`bullet.1` to `bullet.4`, again from 1 below that), styled where the client draws no `replace`; a task
//! item's checkbox drawn too. A numbered item keeps its number.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::{List, ListItem, Node};

use super::checkbox;
use super::line_span::LineSpan;
use super::planner::{Part, Planner};
use super::primitive::Primitive;
use super::source::Source;
use super::style;
use super::walk::{self, Nesting};

/// How many bullet glyphs there are before they repeat.
const BULLETS: usize = 4;

/// Draws `list`'s items, and what they hold, at `nesting`.
pub(crate) fn draw(list: &List, planner: &mut Planner, nesting: Nesting) {
    let lists = if list.ordered {
        nesting.lists
    } else {
        nesting.lists + 1
    };
    let inner = Nesting { lists, ..nesting };
    for node in &list.children {
        if let Node::ListItem(item) = node {
            draw_item(item, list.ordered, lists, planner);
        }
        walk::draw(node.children(), planner, inner);
    }
}

/// Draws `item`'s bullet, at the bulleted lists' `depth`, and its checkbox.
fn draw_item(item: &ListItem, ordered: bool, depth: usize, planner: &mut Planner) {
    let Some(at) = item.position else { return };
    let ((line, column), (last, _)) = (Source::place(at.start), Source::place(at.end));
    let part = Part {
        own: LineSpan::one(line),
        element: LineSpan::of(line, last),
    };
    let bullet = planner.source().char_at(line, column);
    if !ordered && planner.settings().list.on && matches!(bullet, Some('-' | '*' | '+')) {
        let level = (depth.max(1) - 1) % BULLETS + 1;
        let range = planner.source().range(line, column, column + 1);
        let style = style::bullet(level);
        if planner.has(Primitive::Replace) {
            let glyph = planner.glyph(&format!("bullet.{level}"));
            planner.replace(range, glyph, style, part);
        } else {
            planner.style(range, style, false);
        }
    }
    if let Some(checked) = item.checked
        && planner.settings().checkbox.on
    {
        checkbox::draw(checked, (line, column), part, planner);
    }
}
