//! Stage 2, resolving (spec §6, §11): every component name bound to the component it stands for, through local
//! definitions, imports, `@compmd/core` and ambient packs.
//!
//! The processor does no I/O: imported documents and component files come from the host's [`Files`], packs from its
//! [`Packs`]. A host loads what they serve before it resolves, so resolving needs no waiting.

mod language;
mod load;
mod names;
mod origin;
mod source;
mod uses;

pub use language::Language;
pub(crate) use names::{Binding, Names};
pub(crate) use origin::Origin;
use origin::RESERVED_SCOPE;
pub(crate) use uses::invocations;

use crate::diagnostic::Diagnostic;
use crate::host::{Files, Packs, Release};
use crate::lines::{LineStarts, Lines};
use crate::parse::{Parsed, Spliced};
use crate::reserved::Keyword;
use crate::settings::Settings;
use crate::tree::{Move, Node, Param, Point, Position, Root, Shift, Signature};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;

/// The document of `@compmd/core`, built in (spec §9).
pub(crate) const CORE_TEXT: &str = include_str!("core.comp.md");

/// The packs every document imports, at the lowest priority (spec §6.3), and whether to report their names.
///
/// ```
/// use compmd::Ambient;
///
/// let ambient = Ambient { packs: vec!["@acme/kit".into()], report: true };
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ambient {
    /// The packs, by name. Their newest release is used.
    pub packs: Vec<String>,
    /// Whether to report each use of a name from them (W113).
    pub report: bool,
}

/// Where documents and the files they name come from: the host's files and packs, and `@compmd/core`, which the
/// processor serves itself: the host is never asked for a pack of `@compmd/` (spec §6.4).
#[derive(Clone, Copy)]
pub(crate) struct Store<'h> {
    pub(crate) files: &'h dyn Files,
    pub(crate) packs: &'h dyn Packs,
}

impl Store<'_> {
    /// The text of the file at `origin`, or `None` when there is none.
    pub(crate) fn read(&self, origin: &Origin) -> Option<String> {
        match origin {
            Origin::File(path) => self.files.file(path),
            _ if origin.is_core() => Some(CORE_TEXT.to_string()),
            Origin::Pack { name, .. } if name.starts_with(RESERVED_SCOPE) => None,
            Origin::Pack {
                name,
                version,
                path,
            } => self.packs.pack_file(name, version, path),
        }
    }

    /// Every release of the host's `pack`; none of `@compmd/`.
    pub(crate) fn releases(&self, pack: &str) -> Vec<Release> {
        if pack.starts_with(RESERVED_SCOPE) {
            Vec::new()
        } else {
            self.packs.releases(pack)
        }
    }
}

/// A document, by its place in [`Resolved::documents`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct DocumentId(usize);

/// A component, by its place in [`Resolved::components`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ComponentId(usize);

/// A loaded document: the processed one, or one it imports, directly or not.
#[derive(Clone, Debug)]
pub(crate) struct Document {
    /// Where it comes from.
    pub(crate) origin: Origin,
    /// Its text.
    pub(crate) text: String,
    /// Where the lines of its text start.
    pub(crate) line_starts: LineStarts,
    /// Its tree.
    pub(crate) root: Rc<Root>,
    /// What each component name in it stands for.
    pub(crate) names: Names,
    /// What it exports, by public name.
    pub(crate) exports: BTreeMap<String, Binding>,
    /// Whether the processor provides it (`@compmd/core`, an ambient pack), so that a signature fence alone in it
    /// is a component the processor implements.
    pub(crate) provided: bool,
    pub(crate) settings: Settings,
}

impl Document {
    /// The lines of its text.
    pub(crate) fn lines(&self) -> Lines<'_> {
        Lines::new(&self.text, &self.line_starts)
    }
}

