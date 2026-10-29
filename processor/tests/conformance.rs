//! The conformance set (spec §13.2): every parse-only case's tree and diagnostics, every tree checked against the
//! tree schema (spec §12.4), and every processor case through stages 1 to 4b: its diagnostics and its expanded tree.

mod common;

use common::{Setup, cases, conformance, read_json, without_positions};
use compmd::diagnostic::Diagnostic;
use compmd::{ParseOptions, Processor};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// The features this processor claims besides being one (spec §13.1): all but the host languages.
const FEATURES: [&str; 3] = ["processor", "remote-data", "origins"];

/// The host languages, whose components a renderer runs: the crate expands their invocations to nodes.
const HOST_LANGUAGES: [&str; 5] = ["jsx", "tsx", "svelte", "js", "ts"];

/// The tree schema with `root` as its root: `parseRoot` or `expandedRoot`.
fn tree_schema(root: &str) -> jsonschema::Validator {
    let mut schema =
        read_json(&conformance().join("compmd-tree.schema.json")).expect("the tree schema");
    let object = schema.as_object_mut().expect("the schema is an object");
    object.remove("anyOf");
    object.insert("$ref".into(), json!(format!("#/$defs/{root}")));
    jsonschema::validator_for(&schema).expect("a valid schema")
}

/// The schema's parse tree, `#/$defs/parseRoot`, as the schema to validate against.
fn parse_tree_schema() -> jsonschema::Validator {
    tree_schema("parseRoot")
}

/// A diagnostic as the conformance set writes it: `{code, file, line, column, related}`.
fn conformance_form(diagnostic: &Diagnostic) -> Value {
    let mut form = json!({
        "code": diagnostic.code.as_str(),
        "line": diagnostic.range.start.line,
        "column": diagnostic.range.start.column,
    });
    if let Some(file) = &diagnostic.file {
        form["file"] = json!(file);
    }
    if !diagnostic.related.is_empty() {
        let related: Vec<Value> = diagnostic
            .related
            .iter()
            .map(|related| json!({ "line": related.range.start.line, "column": related.range.start.column }))
            .collect();
        form["related"] = json!(related);
    }
    form
}

fn sorted(mut diagnostics: Vec<Value>) -> Vec<Value> {
    diagnostics.sort_by_key(|d| {
        (
            d["line"].as_u64(),
            d["column"].as_u64(),
            d["code"].as_str().map(str::to_string),
        )
    });
    diagnostics
}

/// The first place two JSON values differ: its path and both sides.
fn first_difference(path: &str, expected: &Value, actual: &Value) -> Option<String> {
    match (expected, actual) {
        (Value::Object(e), Value::Object(a)) => e.keys().chain(a.keys()).find_map(|key| {
            let at = format!("{path}/{key}");
            match (e.get(key), a.get(key)) {
                (Some(ev), Some(av)) => first_difference(&at, ev, av),
                (ev, av) => Some(format!("{at}: expected {ev:?}, got {av:?}")),
            }
        }),
        (Value::Array(e), Value::Array(a)) if e.len() == a.len() => e
            .iter()
            .zip(a)
            .enumerate()
            .find_map(|(i, (ev, av))| first_difference(&format!("{path}/{i}"), ev, av)),
        _ if expected == actual => None,
        _ => Some(format!("{path}: expected {expected}, got {actual}")),
    }
}

/// A case parsed: its tree and diagnostics, and where the tree differs from `parse.json`.
struct Parsed {
    tree: Value,
    diagnostics: Vec<Diagnostic>,
    difference: Option<String>,
}

