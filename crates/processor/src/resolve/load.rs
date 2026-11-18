//! Loads a document and everything it imports, and binds the names of each (spec §6): F013, F020–F022, F026 for
//! imports, F029, F035, W105, W112, W117, F002 for a signature fence with nothing to attach to, F037 for a
//! declaration whose type isn't a type name, and W121 for a pack that needs what the host doesn't have.

mod definitions;
mod exports;
mod packs;

use super::libraries::{self, Ask};
use super::names::{Binding, Builder, Level, Names, Renderer};
use super::origin::CORE;
use super::source::Source;
use super::{
    Ambient, Body, Component, ComponentId, Document, DocumentId, Found, Language, Origin, Parses,
    Resolved, Store, uses,
};
use crate::diagnostic::{Code, Diagnostic, Fix, Location, Stage};
use crate::host::NeededPack;
use crate::lines::LineStarts;
use crate::parse::{ParseOptions, Parsed, parse};
use crate::policy::Policy;
use crate::reserved;
use crate::settings::{self, Import};
use crate::tree::start_of as location;
use compmd_pack::PackName;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Why an import loads nothing: the diagnostic it gets, where, and what it says. Its names are dropped.
struct Refusal {
    code: Code,
    at: Location,
    message: String,
}

impl Refusal {
    fn new(code: Code, at: Location, message: impl Into<String>) -> Self {
        Self {
            code,
            at,
            message: message.into(),
        }
    }
}

/// Loads documents, collecting them, their components and the diagnostics.
pub(super) struct Loader<'h, 'p> {
    store: Store<'h>,
    /// The parses of documents loaded before, which a document whose text hasn't changed reuses.
    parses: &'p mut Parses,
    /// The processed document, which diagnostics name with no file.
    main: Origin,
    documents: Vec<Document>,
    components: Vec<Component>,
    diagnostics: Vec<Diagnostic>,
    /// The documents being loaded, outermost first: importing one of them closes a cycle (F020).
    loading: Vec<Origin>,
    loaded: HashMap<Origin, DocumentId>,
    core: Option<DocumentId>,
    ambient: Vec<DocumentId>,
    /// The ambient packs.
    ambient_names: Vec<PackName>,
    /// The host's policy, which may refuse a pack.
    policy: &'h Policy,
    /// The pack imports the host has no release for, in the order they are loaded.
    missing: Vec<NeededPack>,
    /// The pack imports that are pending, by pack, in the order they are loaded.
    pending: Vec<PackName>,
    /// The library packs the loaded packs name, in the order they are loaded.
    asks: Vec<Ask>,
    /// Where each component file import is written, its `names` entry: it has no definition fence to report at.
    file_imports: HashMap<ComponentId, Location>,
}

impl<'h, 'p> Loader<'h, 'p> {
    pub(super) fn new(
        store: Store<'h>,
        main: Origin,
        parses: &'p mut Parses,
        policy: &'h Policy,
    ) -> Self {
        Self {
            store,
            parses,
            main,
            documents: Vec::new(),
            components: Vec::new(),
            diagnostics: Vec::new(),
            loading: Vec::new(),
            loaded: HashMap::new(),
            core: None,
            ambient: Vec::new(),
            ambient_names: Vec::new(),
            policy,
            missing: Vec::new(),
            pending: Vec::new(),
            asks: Vec::new(),
            file_imports: HashMap::new(),
        }
    }

