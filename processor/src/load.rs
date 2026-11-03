//! Stage 4a, loading (spec §8.6, §8.7, §11): the values of the processed document's inputs, and the data imports of
//! every loaded document, each typed by its schema.
//!
//! The processor does no I/O: data comes from the host's [`Files`](crate::host::Files) and
//! [`Packs`](crate::host::Packs), a [`Fetch`] for `https:` URLs and [`Data`] for `host:` sources. Since a `from` may
//! interpolate inputs, which sources a render needs depends on the inputs: the validated
//! [`Processor`](crate::Processor) lists them, so a host can fetch them before it loads.

mod format;
mod from;
mod inputs;
mod schema;

use crate::cel::{Key, Value};
use crate::context::RenderContext;
use crate::diagnostic::{Code, Diagnostic, Location, Stage};
use crate::expanded::Inputs;
use crate::host::{CacheKey, Data, Fetch, Needed};
use crate::resolve::{DocumentId, Origin, Resolved, Store};
use crate::settings::DataImport;
use crate::validate::Validated;
use crate::validate::data::origin;
use format::{Format, Parsed};
use from::Target;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::rc::Rc;

/// What loading gave: the inputs' values, every document's data, and the diagnostics of stage 4a.
#[derive(Clone, Debug, Default)]
pub(crate) struct Loaded {
    /// The processed document's inputs, with the values the render uses.
    pub(crate) inputs: Inputs,
    /// The diagnostics of stage 4a, until the processor takes them.
    pub(crate) diagnostics: Vec<Diagnostic>,
    data: HashMap<DocumentId, Value>,
}

impl Loaded {
    /// `md.data` of the document `id`: each data import by its name, `null` when it failed.
    pub(crate) fn data(&self, id: DocumentId) -> Value {
        self.data
            .get(&id)
            .cloned()
            .unwrap_or_else(|| Value::Map(Rc::default()))
    }
}

/// The remote and `host:` sources a render with `context` needs, each once, in the order they are first written; the
/// remote URLs of origins `allowed_origins` leaves out; and the packs resolving found no release for.
pub(crate) fn sources(
    resolved: &Resolved,
    validated: &Validated,
    context: &RenderContext,
    allowed_origins: &[String],
) -> Needed {
    let (inputs, _) = inputs::of(resolved, context);
    let input = inputs.value();
    let mut needed = Needed {
        packs: resolved.missing.clone(),
        ..Needed::default()
    };
    for (id, index, import) in all_imports(resolved) {
        let refused = validated.refuses_data(id, index);
        let (list, source) = match from::target(resolved, id, import, &input) {
            Ok(Target::Remote(url)) if !refused => (&mut needed.remote, url),
            Ok(Target::Remote(url))
                if origin(&url).is_some_and(|origin| {
                    !allowed_origins.iter().any(|allowed| allowed == origin)
                }) =>
            {
                (&mut needed.disallowed, url)
            }
            Ok(Target::Host(source)) if !refused => (&mut needed.host, source),
            Ok(_) | Err(_) => continue,
        };
        if !list.contains(&source) {
            list.push(source);
        }
    }
    needed
}

/// The cache key of each of the processed document's data imports, for the values `inputs` hold, in written order;
/// an import validating refused, or whose `from` gives no source, has none.
pub(crate) fn cache_keys(
    resolved: &Resolved,
    validated: &Validated,
    inputs: &Inputs,
) -> Vec<CacheKey> {
    let input = inputs.value();
    imports(resolved, validated)
        .filter(|(id, _, _)| *id == resolved.main)
        .filter_map(|(id, _, import)| {
            let key = match from::target(resolved, id, import, &input).ok()? {
                Target::File(origin) => origin.to_string(),
                Target::Remote(url) => url,
                Target::Host(source) => format!("host:{source}"),
            };
            Some(CacheKey {
                name: import.name.clone(),
                key,
            })
        })
        .collect()
}

