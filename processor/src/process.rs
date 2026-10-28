//! The whole processor in one call, for a host that hands everything over at once, as a browser does: a [`Request`]
//! holds the files, the packs, the data and the render context, and [`process`] runs stages 1 to 4b on it.
//!
//! A request is JSON (`camelCase`), and so is what [`process`] and [`sources`] give.

use crate::cel::time::parse_timestamp;
use crate::context::Context;
use crate::diagnostic::Diagnostic;
use crate::host::{self, Fetched, Release, Time};
use crate::load::Source;
use crate::{Ambient, ParseOptions, Processor, expand, expanded, tree};
use semver::Version;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value as Json};
use std::collections::HashMap;

/// A document to process, with everything it needs.
///
/// ```
/// use compmd::process::Request;
///
/// let request: Request = serde_json::from_str(r##"{
///     "path": "notes.comp.md",
///     "files": { "notes.comp.md": "# {{md.doc.title}}\n" },
///     "now": "2026-10-22T09:00:00Z"
/// }"##).unwrap();
/// assert_eq!(request.tz, "UTC");
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    /// The processed document's path in `files`.
    pub path: String,
    /// The files of the document's root folder by their `/`-separated paths: the processed document, what it
    /// imports, its data.
    #[serde(default)]
    pub files: HashMap<String, String>,
    /// The registry's packs.
    #[serde(default)]
    pub packs: Packs,
    /// Packs every document imports, at the lowest priority (spec §6.3).
    #[serde(default)]
    pub ambient: Vec<String>,
    /// Whether to report each use of a name from an ambient pack (W113).
    #[serde(default)]
    pub report_ambient: bool,
    /// The origins remote data may come from, such as `https://status.example.org`.
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    /// Remote data the host fetched; [`sources`] lists the URLs a render needs.
    #[serde(default)]
    pub remote: Remote,
    /// The host's data; [`sources`] lists the sources a render needs.
    #[serde(default)]
    pub host: HostData,
    /// The render time, in nanoseconds since 1970-01-01T00:00:00Z; RFC 3339 in JSON.
    #[serde(deserialize_with = "timestamp")]
    pub now: i128,
    /// The processor's time zone: an IANA name, or `UTC`, the default.
    #[serde(default = "utc")]
    pub tz: String,
    /// The values given for the document's inputs, by name.
    #[serde(default)]
    pub inputs: Map<String, Json>,
    /// Whether expanded nodes carry their origins (spec §12.3).
    #[serde(default)]
    pub origins: bool,
}

/// The packs of a registry, by name.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct Packs(pub HashMap<String, Vec<Pack>>);

/// A release of a pack, with its files.
#[derive(Debug, Deserialize)]
pub struct Pack {
    /// Its version.
    pub version: Version,
    /// The integrity hash the registry publishes for it, `sha512-…`.
    #[serde(default)]
    pub integrity: Option<String>,
    /// Its files by path: `package.json`, the pack's document, and what it imports.
    pub files: HashMap<String, String>,
}

/// Remote data the host fetched, by URL.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct Remote(pub HashMap<String, Fetched>);

/// The host's data, by source without `host:`.
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct HostData(pub HashMap<String, Json>);

impl host::Packs for Packs {
    fn releases(&self, pack: &str) -> Vec<Release> {
        self.0
            .get(pack)
            .into_iter()
            .flatten()
            .map(|release| Release {
                version: release.version.clone(),
                integrity: release.integrity.clone(),
            })
            .collect()
    }

    fn pack_file(&self, pack: &str, version: &Version, path: &str) -> Option<String> {
        self.0
            .get(pack)?
            .iter()
            .find(|release| release.version == *version)?
            .files
            .get(path)
            .cloned()
    }
}

impl host::Fetch for Remote {
    fn fetch(&self, url: &str) -> Option<Fetched> {
        self.0.get(url).cloned()
    }
}

impl host::Data for HostData {
    fn data(&self, source: &str) -> Option<Json> {
        self.0.get(source).cloned()
    }
}

/// What processing gave.
#[derive(Debug, Serialize)]
pub struct Processed {
    /// The processed document's parse tree (spec §12.1).
    pub parse: tree::Root,
    /// The expanded tree (spec §12.3).
    pub expanded: expanded::Root,
    /// The diagnostics of every stage, in stage order.
    pub diagnostics: Vec<Diagnostic>,
}

/// The remote and `host:` sources a render needs, in the order they are written.
#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct Needed {
    /// URLs to fetch.
    pub remote: Vec<String>,
    /// `host:` sources, without `host:`.
    pub host: Vec<String>,
}

/// Why a request can't be processed.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    /// `path` isn't one of `files`.
    #[error("`{0}` isn't one of the request's files")]
    NoDocument(String),
}

