//! A heading ([a document drawn in the editor], decision 11): an ATX heading's `#`s replaced by its level's glyph, a
//! setext heading's glyph inserted before its text and its underline hidden, and its lines in the level's style, to
//! the window's edge with `heading.fill`.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::Heading;

use super::line_span::LineSpan;
use super::planner::{Part, Planner};
use super::source::Source;
use super::style;

/// Draws `heading`.
pub(crate) fn draw(heading: &Heading, planner: &mut Planner) {
    let Some(at) = heading.position else { return };
    let ((first, column), (last, _)) = (Source::place(at.start), Source::place(at.end));
    let lines = LineSpan::of(first, last);
    let part = Part {
        own: lines,
        element: lines,
    };
    let style = style::heading(heading.depth);
    let glyph = planner.glyph(&format!("heading.{}", heading.depth));
    let source = planner.source();
    let marker = source
        .line(first)
        .chars()
        .skip(column)
        .take_while(|c| *c == '#')
        .count();
    if marker > 0 {
        let range = source.range(first, column, column + marker);
        planner.replace(range, glyph, style.clone(), part);
    } else {
        let position = source.position(first, column);
        let underline = source.skip_spaces(last, 0);
        let hidden = source.range(last, underline, source.len(last));
        planner.insert(position, format!("{glyph} "), style.clone(), part);
        planner.hide(hidden, part);
    }
    let range = planner.source().to_end((first, column), last);
    planner.style(range, style, planner.settings().heading.options.fill);
}
