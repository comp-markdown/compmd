//! A GitHub alert's title ([a document drawn in the editor], decision 11): its `[!NOTE]` replaced by its type's icon
//! and title (`ℹ Note`) in its callout's style; styled where the client draws no `replace`.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::Alert;

use super::line_span::LineSpan;
use super::planner::{Part, Planner};
use super::primitive::Primitive;
use super::style;

/// Draws the title of an `alert` whose quote's first `>` is at `column` of `line`, a quote on `element`'s lines.
pub(crate) fn draw(
    alert: Alert,
    (line, column): (usize, usize),
    element: LineSpan,
    planner: &mut Planner,
) {
    let source = planner.source();
    let open = source.skip_spaces(line, column + 1);
    let rest = source.slice(line, open, source.len(line));
    let Some(inside) = rest.strip_prefix("[!").and_then(|title| title.find(']')) else {
        return;
    };
    let end = open + rest[..inside + "[!]".len()].chars().count();
    let range = source.range(line, open, end);
    let part = Part {
        own: LineSpan::one(line),
        element,
    };
    let style = style::callout(alert);
    if planner.has(Primitive::Replace) {
        let name = style::alert_name(alert);
        let icon = planner.glyph(&format!("callout.{name}"));
        planner.replace(range, format!("{icon} {}", title(alert)), style, part);
    } else {
        planner.style(range, style, false);
    }
}

/// The title an alert of `alert`'s type shows.
const fn title(alert: Alert) -> &'static str {
    match alert {
        Alert::Note => "Note",
        Alert::Tip => "Tip",
        Alert::Important => "Important",
        Alert::Warning => "Warning",
        Alert::Caution => "Caution",
    }
}
