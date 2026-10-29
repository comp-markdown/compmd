//! Helpers the test files share. Each file uses some of them, and cargo also builds this file as a test of its own.
#![allow(
    dead_code,
    reason = "each test file uses some of the helpers, and this file's own build none"
)]

use compmd::context::{RenderContext, parse_time};
use compmd::diagnostic::Diagnostic;
use compmd::expand::Options;
use compmd::host::{Fetched, LoadedRelease, Time};
use compmd::processor::Expanded;
use compmd::{Ambient, ParseOptions, Processor};
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A diagnostic as `(code, line, column)`.
pub(crate) fn at(code: &str, line: usize, column: usize) -> (String, usize, usize) {
    (code.to_string(), line, column)
}

/// Where a diagnostic is, as [`at`] writes it.
pub(crate) fn place(diagnostic: &Diagnostic) -> (String, usize, usize) {
    let start = diagnostic.range.start;
    (diagnostic.code.to_string(), start.line, start.column)
}

/// `value` without positions, at any depth.
pub(crate) fn without_positions(value: &Value) -> Value {
    without(value, &["position", "closePosition"])
}

/// `value` without the fields `keys`, at any depth.
pub(crate) fn without(value: &Value, keys: &[&str]) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(|item| without(item, keys)).collect()),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .filter(|(key, _)| !keys.contains(&key.as_str()))
                .map(|(key, value)| (key.clone(), without(value, keys)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// The conformance set's folder.
pub(crate) fn conformance() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../conformance")
}

/// Every folder under `dir` that holds a `case.json`, in order.
pub(crate) fn cases(dir: &Path, out: &mut Vec<PathBuf>) {
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

pub(crate) fn read_json(path: &Path) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    Some(serde_json::from_str(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display())))
}

/// Every file under `dir`, by its `/`-separated path from `dir`.
pub(crate) fn files_of(dir: &Path) -> HashMap<String, String> {
    fn add(root: &Path, dir: &Path, files: &mut HashMap<String, String>) {
        for path in fs::read_dir(dir)
            .expect("a readable folder")
            .flatten()
            .map(|e| e.path())
        {
            if path.is_dir() {
                add(root, &path, files);
            } else if let Ok(text) = fs::read_to_string(&path) {
                let name = path.strip_prefix(root).expect("inside the folder");
                files.insert(name.to_string_lossy().replace('\\', "/"), text);
            }
        }
    }
    let mut files = HashMap::new();
    add(dir, dir, &mut files);
    files
}

/// The packs of a case's `context.json`: `registry` (pack to releases, each a folder of the case) and `ambient` (pack
/// to a folder, whose `package.json` gives the version).
fn packs(dir: &Path, context: &Value) -> HashMap<String, Vec<LoadedRelease>> {
    let loaded = |release: &Value, folder: &Path| LoadedRelease {
        release: serde_json::from_value(release.clone()).expect("a release"),
        files: files_of(folder),
    };
    let mut packs = HashMap::new();
    for (pack, releases) in context["registry"].as_object().into_iter().flatten() {
        let releases = releases.as_array().into_iter().flatten().map(|release| {
            loaded(
                release,
                &dir.join(release["path"].as_str().expect("a path")),
            )
        });
        packs.insert(pack.clone(), releases.collect());
    }
    for (pack, folder) in context["ambient"].as_object().into_iter().flatten() {
        let folder = dir.join(folder.as_str().expect("a folder"));
        let manifest = read_json(&folder.join("package.json")).expect("package.json");
        packs.insert(pack.clone(), vec![loaded(&manifest, &folder)]);
    }
    packs
}

/// Remote data as a case's `context.json` maps it: `urls`, each URL to a file of the case, served with the
/// `Content-Type` of its extension.
fn remote(dir: &Path, context: &Value) -> HashMap<String, Fetched> {
    let fetched = context["urls"].as_object().into_iter().flatten();
    let fetched = fetched.filter_map(|(url, file)| {
        let file = file.as_str()?;
        let content_type = match file.rsplit_once('.').map(|(_, extension)| extension) {
            Some("json") => Some("application/json"),
            Some("csv") => Some("text/csv"),
            Some("yaml") => Some("application/yaml"),
            _ => None,
        };
        let text = fs::read_to_string(dir.join(file)).ok()?;
        Some((
            url.clone(),
            Fetched {
                text,
                content_type: content_type.map(str::to_string),
            },
        ))
    });
    fetched.collect()
}

/// A host's data as a case's `context.json` maps it: `host`, each source to a JSON file of the case.
fn host(dir: &Path, context: &Value) -> HashMap<String, Value> {
    let sources = context["host"].as_object().into_iter().flatten();
    sources
        .filter_map(|(source, file)| Some((source.clone(), read_json(&dir.join(file.as_str()?))?)))
        .collect()
}

/// The time zones of the cases, at their offsets in late October 2026.
struct Zones;

impl Time for Zones {
    fn offset(&self, zone: &str, _at: i64) -> Option<i32> {
        match zone {
            "UTC" => Some(0),
            "Europe/Berlin" => Some(2 * 3600),
            "Asia/Tokyo" => Some(9 * 3600),
            _ => None,
        }
    }
}

/// What a document of a folder is processed with: the folder's files, and the rest from its `context.json`.
pub(crate) struct Setup {
    pub(crate) files: HashMap<String, String>,
    packs: HashMap<String, Vec<LoadedRelease>>,
    ambient: Ambient,
    allowed_origins: Vec<String>,
    render: RenderContext<'static>,
    remote: HashMap<String, Fetched>,
    host: HashMap<String, Value>,
    pub(crate) options: Options,
}

impl Setup {
    /// The setup of the folder `dir`; with origins when `case` requires them.
    pub(crate) fn of(dir: &Path, case: &Value) -> Self {
        let context = read_json(&dir.join("context.json")).unwrap_or(Value::Null);
        let strings = |key: &str| -> Vec<String> {
            let items = context[key].as_array().into_iter().flatten();
            items
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        };
        Self {
            files: files_of(dir),
            packs: packs(dir, &context),
            ambient: Ambient {
                packs: context["ambient"]
                    .as_object()
                    .into_iter()
                    .flat_map(Map::keys)
                    .cloned()
                    .collect(),
                report: context["reportAmbient"] == json!(true),
            },
            allowed_origins: strings("allowedOrigins"),
            render: RenderContext {
                now: parse_time(context["now"].as_str().unwrap_or("2026-01-01T00:00:00Z"))
                    .expect("a time"),
                tz: context["tz"].as_str().unwrap_or("UTC").to_string(),
                time: &Zones,
                inputs: context["inputs"].as_object().cloned().unwrap_or_default(),
            },
            remote: remote(dir, &context),
            host: host(dir, &context),
            options: Options {
                origins: case["requires"]
                    .as_array()
                    .is_some_and(|features| features.contains(&json!("origins"))),
                ..Options::default()
            },
        }
    }

    /// `text`, as the document `path`, through stages 1 to 4b.
    pub(crate) fn process(&self, path: &str, text: &str) -> Processor<Expanded<'_, '_>> {
        Processor::new(path, text)
            .parse(&ParseOptions::default())
            .resolve(&self.files, &self.packs, &self.ambient)
            .validate(&self.allowed_origins)
            .load(&self.render, &self.remote, &self.host)
            .expand(&self.options)
    }
}
