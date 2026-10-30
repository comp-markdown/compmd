//! Loads a document and everything it imports, and binds the names of each (spec §6): F013, F020–F022, F026 for
//! imports, F029, F035, W105, W112, W117, F002 for a signature fence with nothing to attach to, and F037 for a
//! declaration whose type isn't a type name.

use super::names::{Binding, Builder, Level, Names, Renderer};
use super::origin::CORE;
use super::source::{PackSpec, Source};
use super::{
    Ambient, Body, Component, ComponentId, Document, DocumentId, Found, Language, Origin, Parses,
    Resolved, Store, uses,
};
use crate::diagnostic::{Code, Diagnostic, Fix, Location, Stage};
use crate::lines::LineStarts;
use crate::nearest::nearest;
use crate::parse::{CODE, ParseOptions, Parsed, parse};
use crate::policy::Policy;
use crate::reserved;
use crate::settings::{self, Import};
use crate::tree::{
    Declaration, DeclaredType, Definition, DefinitionBody, Node, Position, Signature,
    SignatureFence, TypeName,
};
use semver::Version;
use serde_json::Value;
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
    /// The ambient packs, by name.
    ambient_names: Vec<String>,
    /// The host's policy, which may refuse a pack.
    policy: &'h Policy,
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
        let mut resolved = Resolved {
            documents: self.documents,
            components: self.components,
            main,
            cycles: HashSet::new(),
            policy: self.policy.clone(),
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
                names.declare(&local, Binding::Component(component, Level::Listed));
            }
        }
    }

    /// The pack `spec` names: its newest matching release, checked against the import's `integrity`.
    fn pack(
        &mut self,
        id: DocumentId,
        spec: &PackSpec,
        import: &Import,
    ) -> Result<DocumentId, Refusal> {
        let from = &import.from;
        if let Some(why) = self.policy.refusal(&spec.name, &self.ambient_names) {
            return Err(Refusal::new(
                Code::F026,
                from.at,
                format!("{why}; the import is dropped"),
            ));
        }
        let release = self
            .store
            .releases(&spec.name)
            .into_iter()
            .filter(|release| spec.range.matches(&release.version))
            .max_by(|a, b| a.version.cmp(&b.version))
            .ok_or_else(|| {
                Refusal::new(
                    Code::F026,
                    from.at,
                    format!("no release of `{}` matches `{}`", spec.name, from.value),
                )
            })?;
        if let Some(why) = self.policy.denial(&spec.name, &release.version) {
            return Err(Refusal::new(
                Code::F026,
                from.at,
                format!("{why}; the import is dropped"),
            ));
        }
        if let Some(integrity) = &import.integrity {
            if !spec.exact {
                self.report(
                    id,
                    Code::F009,
                    integrity.at,
                    "`integrity` needs an exact version, such as `@1.4.2`, since a range takes whichever release is newest; ignored",
                );
            } else if release.integrity.as_deref() != Some(integrity.value.as_str()) {
                return Err(Refusal::new(
                    Code::F035,
                    integrity.at,
                    format!(
                        "`{}@{}` doesn't match this `integrity`; the import is dropped",
                        spec.name, release.version
                    ),
                ));
            }
        }
        self.package(&spec.name, release.version, from.at, false)
    }

    /// The newest release of the ambient pack `name`.
    fn ambient_pack(&mut self, name: &str) -> Option<DocumentId> {
        let release = self
            .store
            .releases(name)
            .into_iter()
            .max_by(|a, b| a.version.cmp(&b.version))?;
        self.package(name, release.version, Location::new(1, 1), true)
            .ok()
    }

    /// The document of a pack's release, named by its `package.json` (spec §6.4), checked against it (F022).
    fn package(
        &mut self,
        name: &str,
        version: Version,
        at: Location,
        provided: bool,
    ) -> Result<DocumentId, Refusal> {
        let unloadable = || {
            Refusal::new(
                Code::F026,
                at,
                format!("`{name}@{version}` can't be loaded"),
            )
        };
        let manifest: Value = self
            .store
            .read(&Origin::Pack {
                name: name.to_string(),
                version: version.clone(),
                path: "package.json".to_string(),
            })
            .and_then(|text| serde_json::from_str(&text).ok())
            .ok_or_else(unloadable)?;
        let path = manifest
            .get("compmd")
            .and_then(Value::as_str)
            .map_or("index.comp.md", |path| path.trim_start_matches("./"));
        let origin = Origin::Pack {
            name: name.to_string(),
            path: path.to_string(),
            version,
        };
        let fresh = !self.loaded.contains_key(&origin);
        let id = self.document_at(origin, at, provided)?;
        if fresh {
            self.check_manifest(id, &manifest);
        }
        Ok(id)
    }

    /// F022: a pack's `export.pack` and `export.version` must be its manifest's.
    fn check_manifest(&mut self, id: DocumentId, manifest: &Value) {
        let Some(export) = self.document(id).settings.export.clone() else {
            return;
        };
        let differs = |written: &Option<String>, key: &str| {
            written
                .as_deref()
                .is_some_and(|written| manifest.get(key).and_then(Value::as_str) != Some(written))
        };
        if differs(&export.pack, "name") || differs(&export.version, "version") {
            self.report(
                id,
                Code::F022,
                export.at,
                "`export.pack` and `export.version` must match the pack's `package.json`, which is used",
            );
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

    /// The document's definitions and signature fences.
    fn define(&mut self, id: DocumentId, names: &mut Builder) {
        let root = &self.document(id).root;
        let definitions: Vec<Definition> = root
            .children
            .iter()
            .filter_map(|node| match node {
                Node::ComponentDefinition(definition) => Some(definition.clone()),
                _ => None,
            })
            .collect();
        let fences: Vec<SignatureFence> = root
            .children
            .iter()
            .filter_map(|node| match node {
                Node::ComponentSignature(fence) => Some(fence.clone()),
                _ => None,
            })
            .collect();
        for definition in definitions {
            self.definition(id, definition, names);
        }
        for fence in fences {
            self.signature_fence(id, fence, names);
        }
    }

    fn definition(&mut self, id: DocumentId, definition: Definition, names: &mut Builder) {
        let at = location(definition.position);
        if !self.admit(id, names, &definition.name, at) {
            return;
        }
        let compmd = matches!(definition.body, DefinitionBody::Compmd { .. });
        if let Some(signature) = &definition.signature {
            self.check_params(id, signature, at, compmd);
        }
        let body = match definition.body {
            DefinitionBody::Compmd { children } => Body::Compmd(children),
            DefinitionBody::Host { source, runtime } => Body::Host {
                language: definition.language,
                runtime,
                source,
            },
        };
        let component = self.add(Component {
            name: definition.name.clone(),
            document: id,
            signature: definition.signature,
            body,
            fence: definition.params,
            position: definition.position,
        });
        names.declare(
            &definition.name,
            Binding::Component(component, Level::Local),
        );
    }

    /// A signature fence gives its signature to the document's host-language component of that name; in a pack the
    /// processor provides, a fence alone is a component the processor implements (spec §9).
    fn signature_fence(&mut self, id: DocumentId, fence: SignatureFence, names: &mut Builder) {
        let at = location(fence.position);
        if let Some(signature) = &fence.signature {
            self.check_params(id, signature, at, false);
        }
        let host = match names.declared(&fence.name) {
            Some(Binding::Component(component, _)) => Some(*component).filter(|component| {
                let component = &self.components[component.0];
                component.document == id && component.body.is_host()
            }),
            _ => None,
        };
        if let Some(component) = host {
            self.components[component.0]
                .signature
                .get_or_insert(fence.signature.unwrap_or_default());
        } else if self.document(id).provided {
            if self.admit(id, names, &fence.name, at) {
                let component = self.add(Component {
                    name: fence.name.clone(),
                    document: id,
                    signature: fence.signature,
                    body: Body::Processor,
                    fence: Vec::new(),
                    position: fence.position,
                });
                names.declare(&fence.name, Binding::Component(component, Level::Local));
            }
        } else {
            self.report(
                id,
                Code::F002,
                at,
                format!(
                    "the signature fence for `{}` has nothing to attach to: no definition of `{0}` and no component file imported as `{0}`",
                    fence.name
                ),
            );
        }
    }

    /// F013 for a signature's params: two whose names differ only in `-` and `_`, which expressions can't tell apart,
    /// and, for a `compmd` definition, a name expressions can't use (a `name` param binds a name and is never read).
    fn check_params(&mut self, id: DocumentId, signature: &Signature, at: Location, compmd: bool) {
        let mut seen = HashSet::new();
        for param in &signature.params {
            if !seen.insert(param.name.replace('-', "_")) {
                self.report(
                    id,
                    Code::F013,
                    at,
                    format!("two params differ only in `-` and `_` from `{}`, which expressions read alike", param.name),
                );
            }
            let binds_a_name = param.declared == DeclaredType::Name(TypeName::Name);
            if let Some(why) =
                reserved::as_variable(&param.name).filter(|_| compmd && !binds_a_name)
            {
                self.report(id, Code::F013, at, why);
            }
            self.check_type(id, param);
        }
        self.check_code(id, signature, at);
    }

    /// F009 for a code renderer that declares `code` as anything but a required string: the block's text fills it
    /// (spec §7.3).
    fn check_code(&mut self, id: DocumentId, signature: &Signature, at: Location) {
        if signature.renders.is_none() {
            return;
        }
        let declared = signature.params.iter().find(|param| param.name == CODE);
        if let Some(declared) = declared.filter(|declared| {
            declared.declared != DeclaredType::Name(TypeName::String)
                || declared.optional
                || declared.default.is_some()
        }) {
            self.report(
                id,
                Code::F009,
                declared.type_at.unwrap_or(at),
                "a code renderer's `code` holds the block's text, so it is a required `string`, with no default",
            );
        }
    }

    /// F037 for the declarations of the document's frontmatter: its inputs, and its data imports' schemas.
    fn check_frontmatter_types(&mut self, id: DocumentId) {
        let settings = &self.document(id).settings;
        let fields = settings
            .data
            .iter()
            .flat_map(|data| data.schema.iter().flatten())
            .map(|field| &field.declaration);
        let declarations: Vec<Declaration> = settings
            .inputs
            .iter()
            .map(|input| &input.declaration)
            .chain(fields)
            .cloned()
            .collect();
        for declaration in &declarations {
            self.check_type(id, declaration);
        }
    }

    /// F037 for a declaration whose type isn't a type name (spec §7.3), with the nearest one and the fix that writes
    /// it. The declaration is kept: its type takes any value.
    fn check_type(&mut self, id: DocumentId, declaration: &Declaration) {
        let (Some(at), Some(written)) = (declaration.unknown_at, declaration.declared.unknown())
        else {
            return;
        };
        // `enum` is a type only inside `input<…>`.
        let near = nearest(
            written,
            TypeName::NAMES
                .iter()
                .copied()
                .filter(|name| *name != "enum"),
        );
        let hint = near.map_or_else(String::new, |near| format!("; did you mean `{near}`?"));
        let fix = near.map(|near| Fix::replacing(at, written, near));
        self.report_fixed(
            id,
            Code::F037,
            at,
            format!(
                "`{written}` isn't a type, so `{}` takes any value{hint}",
                declaration.name
            ),
            fix,
        );
    }

    /// The document's exports (spec §6.2): F013 for a keyword, F029 and W117 for host-language components.
    fn exports(&mut self, id: DocumentId, names: &Names) -> BTreeMap<String, Binding> {
        let Some(export) = self.document(id).settings.export.clone() else {
            return BTreeMap::new();
        };
        let mut exports = BTreeMap::new();
        match &export.names {
            settings::Names::All(at) => {
                let defined: Vec<ComponentId> = (0..self.components.len())
                    .map(ComponentId)
                    .filter(|component| {
                        let component = &self.components[component.0];
                        component.document == id && !matches!(component.body, Body::File { .. })
                    })
                    .collect();
                for component in defined {
                    let name = self.components[component.0].name.clone();
                    exports.insert(name, Binding::Component(component, Level::Local));
                    self.check_export(id, component, *at);
                }
            }
            settings::Names::Listed(entries) => {
                for entry in entries {
                    if let Some(why) = reserved::as_component(&entry.renamed) {
                        self.report(id, Code::F013, entry.at, why);
                        continue;
                    }
                    let Some(binding) = names.get(&entry.name).cloned() else {
                        continue;
                    };
                    if let Binding::Component(component, _) = binding {
                        self.check_export(id, component, entry.at);
                    }
                    exports.insert(entry.renamed.clone(), binding);
                }
            }
        }
        exports
    }

    /// An exported host-language component needs a signature fence (F029) and, unless plain JavaScript, a runtime
    /// (W117, at its definition, by the document that defines it).
    fn check_export(&mut self, id: DocumentId, component: ComponentId, at: Location) {
        let exported = &self.components[component.0];
        if !exported.body.is_host() {
            return;
        }
        if exported.signature.is_none() {
            let name = exported.name.clone();
            self.report(
                id,
                Code::F029,
                at,
                format!("`{name}` is exported, so it needs a signature fence: packs are used where it may not run"),
            );
        }
        let exported = &self.components[component.0];
        let unpinned = match &exported.body {
            Body::Host {
                language,
                runtime: None,
                ..
            } => Language::from_name(language).is_some_and(Language::has_runtimes),
            _ => false,
        };
        if unpinned && exported.document == id {
            let (name, at) = (exported.name.clone(), location(exported.position));
            self.report(
                id,
                Code::W117,
                at,
                format!("`{name}` is exported without `runtime=`, so it changes when the binding's default moves"),
            );
        }
    }

    /// W105: a local host-language component without a signature fence, whose params can't be typed.
    fn untyped(&mut self, id: DocumentId, exports: &BTreeMap<String, Binding>) {
        let exported: HashSet<ComponentId> = exports
            .values()
            .filter_map(|binding| match binding {
                Binding::Component(component, _) => Some(*component),
                _ => None,
            })
            .collect();
        let untyped: Vec<(String, Location)> = self
            .components
            .iter()
            .enumerate()
            .filter(|(index, component)| {
                component.document == id
                    && matches!(component.body, Body::Host { .. })
                    && component.signature.is_none()
                    && !exported.contains(&ComponentId(*index))
            })
            .map(|(_, component)| (component.name.clone(), location(component.position)))
            .collect();
        for (name, at) in untyped {
            self.report(
                id,
                Code::W105,
                at,
                format!("`{name}` has no signature fence, so its params go through unchecked"),
            );
        }
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

/// Where a node starts.
fn location(position: Option<Position>) -> Location {
    position.map_or(Location::new(1, 1), |position| position.start.into())
}
