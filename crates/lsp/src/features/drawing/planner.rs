//! What an element's drawing writes its marks into: the document's lines, the primitives and settings the plan
//! follows, the processor's tokens, and the marks so far. It gives each mark its `reveal` lines by `render.reveal`
//! (decision 5), and makes the plan: only the primitives the client declared (decision 3), the marks on the lines
//! asked for, sorted by where they start (decision 4).

use compmd::tokens::SyntaxToken;
use lsp_types::{Position, Range};

use super::glyphs::Glyphs;
use super::line_span::{LineSpan, lsp_line};
use super::mark::{Drawn, Mark, Segment};
use super::planning::Planning;
use super::primitive::Primitive;
use super::source::Source;
use crate::settings::{RenderSettings, Reveal};

/// The part of an element a mark draws, by its lines, and the element's own lines: what it gives way on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Part {
    /// The part's lines: a heading's line, a fence's, a table's lines.
    pub(crate) own: LineSpan,
    /// The whole element's lines.
    pub(crate) element: LineSpan,
}

/// A plan being made.
pub(crate) struct Planner<'a> {
    source: Source<'a>,
    planning: &'a Planning,
    glyphs: Glyphs<'a>,
    tokens: &'a [SyntaxToken],
    marks: Vec<Mark>,
}

impl<'a> Planner<'a> {
    /// A plan of `source` with `planning`, the processor's `tokens` for CompMD's own syntax.
    pub(crate) fn new(
        source: Source<'a>,
        planning: &'a Planning,
        tokens: &'a [SyntaxToken],
    ) -> Self {
        let render = &planning.render;
        Self {
            source,
            planning,
            glyphs: Glyphs::new(render.glyphs, &render.glyph),
            tokens,
            marks: Vec::new(),
        }
    }

    /// The document's lines.
    pub(crate) const fn source(&self) -> &Source<'a> {
        &self.source
    }

    /// The drawing's settings.
    pub(crate) const fn settings(&self) -> &'a RenderSettings {
        &self.planning.render
    }

    /// The processor's tokens.
    pub(crate) const fn tokens(&self) -> &'a [SyntaxToken] {
        self.tokens
    }

    /// Whether the client draws `primitive`.
    pub(crate) const fn has(&self, primitive: Primitive) -> bool {
        self.planning.primitives.contains(primitive)
    }

    /// The glyph `name` names.
    pub(crate) fn glyph(&self, name: &str) -> String {
        self.glyphs.get(name)
    }

    /// A mark that draws `part`: it gives way on the part's lines, or the element's, or never, by `render.reveal`.
    pub(crate) fn draw(&mut self, drawn: Drawn, part: Part) {
        let reveal = match self.planning.render.reveal {
            Reveal::Line => Some(part.own),
            Reveal::Element => Some(part.element),
            Reveal::Off => None,
        };
        self.marks.push(Mark { drawn, reveal });
    }

    /// A mark that only decorates the text, which never gives way: a style, a prefix.
    pub(crate) fn decorate(&mut self, drawn: Drawn) {
        self.marks.push(Mark {
            drawn,
            reveal: None,
        });
    }

    /// `range` hidden.
    pub(crate) fn hide(&mut self, range: Range, part: Part) {
        self.draw(Drawn::Hide { range }, part);
    }

    /// `range` shown as `text` in `style`.
    pub(crate) fn replace(
        &mut self,
        range: Range,
        text: String,
        style: impl Into<String>,
        part: Part,
    ) {
        let style = style.into();
        self.draw(Drawn::Replace { range, text, style }, part);
    }

    /// `text` in `style` at `position`.
    pub(crate) fn insert(
        &mut self,
        position: Position,
        text: String,
        style: impl Into<String>,
        part: Part,
    ) {
        let style = style.into();
        self.draw(
            Drawn::Insert {
                position,
                text,
                style,
            },
            part,
        );
    }

    /// A line of its own above (or below) `line`: `segments`, then `fill` to the window's edge.
    pub(crate) fn line(
        &mut self,
        (line, above): (usize, bool),
        segments: Vec<Segment>,
        fill: Option<Segment>,
        part: Part,
    ) {
        let line = lsp_line(line);
        self.draw(
            Drawn::Line {
                line,
                above,
                segments,
                fill,
            },
            part,
        );
    }

    /// `range` in `style`, to the window's edge when `whole_line`.
    pub(crate) fn style(&mut self, range: Range, style: impl Into<String>, whole_line: bool) {
        let style = style.into();
        self.decorate(Drawn::Style {
            range,
            style,
            whole_line,
        });
    }

    /// `text` in `style` before each of `lines`.
    pub(crate) fn prefix(&mut self, lines: LineSpan, text: String, style: impl Into<String>) {
        let style = style.into();
        self.decorate(Drawn::Prefix { lines, text, style });
    }

    /// The plan of the lines `asked`: the marks of the primitives the client draws on those lines, sorted by where
    /// they start (an element's before what it holds where they start alike).
    pub(crate) fn finish(self, asked: LineSpan) -> Vec<Mark> {
        let primitives = self.planning.primitives;
        let mut marks: Vec<Mark> = self
            .marks
            .into_iter()
            .filter(|mark| primitives.contains(mark.drawn.primitive()))
            .collect();
        marks.retain(|mark| mark.drawn.lines().meets(asked));
        marks.sort_by_key(|mark| mark.drawn.start());
        marks
    }
}