    /// Loads `@compmd/core`, the ambient packs and the processed document, then reports the names its bodies use.
    pub(super) fn resolve(mut self, text: String, parsed: Parsed, ambient: &Ambient) -> Resolved {
        self.core = self
            .document_at(Origin::core(), Location::new(1, 1), true)
            .ok();
        self.ambient_names.clone_from(&ambient.packs);
        self.ambient = ambient
            .packs
            .iter()
            .filter_map(|pack| self.ambient_pack(pack))
            .collect();
        let main = self.load(self.main.clone(), text, parsed, false);
        let chosen = libraries::choose(self.store, self.policy, &self.ambient_names, &self.asks);
        for unmet in &chosen.unmet {
            let (id, at) = unmet.at;
            self.unmet(id, &unmet.asker, at, &unmet.missing);
        }
        for needed in chosen.needed {
            if !self.missing.contains(&needed) {
                self.missing.push(needed);
            }
        }
        let mut resolved = Resolved {
            documents: self.documents,
            components: self.components,
            main,
            cycles: HashSet::new(),
            policy: self.policy.clone(),
            missing: self.missing,
            pending: self.pending,
            libraries: chosen.libraries,
            diagnostics: Found {
                before: self.diagnostics,
                ..Found::default()
            },
        };
        uses::report(&mut resolved, ambient.report);
        resolved
    }

