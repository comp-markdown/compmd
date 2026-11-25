//! Stage 2, resolving (spec §6, §11): every component name bound to the component it stands for, through local
//! definitions, imports, `@compmd/core` and ambient packs.
//!
//! The processor does no I/O: imported documents and component files come from the host's [`Files`], packs from its
//! [`Packs`]. A host loads what they serve before it resolves, so resolving needs no waiting.

mod fonts;
mod language;
mod libraries;
mod load;
mod names;
mod origin;
mod path_refusal;
mod reached;
mod source;
mod uses;

pub(crate) use fonts::{Fallback, Font, FontBinding, FontFile, FontId, FontNames, Style, Weight};
pub use language::Language;
use libraries::Libraries;
pub(crate) use names::{Binding, Names, Renderer};
pub(crate) use origin::Origin;
pub(crate) use uses::{code_blocks, invocations};

use crate::analysis::ComponentProps;
use crate::diagnostic::{Diagnostic, Location};
use crate::host::{Files, NeededPack, Packs, Release};
use crate::line_index::LineIndex;
use crate::lines::Lines;
use crate::parse::{Parsed, Spliced};
use crate::policy::{Policy, Trust};
use crate::reserved::Keyword;
use crate::settings::Settings;
use crate::tree::{Moved, Node, Param, Point, Position, Print, Root, Shift, Signature};
use compmd_pack::{Integrity, PackName, Place, ReleaseId};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::rc::Rc;

/// The document of `@compmd/core`, built in (spec §9).
pub(crate) const CORE_TEXT: &str = include_str!("core.comp.md");

/// The packs every document imports, at the lowest priority (spec §6.3), and whether to report their names.
///
/// ```
/// use compmd::Ambient;
///
/// let ambient = Ambient { packs: vec!["@acme/kit".parse()?], report: true };
/// # Ok::<(), compmd::pack::Invalid>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ambient {
    /// The packs. Their newest release is used.
    pub packs: Vec<PackName>,
    /// Whether to report each use of a name from them (W113).
    pub report: bool,
}

/// Where documents and the files they name come from: the host's files and packs, and `@compmd/core`, which the
/// processor serves itself: the host is never asked for a pack of `@compmd/` (spec §6.4).
#[derive(Clone, Copy)]
pub(crate) struct Store<'h> {
    pub(crate) files: &'h dyn Files,
    pub(crate) packs: &'h dyn Packs,
    /// What reads a host-language component's props, for component analysis (W104).
    pub(crate) props: &'h dyn ComponentProps,
}

