//! Stages 4a and 4b beyond the conformance set: data from packs, URLs and hosts, inputs and their defaults, every
//! mdast node, host components with slots, origins, and what fails, with the diagnostics as `(code, line, column)`.
//! This file holds the render the tests share; each module is one group.

#[path = "../common.rs"]
mod common;

mod components;
mod data;
mod host_components;
mod tree;

use common::place;
use compmd::context::RenderContext;
use compmd::expand;
use compmd::host::{CacheKey, Fetched, LoadedRelease, Needed, Release};
use compmd::{Ambient, Language, ParseOptions, Processor};
use semver::Version;
use serde_json::Value;
use std::collections::HashMap;
use std::rc::Rc;

/// What a render is given besides the document.
#[derive(Default)]
struct Setup {
    files: Vec<(&'static str, &'static str)>,
    /// The files of one pack, `@acme/kit` 1.0.0.
    kit: Vec<(&'static str, &'static str)>,
    web: Vec<(&'static str, &'static str, Option<&'static str>)>,
    host: Value,
    inputs: Value,
    allowed: Vec<&'static str>,
    options: expand::Options,
}

/// What a render gave: the tree as JSON, the diagnostics of every stage, and the sources the host provides.
struct Rendered {
    tree: Value,
    found: Vec<(String, usize, usize)>,
    files: Vec<Option<String>>,
    sources: Needed,
    /// The host-language components, by name and language.
    components: Vec<(String, Language)>,
    /// Whether processing failed, in the setup's mode.
    failed: bool,
    cache_keys: Vec<CacheKey>,
}

fn render(input: &str, setup: Setup) -> Rendered {
    let mut files: HashMap<String, String> = setup
        .files
        .iter()
        .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
        .collect();
    files.insert("input.comp.md".into(), input.to_string());
    let kit = LoadedRelease {
        release: Release {
            version: Version::new(1, 0, 0),
            integrity: None,
            broken: None,
        },
        files: setup
            .kit
            .iter()
            .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
            .collect(),
    };
    let packs = HashMap::from([("@acme/kit".to_string(), vec![kit])]);
    let web: HashMap<String, Fetched> = setup
        .web
        .iter()
        .map(|(url, text, content_type)| {
            let fetched = Fetched {
                text: (*text).to_string(),
                content_type: content_type.map(str::to_string),
            };
            ((*url).to_string(), fetched)
        })
        .collect();
    let host: HashMap<String, Value> = match setup.host {
        Value::Object(sources) => sources.into_iter().collect(),
        _ => HashMap::new(),
    };
    let allowed: Vec<String> = setup
        .allowed
        .iter()
        .map(|origin| (*origin).to_string())
        .collect();
    let context = RenderContext {
        now: 1_792_576_800 * 1_000_000_000,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: setup.inputs.as_object().cloned().unwrap_or_default(),
    };
    let validated = Processor::new("input.comp.md", input)
        .parse(&ParseOptions::default())
        .resolve(files, packs, &Ambient::default())
        .validate(&allowed);
    let needed = validated.sources(&context);
    let expanded = validated.load(context, web, host).expand(&setup.options);
    let diagnostics = expanded.diagnostics();
    let mut found: Vec<(String, usize, usize)> = diagnostics.iter().map(place).collect();
    found.sort();
    Rendered {
        tree: serde_json::to_value(expanded.expanded()).expect("the tree serializes"),
        found,
        files: diagnostics.iter().map(|d| d.file.clone()).collect(),
        sources: needed,
        components: expanded
            .host_components()
            .into_iter()
            .map(|c| (c.name, c.language))
            .collect(),
        failed: expanded.failed(),
        cache_keys: expanded.cache_keys(),
    }
}

/// A render context with no time, the UTC zone and no inputs, for the tests that run the stages themselves.
fn context() -> RenderContext {
    RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    }
}

/// The text of a tree, its text nodes in order, joined by `|` between blocks.
fn text(tree: &Value) -> String {
    fn inline(node: &Value, out: &mut String) {
        if let Some(value) = node["value"].as_str().filter(|_| node["type"] == "text") {
            out.push_str(value);
        }
        for child in node["children"].as_array().into_iter().flatten() {
            inline(child, out);
        }
    }
    tree["children"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|block| {
            let mut out = String::new();
            inline(block, &mut out);
            out
        })
        .collect::<Vec<_>>()
        .join("|")
}

/// The types of a tree's top-level blocks.
fn types(tree: &Value) -> Vec<String> {
    tree["children"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|block| block["type"].as_str().unwrap_or_default().to_string())
        .collect()
}
