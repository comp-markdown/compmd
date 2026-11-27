//! A task item's checkbox ([a document drawn in the editor], decision 11): its `[ ]` or `[x]` replaced by the glyph of
//! its state, styled where the client draws no `replace`.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use super::planner::{Part, Planner};
use super::primitive::Primitive;
use super::style;

/// Draws the checkbox of a task item whose marker is at `column` of `line`, checked or not, as a mark of `part`.
pub(crate) fn draw(
    checked: bool,
    (line, column): (usize, usize),
    part: Part,
    planner: &mut Planner,
) {
    let source = planner.source();
    let marker_end = column
        + source
            .line(line)
            .chars()
            .skip(column)
            .take_while(|c| !c.is_whitespace())
            .count();
    let open = source.skip_spaces(line, marker_end);
    let written = source.slice(line, open, open + 3);
    let Some(range) = is_box(&written).then(|| source.range(line, open, open + 3)) else {
        return;
    };
    let (name, style) = if checked {
        ("checkbox.checked", style::CHECKBOX_CHECKED)
    } else {
        ("checkbox.unchecked", style::CHECKBOX_UNCHECKED)
    };
    if planner.has(Primitive::Replace) {
        let glyph = planner.glyph(name);
        planner.replace(range, glyph, style, part);
    } else {
        planner.style(range, style, false);
    }
}

/// Whether `written` is a checkbox.
fn is_box(written: &str) -> bool {
    matches!(written, "[ ]" | "[x]" | "[X]")
}