impl Store<'_> {
    /// The text of the file at `origin`, or `None` when there is none.
    pub(crate) fn read(&self, origin: &Origin) -> Option<String> {
        match origin {
            Origin::File(path) => self.files.file(path.as_str()),
            _ if origin.is_core() => Some(CORE_TEXT.to_string()),
            Origin::Pack(place) if place.release.pack.is_reserved() => None,
            Origin::Pack(place) => self.packs.pack_file(place),
            Origin::Unplaced(_) => None,
        }
    }

    /// Every release of the host's `pack`; none of `@compmd/`.
    pub(crate) fn releases(&self, pack: &PackName) -> Vec<Release> {
        if pack.is_reserved() {
            Vec::new()
        } else {
            self.packs.releases(pack)
        }
    }

    /// Whether the host is still getting `pack`; never one of `@compmd/`.
    pub(crate) fn pending(&self, pack: &PackName) -> bool {
        !pack.is_reserved() && self.packs.pending(pack)
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
    pub(crate) line_starts: LineIndex,
    /// Its tree.
    pub(crate) root: Rc<Root>,
    /// What each component name in it stands for.
    pub(crate) names: Names,
    /// What it exports, by public name.
    pub(crate) exports: BTreeMap<String, Binding>,
    /// What each font name in it stands for (spec §9.4).
    pub(crate) fonts: FontNames,
    /// The fonts it exports, by public name.
    pub(crate) font_exports: BTreeMap<String, FontBinding>,
    /// The code renderer of each language its names give one to (spec §7.7).
    pub(crate) renderers: BTreeMap<String, Renderer>,
    /// Whether the processor provides it (`@compmd/core`, an ambient pack), so that a signature fence alone in it
    /// is a component the processor implements.
    pub(crate) provided: bool,
    /// The release each pack it imports took, by the pack's name, the ambient packs' among them: what a data `from`
    /// of `@scope/pack/file` in it reads (spec §8.6).
    pub(crate) packs: BTreeMap<PackName, ReleaseId>,
    /// What each of its imports loaded, by where the import's `from` is: a document, or a component file.
    pub(crate) imported: Vec<(Location, Origin)>,
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
    /// For a host-language component, whether one of this processor's bindings runs it, on a runtime the host's
    /// `policy` runs (W102 otherwise).
    pub(crate) fn supported(&self, policy: &Policy) -> bool {
        let runs = |language: Language, runtime: Option<&str>| {
            language.runs(runtime)
                && language
                    .runtime(runtime)
                    .is_none_or(|runtime| policy.runs(runtime))
        };
        match self {
            Self::Host {
                language, runtime, ..
            } => Language::from_name(language)
                .is_some_and(|language| runs(language, runtime.as_deref())),
            Self::File { language, .. } => runs(*language, None),
            Self::Compmd(_) | Self::Processor => true,
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
    /// Where its signature is written: its definition, or a signature fence apart from it, which an edit between the
    /// two moves on its own.
    pub(crate) signature_at: Option<Position>,
    /// What implements it.
    pub(crate) body: Body,
    /// The definition fence's param list, which styles the component's box.
    pub(crate) fence: Vec<Param>,
    /// Where it is defined.
    pub(crate) position: Option<Position>,
}

/// A host-language component, with what a renderer needs to run it (spec §7.4): the name the expanded tree gives it,
/// its language and runtime, and its code. It serializes to the JSON `@compmd/sandbox` and `@compmd/render` read
/// (`HostComponent` there), leaving out what is empty.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct HostComponent {
    /// The name a document gives it, as its nodes in the expanded tree have it.
    pub name: String,
    /// The language: the definition fence's first word, or the component file's extension. `None` for a component
    /// the host implements (spec §7.1), a signature fence alone in an ambient pack, which has no code: the host's
    /// own code runs it, at the trusted level only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Language>,
    /// The fence's `runtime=`; `None` for the binding's default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// The code: the fence's content, or the component file's text; empty for a built module, which loads from
    /// `module`, and for a component the host implements.
    pub code: String,
    /// For a built module, a `.js` file in a pack (spec §6.4), where it is: the renderer loads it from the pack's
    /// files instead of passing its text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<Module>,
    /// For a source component file in a pack (`.jsx`, `.tsx`, `.svelte`, `.ts`), where it is: a host that compiled it
    /// ahead (spec §6.4) gives the renderer the compiled module by this place, and `code` is compiled otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub place: Option<Place>,
    /// How far its code is trusted: its pack's level in the host's policy (spec §15).
    pub trust: Trust,
    /// The pack it comes from; `None` for a document's own. A permissive pack's code runs in that pack's frames only,
    /// so the network and `host` one pack is given reach no other code (spec §15).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack: Option<PackName>,
    /// For a component of a pack, the library packs its pack names in `peerDependencies`, by the name its code
    /// imports, with the module chosen for the document (the companion, section 5); empty for any other.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub libraries: BTreeMap<String, LibraryModule>,
    /// What its uses show on paper, from its signature (spec §7.3); in JSON only when it is their fallback.
    #[serde(skip_serializing_if = "Print::is_drawn")]
    pub print: Print,
    /// For a component of an example (spec §7.8), the example's number, as its expanded root gives it: a renderer
    /// finds the components an example's nodes name among those of its number; absent for the processed document's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<usize>,
}

/// A library pack's module, chosen for a document, and the libraries it imports in turn, by name. In JSON its place
/// is flat: `{pack, version, path, libraries}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LibraryModule {
    /// The module's place: the library pack, the version chosen, and the path its `compmd` field names.
    #[serde(flatten)]
    pub place: Place,
    /// The libraries it imports, by name.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub libraries: BTreeMap<String, LibraryModule>,
}

