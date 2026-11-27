//! A thematic break ([a document drawn in the editor], decisions 3 and 11): a `line` in its place, filled to the
//! window's edge with the rule glyph; where the client draws no `line`, its text replaced by as many rule glyphs and
//! its line in the rule style.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::Void;

use super::line_span::LineSpan;
use super::mark::Segment;
use super::planner::{Part, Planner};
use super::primitive::Primitive;
use super::source::Source;
use super::style;

/// Draws the thematic break `rule`.
pub(crate) fn draw(rule: &Void, planner: &mut Planner) {
    let Some(at) = rule.position else { return };
    let (line, column) = Source::place(at.start);
    let part = Part {
        own: LineSpan::one(line),
        element: LineSpan::one(line),
    };
    let glyph = planner.glyph("rule");
    let source = planner.source();
    let written = source.range(line, column, source.trimmed_len(line));
    if planner.has(Primitive::Line) {
        let hidden = if column == 0 {
            source.whole_lines(LineSpan::one(line))
        } else {
            written
        };
        planner.hide(hidden, part);
        let indent = Segment::new(" ".repeat(column), style::RULE);
        let segments = if column == 0 {
            Vec::new()
        } else {
            vec![indent]
        };
        planner.line(
            (line, true),
            segments,
            Some(Segment::new(glyph, style::RULE)),
            part,
        );
    } else {
        let width = source.trimmed_len(line).saturating_sub(column);
        planner.replace(written, glyph.repeat(width), style::RULE, part);
        planner.style(written, style::RULE, true);
    }
}
