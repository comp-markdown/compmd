//! A mark of the drawing plan ([a document drawn in the editor], Terms and decision 2): one primitive at a place, with
//! its style and the lines it gives way on. Its JSON is what a renderer draws: `kind` names the primitive.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use lsp_types::{Position, Range};
use serde::Serialize;

use super::line_span::LineSpan;
use super::primitive::Primitive;

/// A mark: what it draws, and the lines it gives way on (decision 5), absent when it never does.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(crate) struct Mark {
    #[serde(flatten)]
    pub(crate) drawn: Drawn,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reveal: Option<LineSpan>,
}

/// What a mark draws: one of the six primitives.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub(crate) enum Drawn {
    /// `range` not shown; a range from a line's start to the next line's start hides the whole line.
    Hide { range: Range },
    /// `range` shown as `text`.
    Replace {
        range: Range,
        text: String,
        style: String,
    },
    /// `text` at `position`, shifting what follows.
    Insert {
        position: Position,
        text: String,
        style: String,
    },
    /// A line of its own above or below `line`: its segments, then `fill`'s text repeated to the window's edge. Above
    /// a line that a `hide` hides whole, it stands in that line's place.
    Line {
        line: u32,
        above: bool,
        segments: Vec<Segment>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fill: Option<Segment>,
    },
    /// `text` at the start of each of `lines`, wrapped ones too.
    Prefix {
        lines: LineSpan,
        text: String,
        style: String,
    },
    /// `range` in `style`; to the window's edge when `whole_line`.
    Style {
        range: Range,
        style: String,
        #[serde(rename = "wholeLine")]
        whole_line: bool,
    },
}

/// A text in a style: a part of a `line`, or what fills it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct Segment {
    pub(crate) text: String,
    pub(crate) style: String,
}

impl Segment {
    /// `text` in `style`.
    pub(crate) fn new(text: impl Into<String>, style: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: style.into(),
        }
    }
}

impl Drawn {
    /// The primitive it is.
    pub(crate) const fn primitive(&self) -> Primitive {
        match self {
            Self::Hide { .. } => Primitive::Hide,
            Self::Replace { .. } => Primitive::Replace,
            Self::Insert { .. } => Primitive::Insert,
            Self::Line { .. } => Primitive::Line,
            Self::Prefix { .. } => Primitive::Prefix,
            Self::Style { .. } => Primitive::Style,
        }
    }

    /// Where it starts, which orders the plan.
    pub(crate) const fn start(&self) -> Position {
        match self {
            Self::Hide { range } | Self::Replace { range, .. } | Self::Style { range, .. } => {
                range.start
            }
            Self::Insert { position, .. } => *position,
            Self::Line { line, .. } => Position {
                line: *line,
                character: 0,
            },
            Self::Prefix { lines, .. } => Position {
                line: lines.first,
                character: 0,
            },
        }
    }

    /// The lines it is drawn on.
    pub(crate) fn lines(&self) -> LineSpan {
        match self {
            Self::Hide { range } | Self::Replace { range, .. } | Self::Style { range, .. } => {
                LineSpan::of_range(range)
            }
            Self::Insert { position, .. } => LineSpan {
                first: position.line,
                last: position.line,
            },
            Self::Line { line, .. } => LineSpan {
                first: *line,
                last: *line,
            },
            Self::Prefix { lines, .. } => *lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn at(line: u32, character: u32) -> Position {
        Position { line, character }
    }

    #[test]
    fn a_mark_is_its_primitive_by_kind_with_its_reveal_lines_when_it_has_them() {
        let range = Range {
            start: at(1, 0),
            end: at(1, 2),
        };
        let replace = Mark {
            drawn: Drawn::Replace {
                range,
                text: "◎".into(),
                style: "compmd.heading.2".into(),
            },
            reveal: Some(LineSpan::one(1)),
        };
        assert_eq!(
            serde_json::to_value(&replace).expect("JSON"),
            json!({ "kind": "replace", "range": { "start": { "line": 1, "character": 0 }, "end": { "line": 1, "character": 2 } }, "text": "◎", "style": "compmd.heading.2", "reveal": { "first": 1, "last": 1 } })
        );
        let style = Mark {
            drawn: Drawn::Style {
                range,
                style: "compmd.code".into(),
                whole_line: true,
            },
            reveal: None,
        };
        assert_eq!(
            serde_json::to_value(&style).expect("JSON"),
            json!({ "kind": "style", "range": { "start": { "line": 1, "character": 0 }, "end": { "line": 1, "character": 2 } }, "style": "compmd.code", "wholeLine": true })
        );
        let line = Drawn::Line {
            line: 4,
            above: true,
            segments: vec![Segment::new("rust", "compmd.code.language")],
            fill: None,
        };
        assert_eq!(
            serde_json::to_value(&line).expect("JSON"),
            json!({ "kind": "line", "line": 4, "above": true, "segments": [{ "text": "rust", "style": "compmd.code.language" }] })
        );
    }

    #[test]
    fn a_mark_starts_and_is_drawn_on_its_lines() {
        let range = Range {
            start: at(2, 1),
            end: at(3, 0),
        };
        let marks = [
            Drawn::Hide { range },
            Drawn::Insert {
                position: at(5, 2),
                text: "◎ ".into(),
                style: "s".into(),
            },
            Drawn::Line {
                line: 6,
                above: false,
                segments: Vec::new(),
                fill: None,
            },
            Drawn::Prefix {
                lines: LineSpan::of(7, 9),
                text: "│ ".into(),
                style: "s".into(),
            },
        ];
        let starts: Vec<Position> = marks.iter().map(Drawn::start).collect();
        assert_eq!(starts, [at(2, 1), at(5, 2), at(6, 0), at(7, 0)]);
        let lines: Vec<LineSpan> = marks.iter().map(Drawn::lines).collect();
        assert_eq!(
            lines,
            [
                LineSpan::one(2),
                LineSpan::one(5),
                LineSpan::one(6),
                LineSpan::of(7, 9)
            ]
        );
        let primitives: Vec<Primitive> = marks.iter().map(Drawn::primitive).collect();
        assert_eq!(
            primitives,
            [
                Primitive::Hide,
                Primitive::Insert,
                Primitive::Line,
                Primitive::Prefix
            ]
        );
    }
}
