//! A border of a container's box ([a document drawn in the editor], decisions 3 and 11): its top border in place of
//! its opener, a slot's divider in place of the slot's line, its bottom border in place of its closer, each titled
//! with what its line names. The client that draws `line` gets a line of its own in place of the written lines, which
//! are hidden: the corner, the title, then the border to the window's edge. One that doesn't gets the marker replaced
//! by the corner, the title styled where it is written, and the line in the border's style (a renderer's top border,
//! divider or bottom border).
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use super::line_span::LineSpan;
use super::mark::Segment;
use super::planner::{Part, Planner};
use super::primitive::Primitive;
use super::style;

/// A part of a border's title, as it is written: a line, its columns, and its style.
pub(crate) struct Titled {
    pub(crate) line: usize,
    pub(crate) from: usize,
    pub(crate) to: usize,
    pub(crate) style: &'static str,
}

/// A border, and the written lines it stands for.
pub(crate) struct BoxBorder {
    /// The lines it stands for: an opener and its continuation lines, a slot's line, a closer.
    pub(crate) lines: LineSpan,
    /// The column the written lines start at.
    pub(crate) column: usize,
    /// Where the marker before the title ends on the first line (`:::`, `:: #`): all of a closer.
    pub(crate) marker_end: usize,
    /// The glyph its corner is.
    pub(crate) corner: &'static str,
    /// Its title.
    pub(crate) title: Vec<Titled>,
    /// The style of its line where it isn't a line of its own.
    pub(crate) line_style: &'static str,
    /// The box's lines.
    pub(crate) element: LineSpan,
}

impl BoxBorder {
    /// Draws the border.
    pub(crate) fn draw(&self, planner: &mut Planner) {
        let part = Part {
            own: self.lines,
            element: self.element,
        };
        let horizontal = planner.glyph("box.horizontal");
        let corner = planner.glyph(self.corner);
        let (first, last) = (self.lines.first as usize, self.lines.last as usize);
        if planner.has(Primitive::Line) {
            let source = planner.source();
            let hidden = if self.column == 0 {
                source.whole_lines(self.lines)
            } else {
                source.to_end((first, self.column), last)
            };
            let mut segments = vec![self.lead(&corner, &horizontal)];
            for titled in &self.title {
                let text = source.slice(titled.line, titled.from, titled.to);
                segments.push(Segment::new(text, titled.style));
                segments.push(Segment::new(" ", style::CONTAINER_BORDER));
            }
            planner.hide(hidden, part);
            let fill = Segment::new(horizontal, style::CONTAINER_BORDER);
            planner.line((first, true), segments, Some(fill), part);
        } else {
            let marker = planner.source().range(first, self.column, self.marker_end);
            let lead = self.lead(&corner, &horizontal);
            planner.replace(marker, lead.text, lead.style, part);
            for titled in &self.title {
                let range = planner.source().range(titled.line, titled.from, titled.to);
                planner.style(range, titled.style, false);
            }
            let written = planner.source().to_end((first, self.column), first);
            planner.style(written, self.line_style, true);
        }
    }

    /// What comes before the title: the corner and a stretch of the border, and a space before a title.
    fn lead(&self, corner: &str, horizontal: &str) -> Segment {
        let space = if self.title.is_empty() { "" } else { " " };
        Segment::new(
            format!("{corner}{horizontal}{space}"),
            style::CONTAINER_BORDER,
        )
    }
}