/// The host's sources of data: its files and packs, remote data and `host:` data.
pub(crate) struct Host<'a> {
    pub(crate) store: Store<'a>,
    pub(crate) fetch: &'a dyn Fetch,
    pub(crate) data: &'a dyn Data,
    /// Whether to report each `host:` source (W113), as the host asked for its ambient names (`Ambient::report`).
    pub(crate) report: bool,
}

/// Loads, stage 4a of spec §11: the inputs' values for `context`, and every data import that validating didn't
/// refuse, from `host`; a refused or failed import is its `fallback`, or `null` (spec §10.3).
pub(crate) fn load(
    resolved: &Resolved,
    validated: &Validated,
    context: &RenderContext,
    host: &Host,
) -> Loaded {
    let (inputs, diagnostics) = inputs::of(resolved, context);
    let mut loader = Loader {
        resolved,
        host,
        loaded: Loaded {
            inputs,
            diagnostics,
            data: HashMap::new(),
        },
    };
    let input = loader.loaded.inputs.value();
    let mut data: HashMap<DocumentId, IndexMap<Key, Value>> = HashMap::new();
    for (id, index, import) in all_imports(resolved) {
        let value = if validated.refuses_data(id, index) {
            loader.fallback(id, import).unwrap_or(Value::Null)
        } else {
            let target = from::target(resolved, id, import, &input);
            if let Ok(Target::Host(source)) = &target
                && host.report
            {
                loader.not_portable(id, import, source);
            }
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
struct Loader<'r, 'h> {
    resolved: &'r Resolved,
    host: &'h Host<'h>,
    loaded: Loaded,
}

impl Loader<'_, '_> {
    /// One data import: its source, else its fallback, typed by its schema. F026 when neither loads, and for a
    /// local file that doesn't, since `fallback` is for remote and `host:` sources (spec §8.6): its fallback is then
    /// only the recovery (this processor's choice, `processor/README.md`).
    fn import(
        &mut self,
        id: DocumentId,
        import: &DataImport,
        target: Result<Target, String>,
    ) -> Value {
        let local = matches!(target, Ok(Target::File(_)));
        let loaded = target
            .map_err(Failure::new)
            .and_then(|target| self.target(import, &target));
        let failure = match loaded {
            Ok(parsed) => return self.typed(id, import, parsed),
            Err(failure) => failure,
        };
        let fallback = self.fallback(id, import);
        match &fallback {
            None => self.failed(id, import, failure, "is null"),
            Some(_) if local => self.failed(id, import, failure, "is its fallback"),
            Some(_) => {}
        }
        fallback.unwrap_or(Value::Null)
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
                    .host
                    .fetch
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
                let value = self.host.data.data(source).ok_or_else(|| {
                    Failure::new(format!("this processor doesn't provide `host:{source}`"))
                })?;
                Ok(Parsed::value(value))
            }
        }
    }

    /// A data file of the root folder or of a pack, by its extension.
    fn file(&self, origin: &Origin, import: &DataImport) -> Result<Parsed, Failure> {
        let name = origin.to_string();
        let text = self
            .host
            .store
            .read(origin)
            .ok_or_else(|| Failure::new(format!("`{name}` doesn't exist")))?;
        let format = Format::of_path(origin.path(), import.delimiter)
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

    /// W113 at a `host:` source's `from` (spec §8.6): its data is this processor's, so the document isn't portable.
    fn not_portable(&mut self, id: DocumentId, import: &DataImport, source: &str) {
        let diagnostic = Diagnostic::new(
            Code::W113,
            import.from.at,
            format!(
                "`host:{source}` is this processor's own data, so `md.data.{}` is empty where another processor renders the document",
                import.name
            ),
        );
        self.loaded.diagnostics.push(
            diagnostic
                .in_file(self.resolved.file(id))
                .in_stage(Stage::Load),
        );
    }

    /// F026: at the place inside the data file when known, else at the import's `from`, saying what `md.data.<name>`
    /// is instead (`recovered`).
    fn failed(&mut self, id: DocumentId, import: &DataImport, failure: Failure, recovered: &str) {
        let message = format!("{}; `md.data.{}` {recovered}", failure.message, import.name);
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