    /// Loads a document: its imports, its definitions, its names and its exports.
    fn load(&mut self, origin: Origin, text: String, parsed: Parsed, provided: bool) -> DocumentId {
        let id = DocumentId(self.documents.len());
        let file = (origin != self.main).then(|| origin.to_string());
        self.diagnostics.extend(
            parsed
                .diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.in_file(file.clone())),
        );
        self.documents.push(Document {
            origin: origin.clone(),
            line_starts: LineStarts::new(&text),
            text,
            settings: parsed.settings,
            root: parsed.root,
            names: Names::default(),
            exports: BTreeMap::new(),
            renderers: BTreeMap::new(),
            provided,
        });
        self.loaded.insert(origin.clone(), id);
        self.loading.push(origin);
        let mut names = Builder::default();
        for import in self.document(id).settings.imports.clone() {
            self.import(id, &import, &mut names);
        }
        self.implicit(id, &mut names);
        self.define(id, &mut names);
        self.check_frontmatter_types(id);
        let names = names.build();
        let exports = self.exports(id, &names);
        self.untyped(id, &exports);
        let renderers = renderers(&names, &self.components);
        let document = &mut self.documents[id.0];
        document.names = names;
        document.exports = exports;
        document.renderers = renderers;
        self.loading.pop();
        id
    }

    fn document(&self, id: DocumentId) -> &Document {
        &self.documents[id.0]
    }

    fn report(&mut self, id: DocumentId, code: Code, at: Location, message: impl Into<String>) {
        self.report_fixed(id, code, at, message, None);
    }

    fn report_fixed(
        &mut self,
        id: DocumentId,
        code: Code,
        at: Location,
        message: impl Into<String>,
        fix: Option<Fix>,
    ) {
        let origin = &self.document(id).origin;
        let file = (*origin != self.main).then(|| origin.to_string());
        self.diagnostics.push(
            Diagnostic::new(code, at, message)
                .in_stage(Stage::Resolve)
                .in_file(file)
                .with_fix(fix),
        );
    }

    /// Loads what `import` names and binds its names; when it loads nothing, its names are dropped.
    fn import(&mut self, id: DocumentId, import: &Import, names: &mut Builder) {
        let from = &import.from;
        let loaded = match Source::parse(&from.value) {
            Err(why) => Err(Refusal::new(Code::F026, from.at, why.to_string())),
            Ok(Source::Document(path)) => self.relative(id, path, from.at),
            Ok(Source::File {
                path,
                language,
                stem,
            }) => {
                self.file(id, import, path, language, stem, names);
                return;
            }
            Ok(Source::Pack(spec)) if self.pending(&spec) => {
                self.missing(&spec, import);
                if !self.pending.contains(&spec.name) {
                    self.pending.push(spec.name.clone());
                }
                pend_names(&import.names, &spec.name, names);
                return;
            }
            Ok(Source::Pack(spec)) => self.pack(id, &spec, import),
        };
        match loaded {
            Ok(target) => self.bind(id, import, target, names),
            Err(refusal) => {
                self.report(id, refusal.code, refusal.at, refusal.message);
                drop_names(&import.names, names);
            }
        }
    }

    /// The document at `path`, relative to the document `id`.
    fn relative(
        &mut self,
        id: DocumentId,
        path: &str,
        at: Location,
    ) -> Result<DocumentId, Refusal> {
        let document = self.document(id);
        let origin = document
            .origin
            .join(path)
            .ok_or_else(|| Refusal::new(Code::F026, at, format!("`{path}` leaves its root")))?;
        self.document_at(origin, at, document.provided)
    }

    /// The document at `origin`, loaded unless it is already.
    fn document_at(
        &mut self,
        origin: Origin,
        at: Location,
        provided: bool,
    ) -> Result<DocumentId, Refusal> {
        if self.loading.contains(&origin) {
            return Err(Refusal::new(
                Code::F020,
                at,
                format!("importing `{origin}` closes an import cycle; the import is dropped"),
            ));
        }
        if let Some(id) = self.loaded.get(&origin) {
            return Ok(*id);
        }
        let text = self
            .store
            .read(&origin)
            .ok_or_else(|| Refusal::new(Code::F026, at, format!("`{origin}` can't be loaded")))?;
        let parsed = match self.parses.0.get(&origin) {
            Some((before, parsed)) if *before == text => parsed.clone(),
            _ => {
                let parsed = parse(&text, &ParseOptions::default());
                self.parses
                    .0
                    .insert(origin.clone(), (text.clone(), parsed.clone()));
                parsed
            }
        };
        Ok(self.load(origin, text, parsed, provided))
    }

    /// Imports a component file (`./chart.jsx`), which exports one component, named after its stem.
    fn file(
        &mut self,
        id: DocumentId,
        import: &Import,
        path: &str,
        language: Language,
        stem: &str,
        names: &mut Builder,
    ) {
        let at = import.from.at;
        let Some(origin) = self.document(id).origin.join(path) else {
            self.report(id, Code::F026, at, format!("`{path}` leaves its root"));
            drop_names(&import.names, names);
            return;
        };
        if self.store.read(&origin).is_none() {
            self.report(id, Code::F026, at, format!("`{origin}` can't be loaded"));
            drop_names(&import.names, names);
            return;
        }
        let entries = match &import.names {
            settings::Names::All(at) => vec![(stem.to_string(), *at)],
            settings::Names::Listed(entries) => entries
                .iter()
                .filter_map(|entry| {
                    if entry.name == stem {
                        return Some((entry.renamed.clone(), entry.at));
                    }
                    self.report(
                        id,
                        Code::F021,
                        entry.at,
                        format!(
                            "`{path}` exports one component, `{stem}`, not `{}`",
                            entry.name
                        ),
                    );
                    None
                })
                .collect(),
        };
        for (local, at) in entries {
            if self.admit(id, names, &local, at) {
                let component = self.add(Component {
                    name: local.clone(),
                    document: id,
                    signature: None,
                    body: Body::File {
                        origin: origin.clone(),
                        language,
                    },
                    fence: Vec::new(),
                    position: None,
                });
                self.file_imports.insert(component, at);
                names.declare(&local, Binding::Component(component, Level::Listed));
            }
        }
    }

    /// Binds the names `import` takes from the loaded document `target`.
    fn bind(&mut self, id: DocumentId, import: &Import, target: DocumentId, names: &mut Builder) {
        let exports = self.document(target).exports.clone();
        match &import.names {
            settings::Names::All(_) => {
                for (name, binding) in exports {
                    if let Binding::Component(component, _) = binding {
                        names.share(&name, component, &import.from.value, Level::Wildcard);
                    }
                }
            }
            settings::Names::Listed(entries) => {
                for entry in entries {
                    let binding = match exports.get(&entry.name) {
                        Some(Binding::Component(component, _)) => {
                            Binding::Component(*component, Level::Listed)
                        }
                        Some(other) => other.clone(),
                        None => {
                            self.report(
                                id,
                                Code::F021,
                                entry.at,
                                format!("`{}` doesn't export `{}`", import.from.value, entry.name),
                            );
                            Binding::Dropped
                        }
                    };
                    if self.admit(id, names, &entry.renamed, entry.at) {
                        names.declare(&entry.renamed, binding);
                    }
                }
            }
        }
    }

    /// `@compmd/core` and the ambient packs, which every document imports.
    fn implicit(&self, id: DocumentId, names: &mut Builder) {
        let core = self.core.filter(|core| *core != id);
        for (name, binding) in core.iter().flat_map(|core| &self.document(*core).exports) {
            if let Binding::Component(component, _) = binding {
                names.share(name, *component, CORE, Level::Core);
            }
        }
        for pack in self.ambient.iter().filter(|pack| **pack != id) {
            for (name, binding) in &self.document(*pack).exports {
                if let Binding::Component(component, _) = binding {
                    names.ambient(name, *component);
                }
            }
        }
    }

    /// Whether `name` may be declared at the first two levels: not a keyword or `md`, which `@compmd/core` alone
    /// defines, and not declared yet (F013). Shadowing a `@compmd/core` component is allowed, with W112.
    fn admit(&mut self, id: DocumentId, names: &Builder, name: &str, at: Location) -> bool {
        let core = self.document(id).origin.is_core();
        if let Some(why) = reserved::as_component(name).filter(|_| !core) {
            self.report(id, Code::F013, at, why);
            return false;
        }
        if names.is_declared(name) {
            self.report(
                id,
                Code::F013,
                at,
                format!(
                    "`{name}` is already declared in this document; the first declaration wins"
                ),
            );
            return false;
        }
        let core = self.core.filter(|core| *core != id);
        if core.is_some_and(|core| self.document(core).exports.contains_key(name)) {
            self.report(
                id,
                Code::W112,
                at,
                format!("`{name}` shadows `@compmd/core`'s `{name}` in this document"),
            );
        }
        true
    }

    fn add(&mut self, component: Component) -> ComponentId {
        self.components.push(component);
        ComponentId(self.components.len() - 1)
    }
}

