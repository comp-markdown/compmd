//! The processor as a host uses it, one scenario a test: what the host gives it (files, packs, a policy, data, a
//! render context), what it asks the host for, and what a document gets when something is wrong. Each test reads as
//! an example of the public API, a good use or a bad one. This file holds the host the tests share; each module is
//! one group of uses.

use compmd::context::{RenderContext, parse_time};
use compmd::expand::Options;
use compmd::host::{Data, Fetch, LoadedRelease, Release, Time, WithPending};
use compmd::policy::Policy;
use compmd::processor::{Expanded, Validated};
use compmd::{Ambient, ParseOptions, Processor};
use semver::Version;
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

mod diagnostics_and_their_stages;
mod edits_and_reloads;
mod packs;
mod policy;
mod the_render_context;
mod what_the_processor_asks_the_host;

/// What a host gives the processor; each test changes what it needs.
struct Host {
    files: HashMap<String, String>,
    packs: HashMap<String, Vec<LoadedRelease>>,
    /// The packs the host is still getting.
    pending: HashSet<String>,
    policy: Policy,
    ambient: Ambient,
    parse: ParseOptions,
    allowed_origins: Vec<String>,
    remote: Rc<dyn Fetch>,
    data: Rc<dyn Data>,
    context: RenderContext,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            files: HashMap::new(),
            packs: HashMap::new(),
            pending: HashSet::new(),
            policy: Policy::default(),
            ambient: Ambient::default(),
            parse: ParseOptions::default(),
            allowed_origins: Vec::new(),
            remote: Rc::new(()),
            data: Rc::new(()),
            context: RenderContext {
                now: parse_time("2026-11-02T08:50:00Z").expect("a time"),
                tz: "UTC".into(),
                time: Rc::new(Zones),
                inputs: serde_json::Map::new(),
            },
        }
    }
}

impl Host {
    /// `text`, as `a.comp.md`, through stage 3: what `sources` is asked of before loading.
    fn validate(&self, text: &str) -> Processor<Validated> {
        Processor::new("a.comp.md", text)
            .parse(&self.parse)
            .resolve_with(
                self.files.clone(),
                WithPending {
                    held: self.packs.clone(),
                    pending: self.pending.clone(),
                },
                &self.ambient,
                &self.policy,
            )
            .validate(&self.allowed_origins)
    }

    /// `text`, as `a.comp.md`, through every stage.
    fn process(&self, text: &str) -> Processor<Expanded> {
        self.validate(text)
            .load(
                self.context.clone(),
                Rc::clone(&self.remote),
                Rc::clone(&self.data),
            )
            .expand(&Options::default())
    }

    /// Adds `release` to the packs.
    fn with(mut self, name: &str, release: LoadedRelease) -> Self {
        self.packs
            .entry(name.to_string())
            .or_default()
            .push(release);
        self
    }
}

/// The time zones the tests use.
struct Zones;

impl Time for Zones {
    fn offset(&self, zone: &str, _at: i64) -> Option<i32> {
        match zone {
            "UTC" => Some(0),
            "Asia/Tokyo" => Some(9 * 3600),
            _ => None,
        }
    }
}

/// A release of the pack `name` at `version` whose document is `document`, with `manifest` added to its
/// `package.json`, and an integrity of its own: `@acme/kit` 1.2.0's is `sha512-acme/kit120`.
fn release(name: &str, version: &str, document: &str, manifest: Value) -> LoadedRelease {
    let mut package = json!({ "name": name, "version": version });
    if let (Some(package), Value::Object(more)) = (package.as_object_mut(), manifest) {
        package.extend(more);
    }
    let digest: String = format!("{name}{version}")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '/')
        .collect();
    LoadedRelease {
        release: Release {
            version: Version::parse(version).expect("a version"),
            integrity: Some(format!("sha512-{digest}").parse().expect("an integrity")),
            broken: None,
        },
        files: HashMap::from([
            ("package.json".to_string(), package.to_string()),
            ("index.comp.md".to_string(), document.to_string()),
        ]),
    }
}

/// A pack's document that exports one `compmd` component, `card`, showing `text`.
fn card_pack(text: &str) -> String {
    format!(
        "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=card\n---\n{text}\n```\n"
    )
}

/// A pack's document that exports one JavaScript component, `name`, with its signature.
fn js_pack(name: &str) -> String {
    format!(
        "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```js component={name}\nexport default () => {{}};\n```\n\n```compmd component={name} signature\nparams: {{}}\n```\n"
    )
}

/// A document that imports `names` from each of `froms`, then uses `body`.
fn importing(froms: &[&str], body: &str) -> String {
    let imports: String = froms
        .iter()
        .flat_map(|from| ["    - from: \"", from, "\"\n      names: \"*\"\n"])
        .collect();
    format!("---\nmd:\n  import:\n{imports}---\n\n{body}")
}

/// The diagnostics as `code message`.
fn diagnostics<S>(processor: &Processor<S>) -> Vec<String> {
    processor
        .diagnostics()
        .iter()
        .map(|diagnostic| format!("{} {}", diagnostic.code, diagnostic.message))
        .collect()
}

/// The codes of the diagnostics.
fn codes<S>(processor: &Processor<S>) -> Vec<String> {
    processor
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code.to_string())
        .collect()
}

/// Every text the expanded tree shows, joined.
fn shown(processor: &Processor<Expanded>) -> String {
    fn collect(value: &Value, out: &mut String) {
        match value {
            Value::Object(node) => {
                if node.get("type") == Some(&json!("text")) {
                    out.push_str(node["value"].as_str().unwrap_or_default());
                }
                for child in node.values() {
                    collect(child, out);
                }
            }
            Value::Array(items) => items.iter().for_each(|item| collect(item, out)),
            _ => {}
        }
    }
    let mut out = String::new();
    collect(
        &serde_json::to_value(processor.expanded()).expect("a tree"),
        &mut out,
    );
    out
}
