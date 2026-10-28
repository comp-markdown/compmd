//! The conformance set (spec §13.2): every parse-only case's tree and diagnostics, every tree checked against the
//! tree schema (spec §12.4), and the diagnostics of the resolve and validate cases (stages 2 and 3).

use compmd::diagnostic::Diagnostic;
use compmd::resolve::{Files, Registry, Release, Sources, resolve};
use compmd::validate::{Options, validate};
use compmd::{ParseOptions, parse};
use semver::Version;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn conformance() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../conformance")
}

/// Every folder under `dir` that holds a `case.json`, in order.
fn cases(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .expect("a readable folder")
        .flatten()
        .map(|e| e.path())
        .collect();
    entries.sort();
    if dir.join("case.json").is_file() {
        out.push(dir.to_path_buf());
    }
    for entry in entries.into_iter().filter(|path| path.is_dir()) {
        cases(&entry, out);
    }
}

fn read_json(path: &Path) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    Some(serde_json::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display())))
}

/// The schema's parse tree, `#/$defs/parseRoot`, as the schema to validate against.
fn parse_tree_schema() -> jsonschema::Validator {
    let mut schema =
        read_json(&conformance().join("compmd-tree.schema.json")).expect("the tree schema");
    let object = schema.as_object_mut().expect("the schema is an object");
    object.remove("anyOf");
    object.insert("$ref".into(), json!("#/$defs/parseRoot"));
    jsonschema::validator_for(&schema).expect("a valid schema")
}

fn without_positions(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(without_positions).collect()),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .filter(|(key, _)| *key != "position" && *key != "closePosition")
                .map(|(key, value)| (key.clone(), without_positions(value)))
                .collect(),
        ),
        other => other.clone(),
    }
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
    let parsed = parse(&input, &options);
    let tree = serde_json::to_value(&parsed.root).expect("the tree serializes");
    let difference = read_json(&dir.join("parse.json")).and_then(|expected| {
        if case["positions"] == json!(true) {
            first_difference("", &expected, &tree)
        } else {
            first_difference("", &without_positions(&expected), &without_positions(&tree))
        }
    });
    Parsed {
        tree,
        diagnostics: parsed.diagnostics,
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

/// The files of a case's folder.
struct CaseFiles(PathBuf);

impl Files for CaseFiles {
    fn read(&self, path: &str) -> Option<String> {
        fs::read_to_string(self.0.join(path)).ok()
    }
}

/// The stubbed registry of a case's `context.json`: `registry` (pack to releases, each a folder of the case) and
/// `ambient` (pack to a folder, whose `package.json` gives the version).
struct CaseRegistry {
    dir: PathBuf,
    packs: BTreeMap<String, Vec<(Release, String)>>,
}

impl CaseRegistry {
    fn new(dir: &Path, context: &Value) -> Self {
        let mut packs: BTreeMap<String, Vec<(Release, String)>> = BTreeMap::new();
        for (pack, releases) in context["registry"].as_object().into_iter().flatten() {
            for release in releases.as_array().into_iter().flatten() {
                let version = Version::parse(release["version"].as_str().expect("a version"))
                    .expect("a semver version");
                let integrity = release["integrity"].as_str().map(str::to_string);
                let folder = release["path"].as_str().expect("a path").to_string();
                packs
                    .entry(pack.clone())
                    .or_default()
                    .push((Release { version, integrity }, folder));
            }
        }
        for (pack, folder) in context["ambient"].as_object().into_iter().flatten() {
            let folder = folder.as_str().expect("a folder").to_string();
            let manifest =
                read_json(&dir.join(&folder).join("package.json")).expect("package.json");
            let version = Version::parse(manifest["version"].as_str().expect("a version"))
                .expect("a semver version");
            let release = Release {
                version,
                integrity: None,
            };
            packs
                .entry(pack.clone())
                .or_default()
                .push((release, folder));
        }
        Self {
            dir: dir.to_path_buf(),
            packs,
        }
    }
}

impl Registry for CaseRegistry {
    fn releases(&self, pack: &str) -> Vec<Release> {
        self.packs
            .get(pack)
            .into_iter()
            .flatten()
            .map(|(release, _)| release.clone())
            .collect()
    }

    fn read(&self, pack: &str, version: &Version, path: &str) -> Option<String> {
        let (_, folder) = self
            .packs
            .get(pack)?
            .iter()
            .find(|(release, _)| release.version == *version)?;
        fs::read_to_string(self.dir.join(folder).join(path)).ok()
    }
}

/// Runs a case through stages 1 to 3; the reasons it fails, if any.
fn run_processor(dir: &Path) -> Vec<String> {
    let text = fs::read_to_string(dir.join("input.comp.md")).expect("input.comp.md");
    let context = read_json(&dir.join("context.json")).unwrap_or(Value::Null);
    let files = CaseFiles(dir.to_path_buf());
    let registry = CaseRegistry::new(dir, &context);
    let sources = Sources {
        files: &files,
        registry: &registry,
        ambient: context["ambient"]
            .as_object()
            .map(|packs| packs.keys().cloned().collect())
            .unwrap_or_default(),
        report_ambient: context["reportAmbient"] == json!(true),
    };
    let parsed = parse(&text, &ParseOptions::default());
    let resolved = resolve("input.comp.md", &text, parsed, &sources);
    let options = Options {
        allowed_origins: context["allowedOrigins"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|origin| origin.as_str().map(str::to_string))
            .collect(),
    };
    let mut diagnostics = resolved.diagnostics.clone();
    diagnostics.extend(validate(&resolved, &options));
    let expected = read_json(&dir.join("diagnostics.json"))
        .and_then(|d| d.as_array().cloned())
        .unwrap_or_default();
    let expected = sorted(expected);
    let actual = sorted(diagnostics.iter().map(conformance_form).collect());
    if expected == actual {
        return Vec::new();
    }
    let messages: Vec<String> = diagnostics
        .iter()
        .map(|d| {
            format!(
                "{} {}:{} {}",
                d.code, d.range.start.line, d.range.start.column, d.message
            )
        })
        .collect();
    vec![format!(
        "diagnostics: expected {}, got {}\n    {}",
        json!(expected),
        json!(actual),
        messages.join("\n    ")
    )]
}

/// The resolve and validate cases that need no feature beyond a processor.
#[test]
fn resolve_and_validate_cases_pass() {
    let mut dirs = Vec::new();
    for area in ["resolve", "validate"] {
        cases(&conformance().join(area), &mut dirs);
    }
    let mut failures = Vec::new();
    let mut ran = 0;
    for dir in &dirs {
        let case = read_json(&dir.join("case.json")).expect("case.json");
        if case["requires"] != json!(["processor"]) {
            continue;
        }
        ran += 1;
        let id = dir
            .strip_prefix(conformance())
            .unwrap_or(dir)
            .display()
            .to_string();
        failures.extend(
            run_processor(dir)
                .into_iter()
                .map(|failure| format!("{id}: {failure}")),
        );
    }
    assert!(ran >= 41, "only {ran} resolve and validate cases found");
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
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
