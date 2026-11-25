//! The configuration ([the Vite plugin], decisions 1 and 2): `compmd.config.json`, or `package.json`'s `compmd` object,
//! written for people, read and checked as `@compmd/vite`'s `readConfig` reads it, with every default filled in.
//!
//! [the Vite plugin]: https://github.com/comp-markdown/compmd/blob/main/docs/vite.md

use std::collections::BTreeMap;

use compmd::policy::{Deny, Imports, Trust};
use semver::VersionReq;
use serde_json::{Map, Value};

use crate::config_error::ConfigError;
use crate::names::{is_pack_name, is_package_name, is_scope};
use crate::origin::{origin_of, registry_url_of};
use crate::size::bytes_of;

/// The configuration's keys, in the order its schema lists them.
const KEYS: [&str; 10] = [
    "$schema",
    "packs",
    "imports",
    "deny",
    "maxPackSize",
    "libraries",
    "registries",
    "fetch",
    "origins",
    "editor",
];

/// A pack's options.
const PACK_KEYS: [&str; 5] = ["trust", "network", "host", "ambient", "version"];

/// The editor's options.
const EDITOR_KEYS: [&str; 1] = ["suggest"];

/// The configuration, read and checked.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Config {
    /// The packs it lists, by pack name (`@acme/maps`) or folder path (`./components`), in the order it writes them.
    pub packs: Vec<(String, ListedPack)>,
    /// Which packs a document may import: any installed one (the default), or only those listed.
    pub imports: Imports,
    /// The packs denied, all versions or those in a range, with the reason readers see.
    pub deny: Vec<Deny>,
    /// In bytes: a pack whose files are larger is left out. `None` for no limit.
    pub max_pack_size: Option<f64>,
    /// The npm packages the frames offer through their import map, besides React and Svelte.
    pub libraries: Vec<Library>,
    /// Each registry's URL, ending in `/`, by scope (`@kit`) or `*` for every other scope.
    pub registries: BTreeMap<String, String>,
    /// Who fetches from the registries.
    pub fetch: Fetch,
    /// The origins documents' remote data and images may come from without asking the reader.
    pub origins: Origins,
    /// What the editor offers ([the language server], Q2).
    ///
    /// [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md
    pub editor: Editor,
}

/// A pack the configuration lists.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListedPack {
    /// How far its code is trusted: strict by default.
    pub trust: Trust,
    /// Whether every document gets its names without importing it.
    pub ambient: bool,
    /// The range its installed version must be in.
    pub version: Option<VersionReq>,
}

/// A library the frames offer components: an npm package and the import names it gives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Library {
    /// The package, `d3`.
    pub name: String,
    /// What components import it by, `d3` or `d3/scale`.
    pub imports: Vec<String>,
}

/// Who fetches packs from the registries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Fetch {
    /// The app's own proxy: the default.
    #[default]
    Host,
    /// The reader's browser.
    Direct,
}

/// The origins documents' data and images need no question for (spec §15).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Origins {
    /// These, normalized: `https://host[:port]`. None by default: the reader is asked about each.
    Listed(Vec<String>),
    /// Every origin, for documents the app publishes itself.
    Any,
}

impl Default for Origins {
    fn default() -> Self {
        Self::Listed(Vec::new())
    }
}

/// What the editor offers for the project's documents.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Editor {
    /// The scopes whose packs completion and auto-import offer, besides CompMD's own, `@acme`.
    pub suggest: Vec<String>,
}

/// Whether a pack key is a folder path, `./components`, rather than a pack name.
#[must_use]
pub fn is_folder(key: &str) -> bool {
    key.starts_with("./") || key.starts_with("../")
}

/// Whether `value` is a list of strings.
fn strings(value: &Value) -> Option<Vec<String>> {
    value
        .as_array()?
        .iter()
        .map(|item| item.as_str().map(str::to_owned))
        .collect()
}

/// Whether `range` is a range the processor matches versions against.
fn is_range(range: &Value) -> bool {
    range
        .as_str()
        .is_some_and(|text| VersionReq::parse(text).is_ok())
}

