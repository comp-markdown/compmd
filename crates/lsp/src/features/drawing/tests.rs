//! The drawing's test set ([a document drawn in the editor], decision 12): documents and the plan each gives, by the
//! primitives the client draws and the settings. A plan is written one mark a line, as `describe` writes it.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use lsp_types::Position;
use serde_json::json;

use super::mark::Drawn;
use super::primitive::{Primitive, Primitives};
use super::*;
use crate::features::testing::{asking, processed};
use crate::settings::RenderSettings;

mod compmd;
mod fallbacks;
mod markdown;
mod plan;

/// Every primitive.
fn all() -> Primitives {
    Primitive::ALL.into_iter().collect()
}

/// Every primitive but `without`.
fn all_but(without: &[Primitive]) -> Primitives {
    Primitive::ALL
        .into_iter()
        .filter(|primitive| !without.contains(primitive))
        .collect()
}

/// The settings `value` writes, as `compmd.render`.
fn render(value: serde_json::Value) -> RenderSettings {
    serde_json::from_value(value).expect("render settings")
}

/// The whole of `text`'s plan with `primitives` and `render`.
fn plan_with(text: &str, primitives: Primitives, render: RenderSettings) -> Plan {
    let everything = Range::new(Position::new(0, 0), Position::new(u32::MAX, 0));
    plan_of(text, everything, &Planning { primitives, render })
}

/// The plan of `range` of `text`, made with `planning`.
fn plan_of(text: &str, range: Range, planning: &Planning) -> Plan {
    let processor = processed(text);
    asking(text, &processor, &[], |asked| plan(range, planning, asked))
}

/// `text`'s plan with `primitives` and the default settings, one mark a line.
fn drawn(text: &str, primitives: Primitives) -> Vec<String> {
    described(&plan_with(text, primitives, RenderSettings::default()))
}

/// `text`'s plan with `primitives`, a code block's language above it (`code.language: above`), one mark a line.
fn drawn_above(text: &str, primitives: Primitives) -> Vec<String> {
    let above = render(serde_json::json!({ "code": { "language": "above" } }));
    described(&plan_with(text, primitives, above))
}

/// `text`'s plan with every primitive and the settings `value` writes, one mark a line.
fn drawn_with(text: &str, value: serde_json::Value) -> Vec<String> {
    described(&plan_with(text, all(), render(value)))
}

fn described(plan: &Plan) -> Vec<String> {
    plan.marks.iter().map(describe).collect()
}

/// A mark in a line: its kind, where it is, what it draws, and the lines it gives way on.
fn describe(mark: &Mark) -> String {
    let at = |position: &Position| format!("{}:{}", position.line, position.character);
    let range = |range: &Range| format!("{}-{}", at(&range.start), at(&range.end));
    let lines = |lines: &LineSpan| {
        if lines.first == lines.last {
            lines.first.to_string()
        } else {
            format!("{}-{}", lines.first, lines.last)
        }
    };
    let segment = |segment: &mark::Segment| format!("{:?} {}", segment.text, segment.style);
    let written = match &mark.drawn {
        Drawn::Hide { range: hidden } => format!("hide {}", range(hidden)),
        Drawn::Replace {
            range: replaced,
            text,
            style,
        } => {
            format!("replace {} {text:?} {style}", range(replaced))
        }
        Drawn::Insert {
            position,
            text,
            style,
        } => format!("insert {} {text:?} {style}", at(position)),
        Drawn::Line {
            line,
            above,
            segments,
            fill,
        } => {
            let segments: Vec<String> = segments.iter().map(segment).collect();
            let fill = fill
                .as_ref()
                .map(|fill| format!(" fill {}", segment(fill)))
                .unwrap_or_default();
            let place = if *above { "above" } else { "below" };
            format!("line {place} {line} [{}]{fill}", segments.join(", "))
        }
        Drawn::Prefix {
            lines: prefixed,
            text,
            style,
        } => format!("prefix {} {text:?} {style}", lines(prefixed)),
        Drawn::Style {
            range: styled,
            style,
            whole_line,
        } => {
            let whole = if *whole_line { " whole" } else { "" };
            format!("style {} {style}{whole}", range(styled))
        }
    };
    match &mark.reveal {
        Some(reveal) => format!("{written} reveal {}", lines(reveal)),
        None => written,
    }
}

#[test]
fn a_plan_is_its_version_and_its_marks_in_json() {
    let plan = plan_with("# A\n", all(), RenderSettings::default());
    let value = serde_json::to_value(&plan).expect("JSON");
    assert_eq!(value["version"], json!(1));
    assert_eq!(value["marks"][0]["kind"], json!("replace"));
    assert_eq!(
        value["marks"][0]["reveal"],
        json!({ "first": 0, "last": 0 })
    );
}
