//! The processor (spec §11), its stages as states: a [`Processor`] goes from [`New`] through [`Parsed`],
//! [`Resolved`], [`Validated`] and [`Loaded`] to [`Expanded`]. Each stage takes the processor and gives it back in its
//! next state, so the stages can only run in order; each state gives what its stage made, and every state the
//! diagnostics so far. Each stage takes what it uses: the parse options; the host's files and packs, and the ambient
//! packs; the origins remote data may come from; the render context, and the host's remote and `host:` data; the
//! expansion options.
//!
//! The expanded processor keeps all of them, so an edit of the text takes only the edit
//! ([`Processor::edit`](Processor#method.edit-1)).
//!
//! ```
//! use compmd::context::RenderContext;
//! use compmd::expand::Options;
//! use compmd::{Ambient, ParseOptions, Processor};
//! use std::collections::HashMap;
//! use std::rc::Rc;
//!
//! let text = "---\nteam: [Ana, Kenji]\n---\n\n::: each {in=(md.doc.team) as=p join=inline sep=\", \"}\n{{p}}\n::: /each\n";
//! let files: HashMap<String, String> = HashMap::new();
//! let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
//! let processor = Processor::new("team.comp.md", text)
//!     .parse(&ParseOptions::default())
//!     .resolve(files, (), &Ambient::default())
//!     .validate(&[])
//!     .load(context, (), ())
//!     .expand(&Options::default());
//! assert!(processor.diagnostics().is_empty());
//! let json = serde_json::to_value(processor.expanded()).unwrap();
//! assert_eq!(json["children"][0]["children"][0]["value"], "Ana, Kenji");
//! ```
//!
//! Parsing alone is the first two calls:
//!
//! ```
//! use compmd::{ParseOptions, Processor};
//! use compmd::tree::Node;
//!
//! let parsed = Processor::new("a.comp.md", "::: card {title=\"Setup\"}\nInstall it.\n::: /card\n")
//!     .parse(&ParseOptions::default());
//! assert!(parsed.diagnostics().is_empty());
//! let Node::ComponentContainer(card) = &parsed.tree().children[0] else {
//!     panic!("a container");
//! };
//! assert_eq!(card.name, "card");
//! ```

use crate::AllowedOrigins;
use crate::context::RenderContext;
use crate::diagnostic::Diagnostic;
use crate::expanded::{self, Inputs};
use crate::host::{Data, Fetch, Files, Needed, Packs};
use crate::load;
use crate::parse::{self, ParseOptions};
use crate::policy::Policy;
use crate::resolve::{self, Ambient, Store};
pub use crate::resolve::{HostComponent, LibraryModule, Module};
use crate::tree::Move;
use crate::{expand, tree, validate};
use std::collections::HashMap;
use std::rc::Rc;

mod edit;
mod evaluate;
mod examples;
mod outputs;

/// A document on its way through the stages, in the state `S`.
pub struct Processor<S> {
    diagnostics: Vec<Diagnostic>,
    state: S,
}

/// A document not yet parsed.
pub struct New {
    path: String,
    text: String,
}

/// After stage 1: the document's tree and settings.
pub struct Parsed {
    path: String,
    text: String,
    /// The tree and settings; the processor holds a copy of its diagnostics.
    document: parse::Parsed,
    options: ParseOptions,
    /// Whether the document is an example (spec §7.8), whose own ` ```compmd ` blocks show their source (W123).
    example: bool,
}

/// After stage 2: the document, everything it imports, and what each name stands for.
#[allow(
    clippy::struct_field_names,
    reason = "`resolved` is stage 2's result, named as in the later states"
)]
pub struct Resolved {
    store: HostFiles,
    resolved: resolve::Resolved,
    edit_context: EditContext,
}

/// The host's files and packs, which stages 2 and 4a read.
struct HostFiles {
    files: Rc<dyn Files>,
    packs: Rc<dyn Packs>,
}

impl HostFiles {
    fn store(&self) -> Store<'_> {
        Store {
            files: &*self.files,
            packs: &*self.packs,
        }
    }
}

/// What an edit needs to run stages 1 and 2 again: what stage 1 was given and made, and the ambient packs. The text
/// is the resolved document's.
struct EditContext {
    path: String,
    options: ParseOptions,
    /// The parse of the text, whose tree the resolved document shares.
    parsed: parse::Parsed,
    ambient: Ambient,
    policy: Policy,
    /// The parses of the documents resolving loaded, which it reuses while their text stays the same.
    parses: resolve::Parses,
    /// Whether the document is an example (spec §7.8).
    example: bool,
}