/// The origins `written` names, adding what's wrong with them to `problems`, each after `at`.
fn origins_in(written: &[String], at: &str, problems: &mut Vec<String>) -> Vec<String> {
    written
        .iter()
        .filter_map(|origin| {
            origin_of(origin)
                .map_err(|problem| problems.push(format!("{at}: {problem}")))
                .ok()
        })
        .collect()
}

/// The pack `key`'s options, read, adding what's wrong with them to `problems`.
fn read_pack(key: &str, options: &Value, problems: &mut Vec<String>) -> ListedPack {
    let at = format!("packs[\"{key}\"]");
    if !is_folder(key) && !is_pack_name(key) {
        problems.push(format!(
            "{at}: a pack is a scoped name (`@scope/pack`) or a folder path (`./components`)"
        ));
    }
    let Some(options) = options.as_object() else {
        problems.push(format!("{at}: its options are an object, `{{}}` for none"));
        return ListedPack::default();
    };
    for name in options
        .keys()
        .filter(|name| !PACK_KEYS.contains(&name.as_str()))
    {
        problems.push(format!(
            "{at}.{name}: not an option; a pack takes {}",
            PACK_KEYS.join(", ")
        ));
    }
    let trust = options.get("trust").map_or(Some("strict"), Value::as_str);
    let permissive = trust == Some("permissive");
    if !matches!(trust, Some("strict" | "permissive" | "trusted")) {
        problems.push(format!("{at}.trust: `strict`, `permissive` or `trusted`"));
    }
    if trust == Some("trusted") && !is_folder(key) {
        problems.push(format!(
            "{at}.trust: only a folder in the app may be trusted, never a registry pack; a registry pack that needs more is `permissive`"
        ));
    }
    let network = options.get("network");
    let host = options.get("host");
    if network.is_some() && !permissive {
        problems.push(format!(
            "{at}.network: only a permissive pack reaches the network"
        ));
    }
    if host.is_some() && !permissive {
        problems.push(format!("{at}.host: only a permissive pack is given `host`"));
    }
    if host.is_some_and(|host| !host.is_boolean()) {
        problems.push(format!("{at}.host: true or false"));
    }
    let ambient = options.get("ambient").unwrap_or(&Value::Bool(false));
    if !ambient.is_boolean() {
        problems.push(format!("{at}.ambient: true or false"));
    }
    let version = options.get("version");
    if version.is_some_and(|version| !is_range(version)) {
        problems.push(format!("{at}.version: a semver range, such as `^1.2.0`"));
    }
    let written = network.map(|network| {
        strings(network).unwrap_or_else(|| {
            problems.push(format!("{at}.network: a list of origins"));
            Vec::new()
        })
    });
    let origins = origins_in(
        &written.unwrap_or_default(),
        &format!("{at}.network"),
        problems,
    );
    ListedPack {
        trust: match trust {
            Some("permissive") => Trust::Permissive {
                network: origins,
                host: host == Some(&Value::Bool(true)),
            },
            Some("trusted") => Trust::Trusted,
            _ => Trust::Strict,
        },
        ambient: ambient == &Value::Bool(true),
        version: version
            .and_then(Value::as_str)
            .and_then(|range| VersionReq::parse(range).ok()),
    }
}

/// The packs denied, read, adding what's wrong with them to `problems`.
fn read_deny(deny: &Value, problems: &mut Vec<String>) -> Vec<Deny> {
    let Some(entries) = deny.as_array() else {
        problems.push("deny: a list of `{pack, version?, reason}`".to_owned());
        return Vec::new();
    };
    let mut denied = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        let at = format!("deny[{i}]");
        let pack = entry.get("pack").and_then(Value::as_str);
        let reason = entry.get("reason").and_then(Value::as_str);
        let (Some(pack), Some(reason)) = (pack, reason.filter(|reason| !reason.trim().is_empty()))
        else {
            problems.push(format!(
                "{at}: `{{pack, reason}}`, and `version` for some versions only"
            ));
            continue;
        };
        let name = pack.parse().ok();
        if name.is_none() {
            problems.push(format!("{at}.pack: a scoped name, `@scope/pack`"));
        }
        let version = entry.get("version");
        if version.is_some_and(|version| !is_range(version)) {
            problems.push(format!("{at}.version: a semver range, such as `<2`"));
        }
        if let Some(pack) = name {
            denied.push(Deny {
                pack,
                version: version
                    .and_then(Value::as_str)
                    .and_then(|range| VersionReq::parse(range).ok()),
                reason: reason.to_owned(),
            });
        }
    }
    denied
}

