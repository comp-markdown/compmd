//! Stage 4a, loading (spec §8.6, §8.7, §11): the values of the processed document's inputs, and the data imports of
//! every loaded document, each typed by its schema.
//!
//! The processor does no I/O: data comes from the project's [`Files`], the packs' [`Registry`], a [`Remote`] for
//! `https:` URLs and [`HostData`] for `host:` sources. Since a `from` may interpolate inputs, which sources a render
//! needs depends on the inputs: [`sources`] lists them, so a host can fetch them before it calls [`load`].

mod format;
mod from;
mod inputs;
mod schema;

use crate::cel::{Key, Value};
use crate::context::Context;
use crate::diagnostic::{Code, Diagnostic, Location, Stage};
use crate::expanded::Inputs;
use crate::resolve::{DocumentId, Files, Origin, Registry, Resolved};
use crate::settings::DataImport;
use crate::validate::Validated;
use format::{Format, Parsed};
use from::Target;
use indexmap::IndexMap;
use serde_json::Value as Json;
use std::collections::HashMap;
use std::rc::Rc;

/// What a remote source gave.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fetched {
    /// The body, as text.
    pub text: String,
    /// The `Content-Type` header, which decides the format; without one, the URL's extension does.
    pub content_type: Option<String>,
}

/// Remote data (the `remote data` feature, spec §8.6): what the host fetched for a URL.
pub trait Remote {
    /// What `url` gave, or `None` when it couldn't be fetched. Only `https:` URLs of origins the host allows reach
    /// this; the host fetches without credentials, and decides how long a copy is kept (`cache`).
    fn fetch(&self, url: &str) -> Option<Fetched>;
}

/// No remote data: every URL fails, so data imports from URLs load their fallback.
impl Remote for () {
    fn fetch(&self, _url: &str) -> Option<Fetched> {
        None
    }
}

/// Data a host provides (spec §8.6): what follows `host:` in a `from`, such as `tasks?status=open`, names it.
pub trait HostData {
    /// The data `source` names, as JSON, or `None` when the host doesn't provide it.
    fn read(&self, source: &str) -> Option<Json>;
}

/// No host data: every `host:` source is missing.
impl HostData for () {
    fn read(&self, _source: &str) -> Option<Json> {
        None
    }
}

/// Where data comes from.
pub struct DataSources<'a> {
    /// The project's files.
    pub files: &'a dyn Files,
    /// The packs, for data files inside them.
    pub registry: &'a dyn Registry,
    /// Remote data.
    pub remote: &'a dyn Remote,
    /// The host's data.
    pub host: &'a dyn HostData,
}

/// A source a render needs that the host provides.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    /// An `https:` URL, its values filled in.
    Remote(String),
    /// A `host:` source, without `host:`.
    Host(String),
}

/// What loading gave: the inputs' values, every document's data, and the diagnostics of stage 4a.
#[derive(Clone, Debug, Default)]
pub struct Loaded {
    /// The processed document's inputs, with the values the render uses.
    pub inputs: Inputs,
    /// The diagnostics of stage 4a.
    pub diagnostics: Vec<Diagnostic>,
    data: HashMap<DocumentId, Value>,
}

impl Loaded {
    /// `md.data` of the document `id`: each data import by its name, `null` when it failed.
    #[must_use]
    pub fn data(&self, id: DocumentId) -> Value {
        self.data
            .get(&id)
            .cloned()
            .unwrap_or_else(|| Value::Map(Rc::default()))
    }

    /// `md.input`: each input's value by its name.
    #[must_use]
    pub fn input(&self) -> Value {
        Value::Map(Rc::new(
            self.inputs
                .0
                .iter()
                .map(|input| {
                    (
                        Key::String(Rc::from(input.name.as_str())),
                        Value::from_json(&input.value),
                    )
                })
                .collect(),
        ))
    }
}

/// The remote and `host:` sources a render with `context` needs, in the order they are written.
#[must_use]
pub fn sources(resolved: &Resolved, validated: &Validated, context: &Context) -> Vec<Source> {
    let (inputs, _) = inputs::of(resolved, context);
    let input = Loaded {
        inputs,
        ..Loaded::default()
    }
    .input();
    imports(resolved, validated)
        .filter_map(
            |(id, _, import)| match from::target(resolved, id, import, &input).ok()? {
                Target::Remote(url) => Some(Source::Remote(url)),
                Target::Host(source) => Some(Source::Host(source)),
                Target::File(_) => None,
            },
        )
        .collect()
}

/// Loads, stage 4a of spec §11: the inputs' values for `context`, and every data import that validating didn't
/// refuse; a refused or failed import is its `fallback`, or `null` (spec §10.3).
#[must_use]
pub fn load(
    resolved: &Resolved,
    validated: &Validated,
    context: &Context,
    sources: &DataSources,
) -> Loaded {
    let (inputs, diagnostics) = inputs::of(resolved, context);
    let mut loader = Loader {
        resolved,
        sources,
        loaded: Loaded {
            inputs,
            diagnostics,
            data: HashMap::new(),
        },
    };
    let input = loader.loaded.input();
    let mut data: HashMap<DocumentId, IndexMap<Key, Value>> = HashMap::new();
    for (id, index, import) in all_imports(resolved) {
        let value = if validated.refuses_data(id, index) {
            loader.fallback(id, import).unwrap_or(Value::Null)
        } else {
            let target = from::target(resolved, id, import, &input);
            loader.import(id, import, target)
        };
        data.entry(id)
            .or_default()
            .insert(Key::String(Rc::from(import.name.as_str())), value);
    }
    loader.loaded.data = data
        .into_iter()
        .map(|(id, fields)| (id, Value::Map(Rc::new(fields))))
        .collect();
    loader.loaded
}