/// After stage 3: the checked documents, and what checking refused.
pub struct Validated {
    store: HostFiles,
    resolved: resolve::Resolved,
    checks: validate::Validated,
    /// The processed document's examples (spec §7.8), each processed beside it, at its stage.
    examples: examples::Examples,
    /// How many diagnostics stages 1 to 3 gave, which a reload keeps.
    kept: usize,
    edit_context: EditContext,
    allowed_origins: AllowedOrigins,
}

/// What stage 4 runs with: the render context, and the host's remote and `host:` data.
struct RenderInputs {
    context: RenderContext,
    fetch: Rc<dyn Fetch>,
    data: Rc<dyn Data>,
    /// The values given for each example's inputs (spec §7.8), by its number; an example not here has its defaults.
    examples: HashMap<usize, serde_json::Map<String, serde_json::Value>>,
}

impl RenderInputs {
    fn new(context: RenderContext, fetch: impl Fetch + 'static, data: impl Data + 'static) -> Self {
        Self {
            context,
            fetch: Rc::new(fetch),
            data: Rc::new(data),
            examples: HashMap::new(),
        }
    }
}

/// After stage 4a: the inputs' values and the data, for one render context.
pub struct Loaded {
    validated: Validated,
    render: RenderInputs,
    data: load::Loaded,
}

/// After stage 4b: the expanded tree.
pub struct Expanded {
    validated: Validated,
    render: RenderInputs,
    /// What stage 4a loaded, which an edit of the body keeps.
    data: load::Loaded,
    options: expand::Options,
    /// Stage 4b's result.
    result: expand::Expanded,
    /// What the expansion replaced in the expanded tree before it.
    replaced: expand::Replaced,
}

/// What an edit changed in the expanded tree: its top-level nodes `from..to` became `nodes`, and every node after
/// them moved by `lines` lines and `offset` code points, which changes their origins (spec §12.3), if they have any.
/// A host that keeps the expanded tree elsewhere, across a worker, WebAssembly or a process, applies it to its copy
/// instead of taking the whole tree again.
///
/// After the first expansion it replaces an empty tree: `from` and `to` are 0, and `nodes` is the whole tree. After a
/// reload, it replaces the whole tree before it.
#[derive(Clone, Debug, PartialEq)]
pub struct Splice {
    /// The first top-level node replaced.
    pub from: usize,
    /// The end of the top-level nodes replaced, in the tree before the edit.
    pub to: usize,
    /// The nodes that replace them.
    pub nodes: Vec<expanded::Node>,
    /// The lines the nodes after them moved by.
    pub lines: isize,
    /// The code points the nodes after them moved by.
    pub offset: isize,
}

impl<S> Processor<S> {
    /// The diagnostics of every stage run so far, in stage order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// The next state, which the stage `name` (`parse`, `resolve`, …) makes from this one with `step`, in a `tracing`
    /// span of its name, and an event with the diagnostics it found, for a host that installs a subscriber.
    fn stage<T>(
        self,
        name: &'static str,
        step: impl FnOnce(S, &mut Vec<Diagnostic>) -> T,
    ) -> Processor<T> {
        let _span = tracing::debug_span!("compmd", stage = name).entered();
        let before = self.diagnostics.len();
        let next = self.then(step);
        tracing::debug!(diagnostics = next.diagnostics.len() - before, "stage done");
        next
    }

    /// The next state, which `step` makes from this one, adding the diagnostics it finds.
    fn then<T>(self, step: impl FnOnce(S, &mut Vec<Diagnostic>) -> T) -> Processor<T> {
        let Processor {
            mut diagnostics,
            state,
        } = self;
        let state = step(state, &mut diagnostics);
        Processor { diagnostics, state }
    }
}

