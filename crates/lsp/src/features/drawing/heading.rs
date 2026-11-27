//! A heading ([a document drawn in the editor], decision 11): an ATX heading's `#`s replaced by its level's glyph
//! (`H1` to `H6`), set back in the marker's style, a setext heading's glyph inserted before its text and its underline
//! hidden, and its lines in the level's style, to the window's edge with `heading.fill`. With `heading.marker` off,
//! its `#`s or its underline stay as written, set back.
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
    let drawn = planner.settings().heading.options.marker;
    let source = planner.source();
    let marker = source
        .line(first)
        .chars()
        .skip(column)
        .take_while(|c| *c == '#')
        .count();
    if marker > 0 {
        let range = source.range(first, column, column + marker);
        if drawn {
            planner.replace(range, glyph, style::HEADING_MARKER, part);
        } else {
            planner.style(range, style::HEADING_MARKER, false);
        }
    } else {
        let position = source.position(first, column);
        let underline = source.skip_spaces(last, 0);
        let underline = source.range(last, underline, source.len(last));
        if drawn {
            planner.insert(position, format!("{glyph} "), style::HEADING_MARKER, part);
            planner.hide(underline, part);
        } else {
            planner.style(underline, style::HEADING_MARKER, false);
        }
    }
    let range = planner.source().to_end((first, column), last);
    planner.style(range, style, planner.settings().heading.options.fill);
}