/// Where a built module is in its pack (spec §6.4). In JSON its place is flat: `{pack, version, path, integrity}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Module {
    /// The file's place.
    #[serde(flatten)]
    pub place: Place,
    /// The integrity the registry publishes for the pack, which the files it is loaded from are checked against.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity: Option<Integrity>,
}

impl Component {
    /// It as a host-language component named `name`, when one of the processor's bindings runs it on a runtime the
    /// host's `policy` runs, and its code is there: a fence's, its file's from `store`, or a built module's place;
    /// or, implemented by the processor, as one the host implements, with no code; trusted as `trust`, importing
    /// `libraries`.
    fn host_code(
        &self,
        name: &str,
        store: Store<'_>,
        policy: &Policy,
        trust: Trust,
        libraries: BTreeMap<String, LibraryModule>,
    ) -> Option<HostComponent> {
        self.body.supported(policy).then_some(())?;
        let (language, runtime, code, module, place) = match &self.body {
            Body::Host {
                language,
                runtime,
                source,
            } => (
                Some(Language::from_name(language)?),
                runtime.clone(),
                source.clone(),
                None,
                None,
            ),
            Body::File {
                origin: Origin::Pack(place),
                language: Language::Js,
            } => {
                let integrity = store
                    .releases(&place.release.pack)
                    .into_iter()
                    .find(|release| release.version == place.release.version)
                    .and_then(|release| release.integrity);
                let module = Module {
                    place: place.clone(),
                    integrity,
                };
                (Some(Language::Js), None, String::new(), Some(module), None)
            }
            Body::File { origin, language } => {
                let place = match origin {
                    Origin::Pack(place) => Some(place.clone()),
                    Origin::File(_) | Origin::Unplaced(_) => None,
                };
                (Some(*language), None, store.read(origin)?, None, place)
            }
            Body::Processor => (None, None, String::new(), None, None),
            Body::Compmd(_) => return None,
        };
        Some(HostComponent {
            name: name.to_string(),
            language,
            runtime,
            code,
            module,
            place,
            trust,
            pack: None,
            libraries,
            print: self
                .signature
                .as_ref()
                .map_or(Print::Drawn, |signature| signature.print),
            example: None,
        })
    }

    /// The built-in keyword it is, if any. Resolving admits a keyword as a name only in `@compmd/core` (F013), so a
    /// component with a keyword's name is always core's.
    pub(crate) fn keyword(&self) -> Option<Keyword> {
        Keyword::from_name(&self.name)
    }