/// What implements a component.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Body {
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
    pub(crate) fn supported(&self) -> bool {
        match self {
            Self::Host {
                language, runtime, ..
            } => Language::from_name(language)
                .is_some_and(|binding| binding.runs(runtime.as_deref())),
            _ => true,
        }
    }

    /// Whether it is host-language code, in a fence or a file.
    pub(crate) fn is_host(&self) -> bool {
        matches!(self, Self::Host { .. } | Self::File { .. })
    }
}

/// A component a name can stand for.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Component {
    /// Its name where it is defined, or the local name of the file import.
    pub(crate) name: String,
    /// The document that defines it, or imports its file.
    pub(crate) document: DocumentId,
    /// Its signature: a `compmd` definition's header, or the signature fence; `None` for a host-language component
    /// without one, whose params go through unchecked (W105).
    pub(crate) signature: Option<Signature>,
    /// What implements it.
    pub(crate) body: Body,
    /// The definition fence's param list, which styles the component's box.
    pub(crate) fence: Vec<Param>,
    /// Where it is defined.
    pub(crate) position: Option<Position>,
}

/// A host-language component, with what a renderer needs to run it (spec §7.4): the name the expanded tree gives it,
/// its language and runtime, and its code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostComponent {
    /// The name a document gives it, as its nodes in the expanded tree have it.
    pub name: String,
    /// The language: the definition fence's first word, or the component file's extension.
    pub language: Language,
    /// The fence's `runtime=`; `None` for the binding's default.
    pub runtime: Option<String>,
    /// The code: the fence's content, or the component file's text.
    pub code: String,
}

impl Component {
    /// It as a host-language component named `name`, when one of the processor's bindings runs it and its code is
    /// there: a fence's, or its file's from `store`.
    fn host_code(&self, name: &str, store: Store<'_>) -> Option<HostComponent> {
        let (language, runtime, code) = match &self.body {
            Body::Host {
                language,
                runtime,
                source,
            } => {
                let language = Language::from_name(language)?;
                language.runs(runtime.as_deref()).then_some(())?;
                (language, runtime.clone(), source.clone())
            }
            Body::File { origin, language } => (*language, None, store.read(origin)?),
            Body::Compmd(_) | Body::Processor => return None,
        };
        Some(HostComponent {
            name: name.to_string(),
            language,
            runtime,
            code,
        })
    }

    /// The built-in keyword it is, if any. Resolving admits a keyword as a name only in `@compmd/core` (F013), so a
    /// component with a keyword's name is always core's.
    pub(crate) fn keyword(&self) -> Option<Keyword> {
        Keyword::from_name(&self.name)
    }

    /// Whether it has no box: a `compmd` component whose fence has `display=contents` with no breakpoint (spec §9.2).
    pub(crate) fn is_contents(&self) -> bool {
        matches!(self.body, Body::Compmd(_))
            && self
                .fence
                .iter()
                .any(|param| param.is_display_contents() && param.breakpoint.is_none())
    }
}

/// The result of resolving a document: it, everything it imports, and the components their names stand for.
#[derive(Clone, Debug)]
pub(crate) struct Resolved {
    /// The processed document, its imports, `@compmd/core` and the ambient packs.
    pub(crate) documents: Vec<Document>,
    /// Every component of those documents.
    pub(crate) components: Vec<Component>,
    /// The processed document.
    pub(crate) main: DocumentId,
    /// The invocations that close a cycle of definitions (F019), by their document and where they start.
    pub(crate) cycles: HashSet<(DocumentId, Point)>,
    /// The diagnostics of stage 2, and of stage 1 for the documents it imports.
    diagnostics: Found,
}

/// The diagnostics of resolving, in three parts: loading's, then those of the names the documents before the
/// processed one use; those of the names each top-level block of the processed document uses; those of the
/// documents after it, then the cycles of definitions (F019).
#[derive(Clone, Debug, Default)]
struct Found {
    before: Vec<Diagnostic>,
    blocks: Vec<Vec<Diagnostic>>,
    after: Vec<Diagnostic>,
}

impl Resolved {
    /// The document `id` names.
    pub(crate) fn document(&self, id: DocumentId) -> &Document {
        &self.documents[id.0]
    }

