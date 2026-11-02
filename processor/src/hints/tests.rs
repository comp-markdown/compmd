//! The hints at a caret: what each place in a document says, in three groups: names (components, code renderers,
//! slots), params (keys, values, what they take), and the places with no hint or only a diagnostic.

use super::*;
use crate::context::RenderContext;
use crate::expand::Options;
use crate::{Ambient, ParseOptions, Processor};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::rc::Rc;

mod names;
mod params;
mod places;

/// The caret in the texts of these tests.
pub(super) const CARET: char = '‸';

/// The hint at the caret of `text`, with `files`, as JSON.
pub(super) fn hint_with(text: &str, files: &[(&str, &str)]) -> Value {
    let offset = text.chars().position(|c| c == CARET).expect("a caret");
    let text = text.replacen(CARET, "", 1);
    let files: HashMap<String, String> = files
        .iter()
        .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
        .collect();
    let context = RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::default(),
    };
    let processor = Processor::new("a.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve(files, (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&Options::default());
    serde_json::to_value(processor.hint_at(offset)).expect("JSON")
}

pub(super) fn hint(text: &str) -> Value {
    hint_with(text, &[])
}

/// The range on `line` from `start` to `end`, columns from 1.
pub(super) fn range(line: usize, start: usize, end: usize) -> Value {
    json!({ "start": { "line": line, "column": start }, "end": { "line": line, "column": end } })
}

/// The texts of a hint's completions.
pub(super) fn texts(hint: &Value) -> Vec<&str> {
    hint["completions"]
        .as_array()
        .expect("completions")
        .iter()
        .map(|completion| completion["text"].as_str().expect("a text"))
        .collect()
}

/// The names of a `params` subject's params.
fn names(hint: &Value) -> Vec<&str> {
    hint["subject"]["params"]
        .as_array()
        .expect("params")
        .iter()
        .map(|param| param["name"].as_str().expect("a name"))
        .collect()
}

pub(super) const CARD: &str = "```compmd component=card\nparams:\n  kind: note | warning = note\n  title:\n    type: string\n    doc: The card's heading.\nslots: [default, footer]\n---\n{{title}}\n```\n\n";