fn parse_case(dir: &Path, case: &Value) -> Parsed {
    let input = fs::read_to_string(dir.join("input.comp.md")).expect("input.comp.md");
    let context = read_json(&dir.join("context.json")).unwrap_or(Value::Null);
    let options = ParseOptions {
        metadata: context.get("metadata").cloned(),
    };
    let parsed = Processor::new("input.comp.md", input).parse(&options);
    let tree = serde_json::to_value(parsed.tree()).expect("the tree serializes");
    let difference = read_json(&dir.join("parse.json")).and_then(|expected| {
        if case["positions"] == json!(true) {
            first_difference("", &expected, &tree)
        } else {
            first_difference("", &without_positions(&expected), &without_positions(&tree))
        }
    });
    Parsed {
        tree,
        diagnostics: parsed.diagnostics().to_vec(),
        difference,
    }
}

/// Runs one parse-only case; the reasons it fails, if any.
fn run(dir: &Path, schema: &jsonschema::Validator, case: &Value) -> Vec<String> {
    let parsed = parse_case(dir, case);
    let mut failures: Vec<String> = schema
        .iter_errors(&parsed.tree)
        .map(|error| format!("schema: {error} at {}", error.instance_path()))
        .collect();
    failures.extend(
        parsed
            .difference
            .map(|difference| format!("tree {difference}")),
    );
    let expected = read_json(&dir.join("diagnostics.json"))
        .and_then(|d| d.as_array().cloned())
        .unwrap_or_default();
    let expected = sorted(expected);
    let actual = sorted(parsed.diagnostics.iter().map(conformance_form).collect());
    if expected != actual {
        failures.push(format!(
            "diagnostics: expected {}, got {}",
            json!(expected),
            json!(actual)
        ));
    }
    failures
}

/// What running a case gave.
struct Run {
    diagnostics: Vec<Diagnostic>,
    tree: Value,
}

/// Runs a case through stages 1 to 4b: its folder's files, and the rest from its `context.json`.
fn run_stages(dir: &Path, case: &Value) -> Run {
    let setup = Setup::of(dir, case);
    let processor = setup.process("input.comp.md", &setup.files["input.comp.md"]);
    Run {
        diagnostics: processor.diagnostics().to_vec(),
        tree: serde_json::to_value(processor.expanded()).expect("the tree serializes"),
    }
}

/// Runs a processor case; the reasons it fails, if any.
fn run_processor(dir: &Path, case: &Value, schema: &jsonschema::Validator) -> Vec<String> {
    let run = run_stages(dir, case);
    let mut failures: Vec<String> = schema
        .iter_errors(&run.tree)
        .map(|error| format!("schema: {error} at {}", error.instance_path()))
        .collect();
    if let Some(expected) = read_json(&dir.join("expanded.json")) {
        let difference = if case["positions"] == json!(true) {
            first_difference("", &expected, &run.tree)
        } else {
            first_difference(
                "",
                &without_positions(&expected),
                &without_positions(&run.tree),
            )
        };
        failures.extend(difference.map(|difference| format!("expanded tree {difference}")));
    }
    let expected = read_json(&dir.join("diagnostics.json"))
        .and_then(|d| d.as_array().cloned())
        .unwrap_or_default();
    let expected = sorted(expected);
    let actual = sorted(run.diagnostics.iter().map(conformance_form).collect());
    if expected != actual {
        let messages: Vec<String> = run
            .diagnostics
            .iter()
            .map(|d| {
                format!(
                    "{} {}:{} {}",
                    d.code, d.range.start.line, d.range.start.column, d.message
                )
            })
            .collect();
        failures.push(format!(
            "diagnostics: expected {}, got {}\n    {}",
            json!(expected),
            json!(actual),
            messages.join("\n    ")
        ));
    }
    failures
}

/// Whether a case is a processor's that needs no feature beyond `features`.
fn needs_only(case: &Value, features: &[&str]) -> bool {
    let requires = case["requires"].as_array().cloned().unwrap_or_default();
    requires.contains(&json!("processor"))
        && requires
            .iter()
            .all(|feature| features.iter().any(|claimed| feature == claimed))
}