/// Every data import validating didn't refuse: its document, its place, and it.
fn imports<'r>(
    resolved: &'r Resolved,
    validated: &'r Validated,
) -> impl Iterator<Item = (DocumentId, usize, &'r DataImport)> {
    all_imports(resolved).filter(|(id, index, _)| !validated.refuses_data(*id, *index))
}

/// Every data import of every loaded document: its document, its place in `md.data`, and it.
fn all_imports(resolved: &Resolved) -> impl Iterator<Item = (DocumentId, usize, &DataImport)> {
    resolved.ids().flat_map(move |id| {
        resolved
            .document(id)
            .settings
            .data
            .iter()
            .enumerate()
            .map(move |(index, import)| (id, index, import))
    })
}

/// A record dropped (F027): why, and the file and place it is at, when that is known.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Violation {
    message: String,
    at: Option<(String, Location)>,
}

/// Why a source couldn't be loaded, and where inside it, when that is known.
#[derive(Debug)]
struct Failure {
    message: String,
    /// The file and the place in it.
    inside: Option<(String, Location)>,
}

impl Failure {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            inside: None,
        }
    }
}

/// Loads the data of one render.
struct Loader<'r, 's> {
    resolved: &'r Resolved,
    sources: &'s DataSources<'s>,
    loaded: Loaded,
}

impl Loader<'_, '_> {
    /// One data import: its source, else its fallback, typed by its schema. F026 when neither loads.
    fn import(
        &mut self,
        id: DocumentId,
        import: &DataImport,
        target: Result<Target, String>,
    ) -> Value {
        let loaded = target
            .map_err(Failure::new)
            .and_then(|target| self.target(import, &target));
        match loaded {
            Ok(parsed) => self.typed(id, import, parsed),
            Err(failure) => self.fallback(id, import).unwrap_or_else(|| {
                self.failed(id, import, failure);
                Value::Null
            }),
        }
    }

    /// The fallback of an import, typed; `None` without one, or when it fails too.
    fn fallback(&mut self, id: DocumentId, import: &DataImport) -> Option<Value> {
        let parsed = self.fallback_parsed(id, import)?;
        Some(self.typed(id, import, parsed))
    }

    fn fallback_parsed(&self, id: DocumentId, import: &DataImport) -> Option<Parsed> {
        let fallback = import.fallback.as_deref()?;
        let origin = self.resolved.document(id).origin.join(fallback)?;
        self.file(&origin, import).ok()
    }

    /// A source's data.
    fn target(&self, import: &DataImport, target: &Target) -> Result<Parsed, Failure> {
        match target {
            Target::File(origin) => self.file(origin, import),
            Target::Remote(url) => {
                let fetched = self
                    .sources
                    .remote
                    .fetch(url)
                    .ok_or_else(|| Failure::new(format!("`{url}` couldn't be fetched")))?;
                let format = fetched
                    .content_type
                    .as_deref()
                    .and_then(Format::of_content_type)
                    .or_else(|| Format::of_path(from::path_of(url), import.delimiter))
                    .ok_or_else(|| Failure::new(format!("the format of `{url}` is unknown")))?;
                format.parse(url, &fetched.text)
            }
            Target::Host(source) => {
                let value = self.sources.host.read(source).ok_or_else(|| {
                    Failure::new(format!("this processor doesn't provide `host:{source}`"))
                })?;
                Ok(Parsed::value(value))
            }
        }
    }

    /// A data file of the project or of a pack, by its extension.
    fn file(&self, origin: &Origin, import: &DataImport) -> Result<Parsed, Failure> {
        let (text, path) = match origin {
            Origin::File(path) => (self.sources.files.read(path), path),
            Origin::Pack {
                name,
                version,
                path,
            } => (self.sources.registry.read(name, version, path), path),
            Origin::Core => (None, &String::new()),
        };
        let name = origin.to_string();
        let text = text.ok_or_else(|| Failure::new(format!("`{name}` doesn't exist")))?;
        let format = Format::of_path(path, import.delimiter)
            .ok_or_else(|| Failure::new(format!("the format of `{name}` is unknown")))?;
        format.parse(&name, &text)
    }

    /// The data typed by the import's schema; each record that violates it, or that couldn't be read, is dropped,
    /// with F027.
    fn typed(&mut self, id: DocumentId, import: &DataImport, parsed: Parsed) -> Value {
        let (value, violations) = match &import.schema {
            Some(fields) => schema::apply(parsed, fields),
            None => (parsed.value, parsed.dropped),
        };
        for violation in violations {
            let diagnostic = match violation.at {
                Some((file, at)) => {
                    Diagnostic::new(Code::F027, at, violation.message).in_file(Some(file))
                }
                None => Diagnostic::new(Code::F027, import.from.at, violation.message)
                    .in_file(self.resolved.file(id)),
            };
            self.loaded
                .diagnostics
                .push(diagnostic.in_stage(Stage::Load));
        }
        Value::from_json(&value)
    }

    /// F026: at the place inside the data file when known, else at the import's `from`.
    fn failed(&mut self, id: DocumentId, import: &DataImport, failure: Failure) {
        let message = format!("{}; `md.data.{}` is null", failure.message, import.name);
        let diagnostic = match failure.inside {
            Some((file, at)) => Diagnostic::new(Code::F026, at, message).in_file(Some(file)),
            None => {
                Diagnostic::new(Code::F026, import.from.at, message).in_file(self.resolved.file(id))
            }
        };
        self.loaded
            .diagnostics
            .push(diagnostic.in_stage(Stage::Load));
    }
}