impl Processor<New> {
    /// A processor for the document at `path` under its root folder, with `text`.
    ///
    /// `path` is relative to the root folder, its names separated by `/`: `posts/harbour.comp.md`. Every name is a
    /// file's or a folder's name, never empty, `.` or `..`, and has no `\`, `:` or NUL. A document at any other path
    /// (`./a.comp.md`, `/docs/a.comp.md`, `C:\docs\a.comp.md`, an empty one) is still processed, but has no place in
    /// its root: each relative path it writes, an import, data or a font's file, is F026.
    #[must_use]
    pub fn new(path: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            diagnostics: Vec::new(),
            state: New {
                path: path.into(),
                text: text.into(),
            },
        }
    }

    /// Stage 1, parsing: the document's tree (spec §12.1). Parsing never fails: a mistake is a diagnostic, and the
    /// recovery rules of spec §10.3 decide the tree.
    #[must_use]
    pub fn parse(self, options: &ParseOptions) -> Processor<Parsed> {
        self.stage("parse", |New { path, text }, diagnostics| {
            let document = parse::parse(&text, options);
            Parsed::new(path, text, document, options.clone(), diagnostics)
        })
    }

    /// Stage 1 for an example (spec §7.8), whose text is a ` ```compmd ` block's: as [`Self::parse`], for a document
    /// that makes no examples of its own.
    fn parse_example(self, options: &ParseOptions) -> Processor<Parsed> {
        let mut parsed = self.parse(options);
        parsed.state.example = true;
        parsed
    }
}

impl Parsed {
    /// The state after parsing `text` into `document`, whose diagnostics go to `diagnostics` too.
    fn new(
        path: String,
        text: String,
        document: parse::Parsed,
        options: ParseOptions,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Self {
        diagnostics.extend_from_slice(&document.diagnostics);
        Self {
            path,
            text,
            document,
            options,
            example: false,
        }
    }
}

impl Processor<Parsed> {
    /// The document's tree.
    #[must_use]
    pub fn tree(&self) -> &tree::Root {
        &self.state.document.root
    }

    /// Stage 2, resolving, with the host's `files` and `packs`, which loading reads data files from too, and the
    /// `ambient` packs: loads the document's imports and their imports, `@compmd/core` and the ambient packs, and
    /// binds every name. The processor keeps `files` and `packs`; a host that needs them too passes an `Rc` it keeps
    /// a clone of.
    #[must_use]
    pub fn resolve(
        self,
        files: impl Files + 'static,
        packs: impl Packs + 'static,
        ambient: &Ambient,
    ) -> Processor<Resolved> {
        self.resolve_with(files, packs, ambient, &Policy::default())
    }

    /// [`Self::resolve`], with the host's `policy` for packs (spec §6.4, §15): the packs it lists and their trust,
    /// whether a document may import others, and the packs it denies or left out, which are F026.
    #[must_use]
    pub fn resolve_with(
        self,
        files: impl Files + 'static,
        packs: impl Packs + 'static,
        ambient: &Ambient,
        policy: &Policy,
    ) -> Processor<Resolved> {
        let store = HostFiles {
            files: Rc::new(files),
            packs: Rc::new(packs),
        };
        self.resolve_in(
            store,
            ambient.clone(),
            policy.clone(),
            resolve::Parses::default(),
        )
    }

    fn resolve_in(
        self,
        store: HostFiles,
        ambient: Ambient,
        policy: Policy,
        mut parses: resolve::Parses,
    ) -> Processor<Resolved> {
        self.stage("resolve", |parsed, diagnostics| {
            let Parsed {
                path,
                text,
                document,
                options,
                example,
            } = parsed;
            let resolved = resolve::resolve(
                &path,
                text,
                document.for_resolving(),
                store.store(),
                resolve::Host {
                    ambient: &ambient,
                    policy: &policy,
                },
                &mut parses,
            );
            let edit_context = EditContext {
                path,
                options,
                parsed: document,
                ambient,
                policy,
                parses,
                example,
            };
            diagnostics.extend(resolved.diagnostics().cloned());
            Resolved {
                store,
                resolved,
                edit_context,
            }
        })
    }
}

impl Processor<Resolved> {
    /// Stage 3, validating, with the origins remote data may come from: a list of them (`&["https://api.example.com"
    /// .into()]`; any other is F028), or [`AllowedOrigins::Every`] for a host's own documents (spec §15). Checks every
    /// invocation, slot, interpolation path, expression, bound input and data source.
    #[must_use]
    pub fn validate(self, allowed_origins: impl Into<AllowedOrigins>) -> Processor<Validated> {
        let allowed_origins = allowed_origins.into();
        self.stage(
            "validate",
            |Resolved {
                 store,
                 resolved,
                 edit_context,
             },
             diagnostics| {
                let checks = validate::validate(&resolved, &allowed_origins);
                diagnostics.extend(checks.diagnostics().cloned());
                let examples = examples::Examples::of(
                    &resolved,
                    &store,
                    &edit_context,
                    &allowed_origins,
                    examples::Examples::default(),
                    diagnostics,
                );
                Validated {
                    store,
                    resolved,
                    checks,
                    examples,
                    kept: diagnostics.len(),
                    edit_context,
                    allowed_origins,
                }
            },
        )
    }
}

impl Processor<Validated> {
    /// The remote and `host:` sources a render with `context` needs, each once, in the order they are first written,
    /// for the host to fetch before it loads; and the packs the document and the packs it loaded import that the host
    /// gave no matching release of, each with its range and pinned `integrity`, then the library packs they name
    /// that it gave none of, for a host that gets packs from a registry to fetch and resolve again with (spec §6.4).
    /// The processor fetches nothing.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::host::NeededPack;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let text = "---\nmd:\n  import:\n    - from: \"@kit/mermaid@1\"\n      names: [diagram]\n---\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let validated = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[]);
    /// let needed = validated.sources(&context);
    /// assert_eq!(needed.packs, [NeededPack { pack: "@kit/mermaid".parse()?, range: "1".into(), integrity: None, library: false }]);
    /// # Ok::<(), compmd::pack::Invalid>(())
    /// ```
    #[must_use]
    pub fn sources(&self, context: &RenderContext) -> Needed {
        let mut needed = load::sources(
            &self.state.resolved,
            &self.state.checks,
            context,
            &self.state.allowed_origins,
        );
        self.state.examples.sources(context, &mut needed);
        needed
    }