/// Runs stages 1 to 4b on the request's document, with the time zones the host knows.
///
/// # Errors
///
/// [`Error::NoDocument`] when the document isn't one of the request's files.
///
/// ```
/// use compmd::process::{Request, process};
///
/// let request: Request = serde_json::from_str(r#"{
///     "path": "a.comp.md",
///     "files": { "a.comp.md": "Hello\n" },
///     "now": "2026-10-22T09:00:00Z"
/// }"#).unwrap();
/// let processed = process(&request, &()).unwrap();
/// assert_eq!(serde_json::to_value(&processed.expanded).unwrap()["children"][0]["type"], "paragraph");
/// ```
pub fn process(request: &Request, time: &dyn Time) -> Result<Processed, Error> {
    let context = request.context(time);
    let options = expand::Options {
        origins: request.origins,
        ..expand::Options::default()
    };
    let processor = Processor::new(&request.path, request.text()?)
        .parse(&ParseOptions::default())
        .resolve(&request.files, &request.packs, &request.ambient())
        .validate(&request.allowed_origins)
        .load(&context, &request.remote, &request.host)
        .expand(&options);
    Ok(processor.into())
}

/// The remote and `host:` sources a render of the request's document needs, for the host to fetch before it calls
/// [`process`].
///
/// # Errors
///
/// [`Error::NoDocument`] when the document isn't one of the request's files.
pub fn sources(request: &Request, time: &dyn Time) -> Result<Needed, Error> {
    let processor = Processor::new(&request.path, request.text()?)
        .parse(&ParseOptions::default())
        .resolve(&request.files, &request.packs, &request.ambient())
        .validate(&request.allowed_origins);
    let mut needed = Needed::default();
    for source in processor.sources(&request.context(time)) {
        match source {
            Source::Remote(url) => needed.remote.push(url),
            Source::Host(source) => needed.host.push(source),
        }
    }
    Ok(needed)
}

impl Request {
    /// The processed document's text.
    fn text(&self) -> Result<&str, Error> {
        self.files
            .get(&self.path)
            .map(String::as_str)
            .ok_or_else(|| Error::NoDocument(self.path.clone()))
    }

    fn ambient(&self) -> Ambient {
        Ambient {
            packs: self.ambient.clone(),
            report: self.report_ambient,
        }
    }

    fn context<'t>(&self, time: &'t dyn Time) -> Context<'t> {
        Context {
            now: self.now,
            tz: self.tz.clone(),
            time,
            inputs: self.inputs.clone(),
        }
    }
}

fn utc() -> String {
    "UTC".to_string()
}

fn timestamp<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i128, D::Error> {
    let text = String::deserialize(deserializer)?;
    parse_timestamp(&text)
        .ok_or_else(|| D::Error::custom(format!("`{text}` isn't an RFC 3339 time")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn request(value: Json) -> Request {
        serde_json::from_value(value).expect("a request")
    }

    #[test]
    fn requests_need_a_time_and_their_document() {
        let late = serde_json::from_value::<Request>(json!({ "path": "a", "now": "yesterday" }))
            .expect_err("an error");
        assert!(
            late.to_string()
                .contains("`yesterday` isn't an RFC 3339 time")
        );
        let missing = request(json!({ "path": "a.comp.md", "now": "2026-10-22T09:00:00Z" }));
        assert_eq!(
            process(&missing, &()).err(),
            Some(Error::NoDocument("a.comp.md".to_string()))
        );
        assert!(sources(&missing, &()).is_err());
    }

    #[test]
    fn packs_serve_their_releases() {
        use host::Packs as _;
        let packs: Packs = serde_json::from_value(json!({
            "@acme/charts": [
                { "version": "1.4.2", "integrity": "sha512-x", "files": { "package.json": "{}" } },
                { "version": "1.5.0", "files": {} }
            ]
        }))
        .expect("packs");
        let releases = packs.releases("@acme/charts");
        assert_eq!(releases.len(), 2);
        assert_eq!(releases[0].integrity.as_deref(), Some("sha512-x"));
        let version = Version::new(1, 4, 2);
        assert_eq!(
            packs.pack_file("@acme/charts", &version, "package.json"),
            Some("{}".to_string())
        );
        assert_eq!(
            packs.pack_file("@acme/charts", &Version::new(2, 0, 0), "a"),
            None
        );
        assert_eq!(packs.releases("@acme/maps"), Vec::new());
    }

    #[test]
    fn sources_list_what_the_host_fetches() {
        let text = "---\nmd:\n  version: 1\n  data:\n    - from: https://status.example.org/{{md.input.region}}.json\n      as: status\n    - from: host:tasks?open=true\n      as: tasks\n  inputs:\n    region: eu | us = eu\n---\nText.\n";
        let request = request(json!({
            "path": "a.comp.md",
            "files": { "a.comp.md": text },
            "allowedOrigins": ["https://status.example.org"],
            "now": "2026-10-22T09:00:00Z",
            "inputs": { "region": "us" }
        }));
        assert_eq!(
            sources(&request, &()),
            Ok(Needed {
                remote: vec!["https://status.example.org/us.json".to_string()],
                host: vec!["tasks?open=true".to_string()],
            })
        );
    }
}
