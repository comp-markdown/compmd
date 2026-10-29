//! The request `process`, `sources` and an open document take, and what `process` and `sources` give: JSON
//! (`camelCase`), as `src/index.ts` types it.

use compmd::context::{RenderContext, parse_time};
use compmd::diagnostic::Diagnostic;
use compmd::expand::Options;
use compmd::host::{Fetched, LoadedRelease, Time};
use compmd::processor::{Expanded, Validated};
use compmd::{Ambient, ParseOptions, Processor, expanded, tree};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value as Json};
use std::collections::HashMap;
use std::rc::Rc;

/// A document to process, with everything it needs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    /// The processed document's path in `files`.
    path: String,
    /// The files of the document's root folder by their `/`-separated paths.
    #[serde(default)]
    files: HashMap<String, String>,
    /// The registry's packs, by name.
    #[serde(default)]
    packs: HashMap<String, Vec<LoadedRelease>>,
    /// Packs every document imports, at the lowest priority (spec §6.3).
    #[serde(default)]
    ambient: Vec<String>,
    /// Whether to report each use of a name from an ambient pack (W113).
    #[serde(default)]
    report_ambient: bool,
    /// The origins remote data may come from.
    #[serde(default)]
    allowed_origins: Vec<String>,
    /// Remote data fetched beforehand, by URL.
    #[serde(default)]
    remote: HashMap<String, Fetched>,
    /// The host's data, by source without `host:`.
    #[serde(default)]
    host: HashMap<String, Json>,
    /// The render time, RFC 3339.
    #[serde(deserialize_with = "time")]
    now: i128,
    /// The processor's time zone: an IANA name, or `UTC`, the default.
    #[serde(default = "utc")]
    tz: String,
    /// The values given for the document's inputs, by name.
    #[serde(default)]
    inputs: Map<String, Json>,
    /// Whether expanded nodes carry their origins (spec §12.3).
    #[serde(default)]
    origins: bool,
}

/// What processing gave.
#[derive(Serialize)]
struct Processed<'a> {
    parse: &'a tree::Root,
    expanded: &'a expanded::Root,
    diagnostics: &'a [Diagnostic],
}

/// A request's document through stage 3, with what loading and expanding it take.
pub(crate) struct Prepared {
    /// The document's text.
    pub(crate) text: String,
    pub(crate) validated: Processor<Validated>,
    pub(crate) context: RenderContext,
    remote: HashMap<String, Fetched>,
    host: HashMap<String, Json>,
    options: Options,
}

/// Why a request can't be processed.
#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    /// The text isn't a request.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// `path` isn't one of `files`.
    #[error("`{0}` isn't one of the request's files")]
    NoDocument(String),
}

/// Runs stages 1 to 4b on the request in `json`: `{parse, expanded, diagnostics}`.
pub(crate) fn process(json: &str, time: Rc<dyn Time>) -> Result<String, Error> {
    let processor = prepare(json, time)?.expand();
    let processed = Processed {
        parse: processor.tree(),
        expanded: processor.expanded(),
        diagnostics: processor.diagnostics(),
    };
    Ok(serde_json::to_string(&processed)?)
}

/// What the frontmatter of the request in `json` gives its render: `{doc, settings, data, inputs}`.
pub(crate) fn frontmatter(json: &str, time: Rc<dyn Time>) -> Result<String, Error> {
    let processor = prepare(json, time)?.expand();
    Ok(serde_json::to_string(&processor.frontmatter())?)
}

/// The remote and `host:` sources a render of the request in `json` needs: `{remote, host}`.
pub(crate) fn sources(json: &str, time: Rc<dyn Time>) -> Result<String, Error> {
    let prepared = prepare(json, time)?;
    let needed = prepared.validated.sources(&prepared.context);
    Ok(serde_json::to_string(&needed)?)
}

/// The request in `json` through stage 3.
pub(crate) fn prepare(json: &str, time: Rc<dyn Time>) -> Result<Prepared, Error> {
    let request: Request = serde_json::from_str(json)?;
    request.prepare(time)
}

impl Request {
    fn prepare(self, time: Rc<dyn Time>) -> Result<Prepared, Error> {
        let text = self
            .files
            .get(&self.path)
            .cloned()
            .ok_or_else(|| Error::NoDocument(self.path.clone()))?;
        let ambient = Ambient {
            packs: self.ambient,
            report: self.report_ambient,
        };
        let validated = Processor::new(self.path, text.as_str())
            .parse(&ParseOptions::default())
            .resolve(self.files, self.packs, &ambient)
            .validate(&self.allowed_origins);
        Ok(Prepared {
            text,
            validated,
            context: RenderContext {
                now: self.now,
                tz: self.tz,
                time,
                inputs: self.inputs,
            },
            remote: self.remote,
            host: self.host,
            options: Options {
                origins: self.origins,
                ..Options::default()
            },
        })
    }
}

impl Prepared {
    /// Stages 4a and 4b.
    pub(crate) fn expand(self) -> Processor<Expanded> {
        self.validated
            .load(self.context, self.remote, self.host)
            .expand(&self.options)
    }
}