/// The libraries, read, adding what's wrong with them to `problems`.
fn read_libraries(libraries: &Value, problems: &mut Vec<String>) -> Vec<Library> {
    let Some(entries) = libraries.as_array() else {
        problems.push("libraries: a list of npm package names, or `{name, imports}`".to_owned());
        return Vec::new();
    };
    let library_of = |entry: &Value| match entry {
        Value::String(name) => Some(Library {
            name: name.clone(),
            imports: vec![name.clone()],
        }),
        Value::Object(fields) => Some(Library {
            name: fields.get("name")?.as_str()?.to_owned(),
            imports: strings(fields.get("imports")?)?,
        }),
        _ => None,
    };
    let mut read = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        match library_of(entry) {
            Some(library) if is_package_name(&library.name) && !library.imports.is_empty() => read.push(library),
            _ => problems.push(format!(
                "libraries[{i}]: an npm package name, such as `\"d3\"`, or `{{ \"name\": \"d3\", \"imports\": [\"d3\"] }}`"
            )),
        }
    }
    read
}

/// The registries, read, adding what's wrong with them to `problems`.
fn read_registries(registries: &Value, problems: &mut Vec<String>) -> BTreeMap<String, String> {
    let Some(registries) = registries.as_object() else {
        problems.push(
            "registries: an object, by scope (`@kit`), with `*` for every other scope".to_owned(),
        );
        return BTreeMap::new();
    };
    let mut read = BTreeMap::new();
    for (scope, url) in registries {
        let at = format!("registries[\"{scope}\"]");
        if scope != "*" && !is_scope(scope) {
            problems.push(format!(
                "{at}: a scope, such as `@kit`, or `*` for every other scope"
            ));
        }
        match url.as_str().map(registry_url_of) {
            Some(Ok(url)) => {
                read.insert(scope.clone(), url);
            }
            Some(Err(problem)) => problems.push(format!("{at}: {problem}")),
            None => problems.push(format!("{at}: a registry's URL")),
        }
    }
    read
}

/// The origins documents may reach without a question, read, adding what's wrong with them to `problems`.
fn read_origins(origins: &Value, problems: &mut Vec<String>) -> Origins {
    if origins == "any" {
        return Origins::Any;
    }
    let Some(written) = strings(origins) else {
        problems.push(
            "origins: a list of origins, such as `[\"https://cdn.example.com\"]`, or `\"any\"` for every origin".to_owned(),
        );
        return Origins::default();
    };
    Origins::Listed(origins_in(&written, "origins", problems))
}

/// What the editor offers, read, adding what's wrong with it to `problems`.
fn read_editor(editor: &Value, problems: &mut Vec<String>) -> Editor {
    let Some(editor) = editor.as_object() else {
        problems.push("editor: an object, such as `{ \"suggest\": [\"@acme\"] }`".to_owned());
        return Editor::default();
    };
    for name in editor
        .keys()
        .filter(|name| !EDITOR_KEYS.contains(&name.as_str()))
    {
        problems.push(format!(
            "editor.{name}: not a key of `editor`; it takes {}",
            EDITOR_KEYS.join(", ")
        ));
    }
    let Some(suggest) = editor.get("suggest") else {
        return Editor::default();
    };
    let Some(scopes) = strings(suggest) else {
        problems.push("editor.suggest: a list of scopes, such as `[\"@acme\"]`".to_owned());
        return Editor::default();
    };
    for scope in scopes.iter().filter(|scope| !is_scope(scope)) {
        problems.push(format!(
            "editor.suggest: `{scope}` isn't a scope, such as `@acme`"
        ));
    }
    Editor { suggest: scopes }
}