/// Every processor case that needs no host language.
#[test]
fn processor_cases_pass() {
    let ran = run_cases(|case, _| needs_only(case, &FEATURES));
    assert!(ran >= 68, "only {ran} processor cases found");
}

/// The cases that need a host language, as far as stage 4b: their invocations are nodes, which a renderer runs.
/// Those that expect W118 are left out, since only running the component finds it.
#[test]
fn host_language_cases_expand() {
    let features: Vec<&str> = FEATURES.iter().chain(&HOST_LANGUAGES).copied().collect();
    let ran = run_cases(|case, dir| {
        let w118 = read_json(&dir.join("diagnostics.json"))
            .is_some_and(|expected| expected.to_string().contains("W118"));
        !needs_only(case, &FEATURES) && needs_only(case, &features) && !w118
    });
    assert!(ran >= 3, "only {ran} host-language cases found");
}

/// Runs the cases `chosen` picks through stages 1 to 4b, and fails with what doesn't match; how many ran.
fn run_cases(chosen: impl Fn(&Value, &Path) -> bool) -> usize {
    let mut dirs = Vec::new();
    cases(&conformance(), &mut dirs);
    let schema = tree_schema("expandedRoot");
    let mut failures = Vec::new();
    let mut ran = 0;
    for dir in &dirs {
        let case = read_json(&dir.join("case.json")).expect("case.json");
        if !chosen(&case, dir) {
            continue;
        }
        ran += 1;
        let id = dir
            .strip_prefix(conformance())
            .unwrap_or(dir)
            .display()
            .to_string();
        failures.extend(
            run_processor(dir, &case, &schema)
                .into_iter()
                .map(|failure| format!("{id}: {failure}")),
        );
    }
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
    ran
}

#[test]
fn the_schema_and_the_comparison_catch_a_wrong_tree() {
    let wrong = json!({ "type": "root", "children": [
        { "type": "componentLeaf", "name": "a", "fallback": [], "params": [{ "key": "k", "breakpoint": null, "kind": "number", "value": "1" }] },
    ] });
    assert!(!parse_tree_schema().is_valid(&wrong));
    let right = json!({ "type": "root", "children": [] });
    assert_eq!(first_difference("", &right, &right), None);
    assert!(first_difference("", &right, &wrong).is_some());
    assert!(first_difference("", &json!({ "a": 1 }), &json!({})).is_some());
}

/// Every case's tree, also of the cases that need a processor: stage 1 makes it whatever the later stages do.
#[test]
fn every_parse_tree_matches() {
    let mut dirs = Vec::new();
    cases(&conformance(), &mut dirs);
    let failures: Vec<String> = dirs
        .iter()
        .filter_map(|dir| {
            let case = read_json(&dir.join("case.json")).expect("case.json");
            let difference = parse_case(dir, &case).difference?;
            Some(format!("{}: {difference}", dir.display()))
        })
        .collect();
    assert!(dirs.len() >= 174, "only {} cases found", dirs.len());
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn parse_only_cases_pass() {
    let mut dirs = Vec::new();
    cases(&conformance(), &mut dirs);
    let schema = parse_tree_schema();
    let mut failures = Vec::new();
    let mut ran = 0;
    let mut not_run: BTreeMap<String, usize> = BTreeMap::new();
    for dir in &dirs {
        let case = read_json(&dir.join("case.json")).expect("case.json");
        if case["requires"] != json!(["parser"]) {
            *not_run.entry(case["requires"].to_string()).or_default() += 1;
            continue;
        }
        ran += 1;
        let id = dir
            .strip_prefix(conformance())
            .unwrap_or(dir)
            .display()
            .to_string();
        failures.extend(
            run(dir, &schema, &case)
                .into_iter()
                .map(|failure| format!("{id}: {failure}")),
        );
    }
    println!("{ran} parse-only cases run; not run, by what they require: {not_run:?}");
    assert!(ran >= 101, "only {ran} parse-only cases found");
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