    /// Stage 4a, loading, for a render with `context`: the inputs' values and every data import, with the host's
    /// remote data from `fetch` and its `host:` data from `data`. The processor keeps all three for a later edit.
    #[must_use]
    pub fn load(
        self,
        context: RenderContext,
        fetch: impl Fetch + 'static,
        data: impl Data + 'static,
    ) -> Processor<Loaded> {
        self.load_with(RenderInputs::new(context, fetch, data))
    }

    fn load_with(self, render: RenderInputs) -> Processor<Loaded> {
        self.stage("load", |mut validated, diagnostics| {
            let host = load::Host {
                store: validated.store.store(),
                fetch: &*render.fetch,
                data: &*render.data,
                report: validated.edit_context.ambient.report,
            };
            let data = load::load(
                &validated.resolved,
                &validated.checks,
                &render.context,
                &host,
            );
            diagnostics.extend_from_slice(&data.diagnostics);
            validated.examples.load(&render, diagnostics);
            Loaded {
                validated,
                render,
                data,
            }
        })
    }
}

impl Processor<Loaded> {
    /// The processed document's inputs, with the values the render uses.
    #[must_use]
    pub fn inputs(&self) -> &Inputs {
        &self.state.data.inputs
    }

    /// Stage 4b, evaluating: the expanded tree.
    #[must_use]
    pub fn expand(self, options: &expand::Options) -> Processor<Expanded> {
        self.stage(
            "expand",
            |Loaded {
                 mut validated,
                 render,
                 mut data,
             },
             diagnostics| {
                let (examples, used) = validated.examples.expand(options, diagnostics);
                data.examples = examples;
                let result = expand::expand(
                    &validated.resolved,
                    &validated.checks,
                    &data,
                    &render.context,
                    &options.after(used),
                );
                diagnostics.extend_from_slice(&result.diagnostics);
                let replaced = expand::Replaced {
                    nodes: 0..0,
                    inserted: result.root.children.len(),
                    by: Move::default(),
                };
                Expanded {
                    validated,
                    render,
                    data,
                    options: options.clone(),
                    result,
                    replaced,
                }
            },
        )
    }

    /// Loads again for a render with `context`, `fetch` and `data`, without running stages 1 to 3 again.
    #[must_use]
    pub fn reload(
        self,
        context: RenderContext,
        fetch: impl Fetch + 'static,
        data: impl Data + 'static,
    ) -> Processor<Loaded> {
        self.then(|loaded, diagnostics| back(loaded.validated, diagnostics))
            .load(context, fetch, data)
    }
}

/// The validated state again, with the diagnostics of stage 4 dropped.
fn back(mut validated: Validated, diagnostics: &mut Vec<Diagnostic>) -> Validated {
    diagnostics.truncate(validated.kept);
    validated.examples.back();
    validated
}

#[cfg(test)]
mod tests;
