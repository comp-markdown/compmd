//! Stage 2, resolving (spec §6, §11): every component name bound to the component it stands for, through local
//! definitions, imports, `@compmd/core` and ambient packs.
//!
//! The processor does no I/O: imported documents and component files come from [`Files`], packs from a
//! [`Registry`]. A host loads what they serve before it resolves, so resolving needs no waiting.

mod language;
mod load;
mod names;
mod origin;
mod source;
mod uses;

pub use language::Language;
pub use names::{Binding, Level, Names};
pub use origin::Origin;

use crate::diagnostic::Diagnostic;
use crate::parse::Parsed;
use crate::settings::Settings;
use crate::tree::{Node, Param, Position, Root, Signature};
use semver::Version;
use std::collections::{BTreeMap, HashMap};
use std::hash::BuildHasher;

/// The files of the project a document is in, by their `/`-separated paths from the project's folder.
pub trait Files {
    /// The text of the file at `path`, or `None` when there is none.
    fn read(&self, path: &str) -> Option<String>;
}

/// Files held in memory, as a browser hands them over.
impl<S: BuildHasher> Files for HashMap<String, String, S> {
    fn read(&self, path: &str) -> Option<String> {
        self.get(path).cloned()
    }
}

/// A release of a pack, as a registry lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    /// Its version.
    pub version: Version,
    /// The integrity hash the registry publishes for it (`sha512-…`), which an import's `integrity` must equal.
    pub integrity: Option<String>,
}

/// The packs a document can import (spec §6.4): npm packages with a `package.json` and a CompMD document.
pub trait Registry {
    /// Every release of `pack`, in any order.
    fn releases(&self, pack: &str) -> Vec<Release>;

    /// The file at `path` in a release of `pack`: `package.json`, the pack's document, or a file it imports.
    fn read(&self, pack: &str, version: &Version, path: &str) -> Option<String>;
}

/// A registry with no packs.
impl Registry for () {
    fn releases(&self, _pack: &str) -> Vec<Release> {
        Vec::new()
    }

    fn read(&self, _pack: &str, _version: &Version, _path: &str) -> Option<String> {
        None
    }
}

/// Where a document's imports come from.
pub struct Sources<'a> {
    /// The project's files.
    pub files: &'a dyn Files,
    /// The packs.
    pub registry: &'a dyn Registry,
    /// Packs of the registry that every document imports, at the lowest priority (spec §6.3). Their newest release
    /// is used.
    pub ambient: Vec<String>,
    /// Whether to report each use of a name from an ambient pack (W113).
    pub report_ambient: bool,
}

/// A document, by its place in [`Resolved::documents`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentId(usize);

/// A component, by its place in [`Resolved::components`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentId(usize);

/// A loaded document: the processed one, or one it imports, directly or not.
#[derive(Clone, Debug)]
pub struct Document {
    /// Where it comes from.
    pub origin: Origin,
    /// Its text.
    pub text: String,
    /// Its tree.
    pub root: Root,
    /// What each component name in it stands for.
    pub names: Names,
    /// What it exports, by public name.
    pub exports: BTreeMap<String, Binding>,
    /// Whether the processor provides it (`@compmd/core`, an ambient pack), so that a signature fence alone in it
    /// is a component the processor implements.
    pub provided: bool,
    pub(crate) settings: Settings,
}

/// What implements a component.
#[derive(Clone, Debug, PartialEq)]
pub enum Body {
    /// A `compmd` definition's body.
    Compmd(Vec<Node>),
    /// Host-language code in a definition fence.
    Host {
        /// The language as written, the fence's first word.
        language: String,
        /// The fence's `runtime=`.
        runtime: Option<String>,
        /// The code.
        source: String,
    },
    /// A component file a document imports, `./chart.jsx`.
    File {
        /// Where the file is.
        origin: Origin,
        /// Its language, from its extension.
        language: Language,
    },
    /// The processor: a signature fence alone in a pack the processor provides (spec §9).
    Processor,
}

impl Body {
    /// For a host-language component, whether one of this processor's bindings runs it (W102 otherwise).
    #[must_use]
    pub fn supported(&self) -> bool {
        match self {
            Self::Host {
                language, runtime, ..
            } => Language::from_name(language)
                .is_some_and(|binding| binding.runs(runtime.as_deref())),
            _ => true,
        }
    }

    /// Whether it is host-language code, in a fence or a file.
    #[must_use]
    pub fn is_host(&self) -> bool {
        matches!(self, Self::Host { .. } | Self::File { .. })
    }
}

/// A component a name can stand for.
#[derive(Clone, Debug, PartialEq)]
pub struct Component {
    /// Its name where it is defined, or the local name of the file import.
    pub name: String,
    /// The document that defines it, or imports its file.
    pub document: DocumentId,
    /// Its signature: a `compmd` definition's header, or the signature fence; `None` for a host-language component
    /// without one, whose params go through unchecked (W105).
    pub signature: Option<Signature>,
    /// What implements it.
    pub body: Body,
    /// The definition fence's param list, which styles the component's box.
    pub fence: Vec<Param>,
    /// Where it is defined.
    pub position: Option<Position>,
}

/// The result of resolving a document: it, everything it imports, and the components their names stand for.
#[derive(Clone, Debug)]
pub struct Resolved {
    /// The processed document, its imports, `@compmd/core` and the ambient packs.
    pub documents: Vec<Document>,
    /// Every component of those documents.
    pub components: Vec<Component>,
    /// The processed document.
    pub main: DocumentId,
    /// The diagnostics of stages 1 and 2, of the processed document and of what it imports.
    pub diagnostics: Vec<Diagnostic>,
}

impl Resolved {
    /// The document `id` names.
    #[must_use]
    pub fn document(&self, id: DocumentId) -> &Document {
        &self.documents[id.0]
    }

    /// The component `id` names.
    #[must_use]
    pub fn component(&self, id: ComponentId) -> &Component {
        &self.components[id.0]
    }

    /// Every document, in order.
    pub fn ids(&self) -> impl Iterator<Item = DocumentId> + use<> {
        (0..self.documents.len()).map(DocumentId)
    }

    /// How diagnostics name the document `id`: `None` for the processed one.
    #[must_use]
    pub fn file(&self, id: DocumentId) -> Option<String> {
        (id != self.main).then(|| self.document(id).origin.to_string())
    }
}

/// Resolves a parsed document, stage 2 of spec §11: loads its imports and their imports, `@compmd/core` and the
/// ambient packs, binds every name, and reports what can't be bound.
///
/// `path` is where the document is in the project, against which its relative imports resolve; `parsed` is what
/// [`parse`](crate::parse) gave for `text`, whose diagnostics come first in the result.
///
/// ```
/// use compmd::resolve::{Binding, Sources, resolve};
/// use compmd::{ParseOptions, parse};
/// use std::collections::HashMap;
///
/// let text = "::: stack {gap=s}\nText.\n::: /stack\n";
/// let files: HashMap<String, String> = HashMap::new();
/// let sources = Sources { files: &files, registry: &(), ambient: Vec::new(), report_ambient: false };
/// let resolved = resolve("notes.comp.md", text, parse(text, &ParseOptions::default()), &sources);
/// assert!(resolved.diagnostics.is_empty());
/// let names = &resolved.document(resolved.main).names;
/// assert!(matches!(names.get("stack"), Some(Binding::Component(..))));
/// ```
#[must_use]
pub fn resolve(path: &str, text: &str, parsed: Parsed, sources: &Sources) -> Resolved {
    load::Loader::new(sources, Origin::File(path.to_string())).resolve(text, parsed)
}