fn utc() -> String {
    "UTC".to_string()
}

fn time<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i128, D::Error> {
    let text = String::deserialize(deserializer)?;
    parse_time(&text).ok_or_else(|| D::Error::custom(format!("`{text}` isn't an RFC 3339 time")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request(value: &Json) -> Request {
        serde_json::from_value(value.clone()).expect("a request")
    }

    #[test]
    fn requests_need_a_time_and_their_document() {
        let late =
            process(r#"{"path": "a", "now": "yesterday"}"#, Rc::new(())).expect_err("an error");
        assert!(
            late.to_string()
                .contains("`yesterday` isn't an RFC 3339 time")
        );
        let missing = json!({ "path": "a.comp.md", "now": "2026-10-22T09:00:00Z" }).to_string();
        let error = process(&missing, Rc::new(())).expect_err("an error");
        assert_eq!(
            error.to_string(),
            "`a.comp.md` isn't one of the request's files"
        );
        assert!(sources(&missing, Rc::new(())).is_err());
        assert!(frontmatter(&missing, Rc::new(())).is_err());
    }

    #[test]
    fn a_request_gives_its_frontmatter() {
        let request = json!({
            "path": "a.comp.md",
            "files": { "a.comp.md": "---\ntitle: T\nmd:\n  data:\n    - from: ./n.json\n      as: n\n---\n", "n.json": "[1, 2]" },
            "now": "2026-10-22T09:00:00Z"
        });
        let given: Json = serde_json::from_str(
            &frontmatter(&request.to_string(), Rc::new(())).expect("frontmatter"),
        )
        .expect("JSON");
        assert_eq!(given["doc"]["title"], "T");
        assert_eq!(given["data"][0]["value"], json!([1, 2]));
    }

    #[test]
    fn a_request_gives_both_trees_and_the_diagnostics() {
        let request = json!({
            "path": "a.comp.md",
            "files": { "a.comp.md": "Hello {{md.input.who}}\n\n:: nope[N]\n" },
            "now": "2026-10-22T09:00:00Z",
            "origins": true
        });
        let processed: Json =
            serde_json::from_str(&process(&request.to_string(), Rc::new(())).expect("processed"))
                .expect("JSON");
        assert_eq!(processed["parse"]["children"][0]["type"], "paragraph");
        assert_eq!(processed["expanded"]["children"][0]["type"], "paragraph");
        assert!(processed["expanded"]["children"][0]["origin"].is_object());
        assert!(
            processed["diagnostics"]
                .as_array()
                .is_some_and(|d| !d.is_empty())
        );
    }

    #[test]
    fn requests_map_to_the_processors_parts() {
        let request = request(&json!({
            "path": "a.comp.md",
            "packs": { "@acme/kit": [{ "version": "1.0.0", "files": {} }] },
            "ambient": ["@acme/kit"],
            "reportAmbient": true,
            "remote": { "https://a.org/x.json": { "text": "{}", "contentType": "application/json" } },
            "host": { "tasks": [] },
            "now": "1970-01-01T00:00:01Z",
            "tz": "Europe/Berlin",
            "inputs": { "region": "us" }
        }));
        assert_eq!(
            request.packs["@acme/kit"][0].release.version.to_string(),
            "1.0.0"
        );
        assert!(request.report_ambient);
        assert_eq!(
            request.remote["https://a.org/x.json"]
                .content_type
                .as_deref(),
            Some("application/json")
        );
        assert_eq!(request.host["tasks"], json!([]));
        let context = Request {
            files: HashMap::from([("a.comp.md".into(), String::new())]),
            ..request
        }
        .prepare(Rc::new(()))
        .expect("prepared")
        .context;
        assert_eq!(context.now, 1_000_000_000);
        assert_eq!(context.tz, "Europe/Berlin");
        assert_eq!(context.inputs["region"], "us");
        let defaults = self::request(&json!({ "path": "a", "now": "2026-10-22T09:00:00Z" }));
        assert_eq!(defaults.tz, "UTC");
        assert!(!defaults.origins);
    }

    #[test]
    fn sources_list_what_the_host_fetches() {
        let text = "---\nmd:\n  version: 1\n  data:\n    - from: https://status.example.org/{{md.input.region}}.json\n      as: status\n    - from: host:tasks?open=true\n      as: tasks\n  inputs:\n    region: eu | us = eu\n---\nText.\n";
        let request = json!({
            "path": "a.comp.md",
            "files": { "a.comp.md": text },
            "allowedOrigins": ["https://status.example.org"],
            "now": "2026-10-22T09:00:00Z",
            "inputs": { "region": "us" }
        });
        let needed: Json =
            serde_json::from_str(&sources(&request.to_string(), Rc::new(())).expect("sources"))
                .expect("JSON");
        assert_eq!(
            needed,
            json!({ "remote": ["https://status.example.org/us.json"], "host": ["tasks?open=true"] })
        );
    }
}