/// The names of an import that loads nothing: listed ones stand for nothing, without F002; a wildcard import may
/// have held any name.
fn drop_names(listed: &settings::Names, names: &mut Builder) {
    match listed {
        settings::Names::All(_) => names.incomplete(),
        settings::Names::Listed(entries) => {
            for entry in entries {
                if !names.is_declared(&entry.renamed) {
                    names.declare(&entry.renamed, Binding::Dropped);
                }
            }
        }
    }
}

/// The names of an import of `pack`, which is pending: listed ones stand for pending components; a wildcard import
/// may hold any name.
fn pend_names(listed: &settings::Names, pack: &PackName, names: &mut Builder) {
    match listed {
        settings::Names::All(_) => names.pending(pack),
        settings::Names::Listed(entries) => {
            for entry in entries {
                if !names.is_declared(&entry.renamed) {
                    names.declare(&entry.renamed, Binding::Pending(pack.clone()));
                }
            }
        }
    }
}

/// The code renderer of each language the components of `names` render (spec §6.3, §7.7): the one at the highest
/// level that has one; two or more there are ambiguous.
fn renderers(names: &Names, components: &[Component]) -> BTreeMap<String, Renderer> {
    /// The rank of the highest level with a renderer for a language, and the renderers there, by name.
    type Best<'n> = (u8, Vec<(&'n str, ComponentId)>);
    let mut found: BTreeMap<&str, Best> = BTreeMap::new();
    for (name, id, level) in names.leveled() {
        let languages = components[id.0]
            .signature
            .iter()
            .flat_map(|signature| signature.renders.iter().flatten());
        for language in languages {
            let rank = level.rank();
            let (best, candidates) = found.entry(language).or_insert((rank, Vec::new()));
            if rank < *best {
                (*best, *candidates) = (rank, Vec::new());
            }
            if rank == *best && !candidates.iter().any(|(_, known)| *known == id) {
                candidates.push((name, id));
            }
        }
    }
    found
        .into_iter()
        .map(|(language, (_, candidates))| {
            let renderer = match candidates[..] {
                [(name, id)] => Renderer::Component(name.to_string(), id),
                _ => Renderer::Ambiguous(
                    candidates
                        .iter()
                        .map(|(name, _)| (*name).to_string())
                        .collect(),
                ),
            };
            (language.to_string(), renderer)
        })
        .collect()
}