    /// Whether its declared params take breakpoints: a `compmd` component's, and `box`'s, which are layout params
    /// (spec §9.3); not a host-language component's.
    pub(crate) fn takes_breakpoints(&self) -> bool {
        match self.body {
            Body::Compmd(_) => true,
            Body::Processor => self.keyword() == Some(Keyword::Box),
            Body::Host { .. } | Body::File { .. } => false,
        }
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
    /// Every font those documents declare, each with its one name in the expanded tree.
    pub(crate) fonts: Vec<Font>,
    /// The processed document.
    pub(crate) main: DocumentId,
    /// The invocations that close a cycle of definitions (F019), by their document and where they start.
    pub(crate) cycles: HashSet<(DocumentId, Point)>,
    /// The host's policy, which gives each component's trust.
    pub(crate) policy: Policy,
    /// The pack imports the host has no release for, each once, in the order they were loaded, then the library
    /// packs the packs name that it has none of.
    pub(crate) missing: Vec<NeededPack>,
    /// The pack imports that are pending (spec §6.4), by pack, each once, in the order they were loaded.
    pub(crate) pending: Vec<PackName>,
    /// The library packs chosen for the document.
    pub(crate) libraries: Libraries,
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

    /// The font `id` names.
    pub(crate) fn font(&self, id: FontId) -> &Font {
        &self.fonts[id.0]
    }

    /// The font whose name in the expanded tree is `family`.
    pub(crate) fn font_of_family(&self, family: &str) -> Option<&Font> {
        self.fonts.iter().find(|font| font.family == family)
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
    pub(crate) fn take_text(&mut self) -> (String, LineIndex) {
        let main = &mut self.documents[self.main.0];
        main.root = Rc::default();
        let lines = std::mem::replace(&mut main.line_starts, LineIndex::new(""));
        (std::mem::take(&mut main.text), lines)
    }

    /// Every host-language component the processed document reaches (`reached.rs`) that one of this processor's
    /// bindings runs, by each name a document gives it, with its code from `store`, and every component the host
    /// implements that it reaches (an ambient pack's signature fence alone, spec §7.1); by name. A name two documents
    /// give different components is the processed document's, or else the first document's in load order.
    pub(crate) fn host_components(&self, store: Store<'_>) -> Vec<HostComponent> {
        let reached = reached::reached(self);
        let mut found: BTreeMap<&str, HostComponent> = BTreeMap::new();
        let main_first = std::iter::once(self.main).chain(self.ids().filter(|id| *id != self.main));
        for id in main_first {
            for (name, component) in self.document(id).names.components() {
                if found.contains_key(name) || !reached.contains(&component) {
                    continue;
                }
                let component = self.component(component);
                let origin = &self.document(component.document).origin;
                // `@compmd/core`'s components are the renderer's own, never the host's.
                if origin.is_core() {
                    continue;
                }
                let (trust, libraries, pack) = match origin {
                    Origin::Pack(place) => (
                        self.policy.trust(&place.release.pack),
                        self.libraries.of(&place.release),
                        Some(place.release.pack.clone()),
                    ),
                    Origin::File(_) | Origin::Unplaced(_) => (Trust::Strict, BTreeMap::new(), None),
                };
                if let Some(host) = component.host_code(name, store, &self.policy, trust, libraries)
                {
                    found.insert(name, HostComponent { pack, ..host });
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
        (text, lines): (String, LineIndex),
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
            if component.document == main {
                component.follow(moved);
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

impl Component {
    /// Moves what `moved` moves of it, each part with the block it is written in: its definition and body with the
    /// definition, its signature where it is written, and each param of its box's fence with that param, so a
    /// signature fence below its definition moves when an edit between the two moves it.
    fn follow(&mut self, moved: Moved) {
        let after_the_edit =
            |position: Option<Position>| position.is_some_and(|at| moved.moves(at.start));
        if after_the_edit(self.signature_at) {
            self.signature.shift(moved.by);
            self.signature_at.shift(moved.by);
        }
        for param in &mut self.fence {
            if after_the_edit(param.position) {
                param.shift(moved.by);
            }
        }
        if after_the_edit(self.position) {
            if let Body::Compmd(body) = &mut self.body {
                body.shift(moved.by);
            }
            self.position.shift(moved.by);
        }
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
    host: Host<'_>,
    known: &mut Parses,
) -> Resolved {
    load::Loader::new(store, Origin::main(path), known, host.policy).resolve(
        text,
        parsed,
        host.ambient,
    )
}

/// What the host configures about packs: the ambient ones, and its policy.
#[derive(Clone, Copy)]
pub(crate) struct Host<'a> {
    pub(crate) ambient: &'a Ambient,
    pub(crate) policy: &'a Policy,
}

#[cfg(test)]
mod tests;