/// The packs, read, adding what's wrong with them to `problems`.
fn read_packs(packs: &Value, problems: &mut Vec<String>) -> Vec<(String, ListedPack)> {
    let Some(packs) = packs.as_object() else {
        problems.push("packs: an object, by pack name or folder path".to_owned());
        return Vec::new();
    };
    packs
        .iter()
        .map(|(key, options)| (key.clone(), read_pack(key, options, problems)))
        .collect()
}

/// `object`'s `key`, read with `read`, or `default` when it has none.
fn read_key<T>(
    object: &Map<String, Value>,
    key: &str,
    default: T,
    problems: &mut Vec<String>,
    read: impl FnOnce(&Value, &mut Vec<String>) -> T,
) -> T {
    object
        .get(key)
        .map_or(default, |value| read(value, problems))
}

/// Reads and checks a configuration, as `@compmd/vite`'s `readConfig` does.
///
/// `value` is the parsed JSON: `compmd.config.json`'s content, or `package.json`'s `compmd` object. `source` says
/// where it came from, for the error.
///
/// # Errors
///
/// [`ConfigError`] with every problem found.
///
/// # Examples
///
/// ```
/// use compmd_project::{read_config, Origins};
///
/// let value = serde_json::json!({ "packs": { "./components": { "trust": "trusted", "ambient": true } }, "origins": "any" });
/// let config = read_config(&value, "compmd.config.json")?;
/// assert_eq!(config.packs[0].0, "./components");
/// assert_eq!(config.origins, Origins::Any);
/// # Ok::<(), compmd_project::ConfigError>(())
/// ```
pub fn read_config(value: &Value, source: &str) -> Result<Config, ConfigError> {
    let error = |problems| ConfigError {
        source: source.to_owned(),
        problems,
    };
    let Some(object) = value.as_object() else {
        return Err(error(vec!["the configuration is an object".to_owned()]));
    };
    let mut problems = Vec::new();
    for key in object.keys().filter(|key| !KEYS.contains(&key.as_str())) {
        problems.push(format!(
            "{key}: not a key of the configuration; it takes {}",
            KEYS.join(", ")
        ));
    }
    let packs = read_key(object, "packs", Vec::new(), &mut problems, read_packs);
    let imports = match object.get("imports").map_or(Some("any"), Value::as_str) {
        Some("listed") => Imports::Listed,
        Some("any") => Imports::Any,
        _ => {
            problems.push("imports: `any` or `listed`".to_owned());
            Imports::Any
        }
    };
    let deny = read_key(object, "deny", Vec::new(), &mut problems, read_deny);
    let max_pack_size = object.get("maxPackSize").map(|size| {
        let bytes = bytes_of(size);
        if bytes.is_none() {
            problems.push("maxPackSize: bytes, or a size such as `\"5 MB\"`".to_owned());
        }
        bytes
    });
    let libraries = read_key(
        object,
        "libraries",
        Vec::new(),
        &mut problems,
        read_libraries,
    );
    let registries = read_key(
        object,
        "registries",
        BTreeMap::new(),
        &mut problems,
        read_registries,
    );
    let fetch = match object.get("fetch").map_or(Some("host"), Value::as_str) {
        Some("direct") => Fetch::Direct,
        Some("host") => Fetch::Host,
        _ => {
            problems.push("fetch: `host` or `direct`".to_owned());
            Fetch::Host
        }
    };
    let origins = read_key(
        object,
        "origins",
        Origins::default(),
        &mut problems,
        read_origins,
    );
    let editor = read_key(
        object,
        "editor",
        Editor::default(),
        &mut problems,
        read_editor,
    );
    if !problems.is_empty() {
        return Err(error(problems));
    }
    Ok(Config {
        packs,
        imports,
        deny,
        max_pack_size: max_pack_size.flatten(),
        libraries,
        registries,
        fetch,
        origins,
        editor,
    })
}

#[cfg(test)]
mod tests;