    /// The component `id` names.
    pub(crate) fn component(&self, id: ComponentId) -> &Component {
        &self.components[id.0]
    }

    /// Every document, in order.
    pub(crate) fn ids(&self) -> impl Iterator<Item = DocumentId> + use<> {
        (0..self.documents.len()).map(DocumentId)
    }

    /// How diagnostics name the document `id`: `None` for the processed one.
    pub(crate) fn file(&self, id: DocumentId) -> Option<String> {
        (id != self.main).then(|| self.document(id).origin.to_string())
    }

    /// Whether the invocation at `position` in the document `id` closes a cycle of definitions (F019), so that it
    /// renders its fallback.
    pub(crate) fn closes_cycle(&self, id: DocumentId, position: Option<Position>) -> bool {
        position.is_some_and(|position| self.cycles.contains(&(id, position.start)))
    }

    /// The processed document's text and line starts, taken for an edit. Its tree is let go too, so that the parse,
    /// which holds the only other reference to it, changes it in place.
    pub(crate) fn take_text(&mut self) -> (String, LineStarts) {
        let main = &mut self.documents[self.main.0];
        main.root = Rc::default();
        let lines = std::mem::replace(&mut main.line_starts, LineStarts::new(""));
        (std::mem::take(&mut main.text), lines)
    }

    /// Every host-language component one of this processor's bindings runs, by each name a document gives it, with
    /// its code from `store`; by name. A name two documents give different components is the processed document's,
    /// or else the first document's in load order.
    pub(crate) fn host_components(&self, store: Store<'_>) -> Vec<HostComponent> {
        let mut found: BTreeMap<&str, HostComponent> = BTreeMap::new();
        let main_first = std::iter::once(self.main).chain(self.ids().filter(|id| *id != self.main));
        for id in main_first {
            for (name, component) in self.document(id).names.components() {
                if found.contains_key(name) {
                    continue;
                }
                if let Some(host) = self.component(component).host_code(name, store) {
                    found.insert(name, host);
                }
            }
        }
        found.into_values().collect()
    }

    /// The diagnostics of stage 2, and of stage 1 for the documents it imports, in order.
    pub(crate) fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> {
        let found = &self.diagnostics;
        found
            .before
            .iter()
            .chain(found.blocks.iter().flatten())
            .chain(&found.after)
    }

    /// The processed document after an edit that changed its top-level blocks `spliced.blocks` and gave it `text`,
    /// with the line starts `lines`, and the tree `root`: the names those blocks use are found again, and everything
    /// after them moves. The edit changed no definition or signature fence, so no name stands for anything else.
    pub(crate) fn splice(
        &mut self,
        (text, lines): (String, LineStarts),
        root: Rc<Root>,
        spliced: &Spliced,
        report_ambient: bool,
    ) {
        let main = self.main;
        let moved = spliced.moved;
        let document = &mut self.documents[main.0];
        document.text = text;
        document.line_starts = lines;
        document.root = root;
        for component in &mut self.components {
            if component.document == main
                && component
                    .position
                    .is_some_and(|position| moved.moves(position.start))
            {
                component.shift(moved.by);
            }
        }
        self.cycles = self
            .cycles
            .drain()
            .map(|(id, mut start)| {
                if id == main {
                    moved.point(&mut start);
                }
                (id, start)
            })
            .collect();
        let start = spliced.blocks.start;
        let blocks = &self.document(main).root.children[start..start + spliced.inserted];
        let found: Vec<Vec<Diagnostic>> = blocks
            .iter()
            .map(|block| uses::block(self, block, report_ambient))
            .collect();
        let diagnostics = &mut self.diagnostics;
        let kept = diagnostics.before.iter_mut().chain(&mut diagnostics.after);
        for diagnostic in kept.chain(diagnostics.blocks.iter_mut().flatten()) {
            moved.diagnostic(diagnostic);
        }
        diagnostics.blocks.splice(spliced.blocks.clone(), found);
    }
}

impl Shift for Component {
    fn shift(&mut self, by: Move) {
        self.signature.shift(by);
        if let Body::Compmd(body) = &mut self.body {
            body.shift(by);
        }
        self.fence.shift(by);
        self.position.shift(by);
    }
}

/// The parses of the documents resolving loaded, by where they come from, with the text each was parsed from: a
/// later resolve parses a document again only when its text changed.
#[derive(Clone, Debug, Default)]
pub(crate) struct Parses(HashMap<Origin, (String, Parsed)>);

/// Resolves a parsed document, stage 2 of spec §11: loads its imports and their imports, `@compmd/core` and the
/// ambient packs from `store`, binds every name, and reports what can't be bound.
///
/// `path` is where the document is under its root folder, against which its relative imports resolve; `parsed` is
/// its tree and settings, whose parse diagnostics the caller holds. The documents it loads are parsed once and kept in
/// `known`.
pub(crate) fn resolve(
    path: &str,
    text: String,
    parsed: Parsed,
    store: Store,
    ambient: &Ambient,
    known: &mut Parses,
) -> Resolved {
    load::Loader::new(store, Origin::File(path.to_string()), known).resolve(text, parsed, ambient)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::LoadedRelease;
    use crate::parse::{ParseOptions, parse};
    use serde_json::{Value, json};
    use std::collections::HashMap;

    /// The packs of a registry, by name, from JSON.
    fn packs_of(packs: Value) -> HashMap<String, Vec<LoadedRelease>> {
        serde_json::from_value(packs).expect("packs")
    }

    /// `input.comp.md` among `files` resolved, with `packs` (as JSON) and the `ambient` packs.
    fn resolved(files: &[(&str, &str)], packs: Value, ambient: &[&str]) -> Resolved {
        let files: HashMap<String, String> = files
            .iter()
            .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
            .collect();
        let packs = packs_of(packs);
        let text = &files["input.comp.md"];
        let ambient = Ambient {
            packs: ambient.iter().map(|pack| (*pack).to_string()).collect(),
            report: false,
        };
        let parsed = parse(text, &ParseOptions::default());
        let store = Store {
            files: &files,
            packs: &packs,
        };
        resolve(
            "input.comp.md",
            text.clone(),
            parsed,
            store,
            &ambient,
            &mut Parses::default(),
        )
    }

    /// The component `name` stands for in the processed document.
    fn bound<'r>(resolved: &'r Resolved, name: &str) -> &'r Component {
        let main = resolved.document(resolved.main);
        let Some(Binding::Component(id, _)) = main.names.get(name) else {
            panic!("`{name}` is bound");
        };
        resolved.component(*id)
    }

    #[test]
    fn a_document_is_parsed_again_only_when_its_text_changed() {
        let text = "---\nmd:\n  import:\n    - from: ./kit.comp.md\n      names: [card]\n---\n\n:: card[x]\n";
        let kit = |body: &str| format!("```compmd component=card\n---\n{body}\n```\n");
        let mut known = Parses::default();
        let mut resolve_with = |kit: &str| {
            let files = HashMap::from([("kit.comp.md".to_string(), kit.to_string())]);
            let store = Store {
                files: &files,
                packs: &(),
            };
            let parsed = parse(text, &ParseOptions::default());
            let resolved = resolve(
                "input.comp.md",
                text.to_string(),
                parsed,
                store,
                &Ambient::default(),
                &mut known,
            );
            resolved
                .ids()
                .map(|id| Rc::clone(&resolved.document(id).root))
                .collect::<Vec<_>>()
        };
        let first = resolve_with(&kit("One."));
        let same = resolve_with(&kit("One."));
        let changed = resolve_with(&kit("Two."));
        assert_eq!(first.len(), 3, "core, the processed document and the kit");
        let shared = |one: &[Rc<Root>], other: &[Rc<Root>]| {
            one.iter()
                .zip(other)
                .filter(|(one, other)| Rc::ptr_eq(one, other))
                .count()
        };
        assert_eq!(shared(&first, &same), 2, "core and the kit are kept");
        assert_eq!(
            shared(&first, &changed),
            1,
            "the changed kit is parsed again"
        );
    }

    #[test]
    fn core_is_built_in_even_when_the_host_has_a_pack_of_its_name() {
        let impostor = json!({ "@compmd/core": [{ "version": "1.0.0", "files": {
            "package.json": "{\"name\": \"@compmd/core\", \"version\": \"1.0.0\"}",
            "index.comp.md": "Not core.\n",
        } }] });
        let packs = packs_of(impostor.clone());
        let store = Store {
            files: &(),
            packs: &packs,
        };
        let manifest = Origin::Pack {
            name: "@compmd/core".into(),
            version: semver::Version::new(1, 0, 0),
            path: "package.json".into(),
        };
        assert_eq!(store.read(&manifest), None, "the host isn't asked");
        let resolved = resolved(
            &[("input.comp.md", ":: box[B]\n")],
            impostor,
            &["@compmd/core"],
        );
        assert_eq!(resolved.documents.len(), 2, "the document and core, once");
        let core = resolved.document(bound(&resolved, "box").document);
        assert!(core.origin.is_core());
        assert_eq!(core.text, CORE_TEXT);
    }

    #[test]
    fn a_component_file_is_bound_with_its_signature_fence() {
        let input = "---\nmd:\n  import:\n    - from: ./charts/Chart.jsx\n      names: [{name: Chart, as: plot}]\n---\n\n```compmd component=plot signature\nparams:\n  rows: array\n```\n";
        let resolved = resolved(
            &[
                ("input.comp.md", input),
                ("charts/Chart.jsx", "export default () => null;"),
            ],
            json!({}),
            &[],
        );
        let plot = bound(&resolved, "plot");
        assert!(
            matches!(&plot.body, Body::File { origin: Origin::File(path), .. } if path == "charts/Chart.jsx")
        );
        assert!(plot.signature.is_some() && plot.body.supported() && plot.body.is_host());
    }

    #[test]
    fn a_packs_own_file_is_loaded_once() {
        let kit = json!({ "@acme/kit": [{ "version": "2.0.0", "files": {
            "package.json": "{\"name\": \"@acme/kit\", \"version\": \"2.0.0\"}",
            "index.comp.md": "---\nmd:\n  import:\n    - from: ./parts.comp.md\n      names: [part]\n  export:\n    names: [part]\n---\n",
            "parts.comp.md": "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=part\n---\nPart.\n```\n",
        } }] });
        let input = "---\nmd:\n  import:\n    - from: \"@acme/kit@2\"\n      names: [part]\n    - from: \"@acme/kit\"\n      names: \"*\"\n---\n";
        let resolved = resolved(&[("input.comp.md", input)], kit, &[]);
        let parts = resolved
            .documents
            .iter()
            .filter(|document| document.origin.to_string() == "@acme/kit@2.0.0/parts.comp.md");
        assert_eq!(parts.count(), 1);
    }

    #[test]
    fn an_ambient_packs_newest_release_gives_the_processors_components() {
        let release = |version: &str, component: &str| {
            json!({ "version": version, "files": {
                "package.json": format!("{{\"name\": \"@host/c\", \"version\": \"{version}\"}}"),
                "index.comp.md": format!("---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component={component}\n---\nX.\n```\n\n```compmd component=tag signature\nparams: {{}}\n```\n"),
            } })
        };
        let packs = json!({ "@host/c": [release("1.0.0", "old"), release("1.1.0", "new")] });
        let resolved = resolved(&[("input.comp.md", ":: tag[T]\n")], packs, &["@host/c"]);
        assert_eq!(bound(&resolved, "tag").body, Body::Processor);
        assert_eq!(bound(&resolved, "new").name, "new");
        assert!(resolved.document(resolved.main).names.get("old").is_none());
    }
}
