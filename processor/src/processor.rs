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

use crate::context::RenderContext;
use crate::diagnostic::Diagnostic;
use crate::expanded::{self, Inputs};
use crate::host::{Data, Fetch, Files, Needed, Packs};
use crate::load;
use crate::parse::{self, Change, ParseOptions, TextEdit};
pub use crate::resolve::HostComponent;
use crate::resolve::{self, Ambient, Store};
use crate::tree::Move;
use crate::{expand, tree, validate};
use std::rc::Rc;

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
    /// The parses of the documents resolving loaded, which it reuses while their text stays the same.
    parses: resolve::Parses,
}

/// After stage 3: the checked documents, and what checking refused.
pub struct Validated {
    store: HostFiles,
    resolved: resolve::Resolved,
    checks: validate::Validated,
    /// How many diagnostics stages 1 to 3 gave, which a reload keeps.
    kept: usize,
    edit_context: EditContext,
    allowed_origins: Vec<String>,
}

/// What stage 4 runs with: the render context, and the host's remote and `host:` data.
struct RenderInputs {
    context: RenderContext,
    fetch: Rc<dyn Fetch>,
    data: Rc<dyn Data>,
}

impl RenderInputs {
    fn new(context: RenderContext, fetch: impl Fetch + 'static, data: impl Data + 'static) -> Self {
        Self {
            context,
            fetch: Rc::new(fetch),
            data: Rc::new(data),
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
        self.then(|New { path, text }, diagnostics| {
            let document = parse::parse(&text, options);
            Parsed::new(path, text, document, options.clone(), diagnostics)
        })
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
        let store = HostFiles {
            files: Rc::new(files),
            packs: Rc::new(packs),
        };
        self.resolve_in(store, ambient.clone(), resolve::Parses::default())
    }

    fn resolve_in(
        self,
        store: HostFiles,
        ambient: Ambient,
        mut parses: resolve::Parses,
    ) -> Processor<Resolved> {
        self.then(|parsed, diagnostics| {
            let Parsed {
                path,
                text,
                document,
                options,
            } = parsed;
            let resolved = resolve::resolve(
                &path,
                text,
                document.for_resolving(),
                store.store(),
                &ambient,
                &mut parses,
            );
            let edit_context = EditContext {
                path,
                options,
                parsed: document,
                ambient,
                parses,
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
    /// Stage 3, validating, with the origins remote data may come from (`https://api.example.com`; any other is
    /// F028): checks every invocation, slot, interpolation path, expression, bound input and data source.
    #[must_use]
    pub fn validate(self, allowed_origins: &[String]) -> Processor<Validated> {
        self.then(
            |Resolved {
                 store,
                 resolved,
                 edit_context,
             },
             diagnostics| {
                let checks = validate::validate(&resolved, allowed_origins);
                diagnostics.extend(checks.diagnostics().cloned());
                Validated {
                    store,
                    resolved,
                    checks,
                    kept: diagnostics.len(),
                    edit_context,
                    allowed_origins: allowed_origins.to_vec(),
                }
            },
        )
    }
}

impl Processor<Validated> {
    /// The remote and `host:` sources a render with `context` needs, each once, in the order they are first written,
    /// for the host to fetch before it loads.
    #[must_use]
    pub fn sources(&self, context: &RenderContext) -> Needed {
        load::sources(&self.state.resolved, &self.state.checks, context)
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
        self.then(|validated, diagnostics| {
            let host = load::Host {
                store: validated.store.store(),
                fetch: &*render.fetch,
                data: &*render.data,
            };
            let data = load::load(
                &validated.resolved,
                &validated.checks,
                &render.context,
                &host,
            );
            diagnostics.extend_from_slice(&data.diagnostics);
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
        self.then(
            |Loaded {
                 validated,
                 render,
                 data,
             },
             diagnostics| {
                let result = expand::expand(
                    &validated.resolved,
                    &validated.checks,
                    &data,
                    &render.context,
                    options,
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

impl Processor<Expanded> {
    /// What the processed document's frontmatter gives the render: its own fields, its settings, each data import
    /// with what was loaded, and its inputs.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::expand::Options;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::collections::HashMap;
    /// use std::rc::Rc;
    ///
    /// let text = "---\ntitle: Team\nmd:\n  data:\n    - from: ./team.csv\n      as: team\n---\n";
    /// let files = HashMap::from([("team.csv".to_string(), "name\nAna\n".to_string())]);
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve(files, (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Options::default());
    /// let frontmatter = processor.frontmatter();
    /// assert_eq!(frontmatter.doc["title"], "Team");
    /// assert_eq!(frontmatter.data[0].value, serde_json::json!([{ "name": "Ana" }]));
    /// ```
    #[must_use]
    pub fn frontmatter(&self) -> crate::frontmatter::Frontmatter {
        let resolved = &self.state.validated.resolved;
        crate::frontmatter::of(resolved, resolved.main, &self.state.data)
    }

    /// The components the processed document can name, grouped by the file that defines them, each with where the
    /// document uses it; and the names it uses that stand for none.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::expand::Options;
    /// use compmd::{Ambient, ParseOptions, Processor};
    /// use std::collections::HashMap;
    /// use std::rc::Rc;
    ///
    /// let text = "```compmd component=note\n---\nA note.\n```\n\n:: note[]\n";
    /// let files: HashMap<String, String> = HashMap::new();
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve(files, (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Options::default());
    /// let components = processor.components();
    /// let here = &components.groups[0];
    /// assert_eq!((here.file.as_deref(), here.components[0].name.as_str()), (None, "note"));
    /// assert_eq!(here.components[0].uses[0].start.line, 6);
    /// ```
    #[must_use]
    pub fn components(&self) -> crate::components::Components {
        crate::components::of(&self.state.validated.resolved)
    }

    /// The processed document's tree (spec §12.1).
    #[must_use]
    pub fn tree(&self) -> &tree::Root {
        let resolved = &self.state.validated.resolved;
        &resolved.document(resolved.main).root
    }

    /// The expanded tree (spec §12.3).
    #[must_use]
    pub fn expanded(&self) -> &expanded::Root {
        &self.state.result.root
    }

    /// What the last edit changed in the expanded tree; after the first expansion, all of it.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::expand::Options;
    /// use compmd::{Ambient, ParseOptions, Processor, TextEdit};
    /// use std::collections::HashMap;
    /// use std::rc::Rc;
    ///
    /// let files: HashMap<String, String> = HashMap::new();
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", "One.\n\nTwo.\n\nThree.\n")
    ///     .parse(&ParseOptions::default())
    ///     .resolve(files, (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Options::default());
    /// assert_eq!(processor.splice().nodes.len(), 3);
    ///
    /// // Type a new line into `Two.`: the paragraph is replaced, and `Three.` moves a line down.
    /// let processor = processor.edit(&TextEdit { start: 9, end: 9, text: "\n2".into() });
    /// let splice = processor.splice();
    /// assert_eq!((splice.from, splice.to, splice.nodes.len()), (1, 2, 1));
    /// assert_eq!((splice.lines, splice.offset), (1, 2));
    /// ```
    #[must_use]
    pub fn splice(&self) -> Splice {
        let replaced = &self.state.replaced;
        let from = replaced.nodes.start;
        Splice {
            from,
            to: replaced.nodes.end,
            nodes: self.state.result.root.children[from..from + replaced.inserted].to_vec(),
            lines: replaced.by.lines,
            offset: replaced.by.offset,
        }
    }

    /// Loads and expands again for a render with `context`, `fetch` and `data`, with the same options, without
    /// running stages 1 to 3 again.
    #[must_use]
    pub fn reload(
        self,
        context: RenderContext,
        fetch: impl Fetch + 'static,
        data: impl Data + 'static,
    ) -> Processor<Expanded> {
        let options = self.state.options.clone();
        let before = self.state.result.root.children.len();
        let mut processor = self
            .then(|expanded, diagnostics| back(expanded.validated, diagnostics))
            .load(context, fetch, data)
            .expand(&options);
        processor.state.replaced.nodes = 0..before;
        processor
    }

    /// The host-language components the expanded tree can name, with their code, by name: what a renderer that
    /// runs them needs, since the tree names a component but doesn't hold its code (spec §7.4, §12.3). Only those
    /// one of the processor's bindings runs (others render their fallback, W102); a name two documents give
    /// different components is the processed document's, or else the first loaded.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::{Ambient, Language, ParseOptions, Processor};
    /// use std::rc::Rc;
    ///
    /// let text = "```jsx component=hello runtime=react@19\nexport default () => <b>Hi</b>;\n```\n\n:: hello[Hi]\n";
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", text)
    ///     .parse(&ParseOptions::default())
    ///     .resolve((), (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Default::default());
    /// let [hello] = processor.host_components().try_into().unwrap();
    /// assert_eq!((hello.name.as_str(), hello.language), ("hello", Language::Jsx));
    /// assert_eq!(hello.code, "export default () => <b>Hi</b>;");
    /// ```
    #[must_use]
    pub fn host_components(&self) -> Vec<HostComponent> {
        let validated = &self.state.validated;
        validated.resolved.host_components(validated.store.store())
    }

    /// The remote and `host:` sources a render needs with the context the processor keeps, as
    /// [`sources`](Processor#method.sources) gives them before loading. After an edit to the frontmatter, a host that
    /// fetches ahead fetches what it hasn't yet and calls [`reload`](Processor#method.reload-1).
    #[must_use]
    pub fn sources(&self) -> Needed {
        let validated = &self.state.validated;
        load::sources(
            &validated.resolved,
            &validated.checks,
            &self.state.render.context,
        )
    }

    /// The same processor after `edit` of its text: the trees and diagnostics a processor of the new text gives,
    /// with everything it was given before, the parse options, the host's files, packs and data, the ambient packs,
    /// the allowed origins, the render context and the expansion options.
    ///
    /// Parsing only parses again the blocks the edit touches and one block either side, and moves the blocks after
    /// them; an edit to the link and footnote definitions parses the whole text again. The later stages then only
    /// run on the top-level blocks whose tree changed, and move what the others gave. An edit to the frontmatter or to
    /// a component definition or signature fence runs them on the whole document again; an edit that changes which
    /// data imports may load loads again.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::expand::Options;
    /// use compmd::{Ambient, ParseOptions, Processor, TextEdit};
    /// use std::collections::HashMap;
    /// use std::rc::Rc;
    ///
    /// let files: HashMap<String, String> = HashMap::new();
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: Rc::new(()), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", "Hello world\n")
    ///     .parse(&ParseOptions::default())
    ///     .resolve(files, (), &Ambient::default())
    ///     .validate(&[])
    ///     .load(context, (), ())
    ///     .expand(&Options::default());
    ///
    /// // Select `world` and type `there`.
    /// let processor = processor.edit(&TextEdit { start: 6, end: 11, text: "there".into() });
    /// let json = serde_json::to_value(processor.expanded()).unwrap();
    /// assert_eq!(json["children"][0]["children"][0]["value"], "Hello there");
    /// ```
    #[must_use]
    pub fn edit(self, edit: &TextEdit) -> Self {
        let Expanded {
            validated,
            render,
            data,
            options,
            result,
            ..
        } = self.state;
        let Validated {
            store,
            mut resolved,
            mut checks,
            edit_context: context,
            allowed_origins,
            ..
        } = validated;
        let before = result.root.children.len();
        let (text, mut lines) = resolved.take_text();
        let (text, document, change) =
            parse::reparse(context.parsed, &text, &mut lines, edit, &context.options);
        let Change::Blocks(spliced) = change else {
            drop(resolved);
            let mut diagnostics = Vec::new();
            let parsed = Parsed::new(
                context.path,
                text,
                document,
                context.options,
                &mut diagnostics,
            );
            let mut processor = Processor {
                diagnostics,
                state: parsed,
            }
            .resolve_in(store, context.ambient, context.parses)
            .validate(&allowed_origins)
            .load_with(render)
            .expand(&options);
            processor.state.replaced.nodes = 0..before;
            return processor;
        };
        let report = context.ambient.report;
        resolved.splice((text, lines), Rc::clone(&document.root), &spliced, report);
        let reload = checks.splice(&resolved, &spliced, &allowed_origins);
        let mut diagnostics = document.diagnostics.clone();
        diagnostics.extend(resolved.diagnostics().cloned());
        diagnostics.extend(checks.diagnostics().cloned());
        let validated = Validated {
            store,
            resolved,
            checks,
            kept: diagnostics.len(),
            edit_context: EditContext {
                parsed: document,
                ..context
            },
            allowed_origins,
        };
        if reload {
            let mut processor = Processor {
                diagnostics,
                state: validated,
            }
            .load_with(render)
            .expand(&options);
            processor.state.replaced.nodes = 0..before;
            return processor;
        }
        let result = expand::splice(
            result,
            &validated.resolved,
            &validated.checks,
            &data,
            &render.context,
            &options,
            &spliced,
        );
        diagnostics.extend_from_slice(&data.diagnostics);
        diagnostics.extend_from_slice(&result.diagnostics);
        let replaced = result.replaced.clone().unwrap_or(expand::Replaced {
            nodes: 0..before,
            inserted: result.root.children.len(),
            by: Move::default(),
        });
        Processor {
            diagnostics,
            state: Expanded {
                validated,
                render,
                data,
                options,
                result,
                replaced,
            },
        }
    }
}

/// The validated state again, with the diagnostics of stage 4 dropped.
fn back(validated: Validated, diagnostics: &mut Vec<Diagnostic>) -> Validated {
    diagnostics.truncate(validated.kept);
    validated
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, json};
    use std::collections::HashMap;

    fn context(inputs: serde_json::Value) -> RenderContext {
        let inputs: Map<String, serde_json::Value> =
            serde_json::from_value(inputs).expect("an object");
        RenderContext {
            now: 0,
            tz: "UTC".into(),
            time: Rc::new(()),
            inputs,
        }
    }

    #[test]
    fn each_state_gives_what_its_stage_made_and_reloads_keep_stages_1_to_3() {
        let text = "---\nmd:\n  data:\n    - from: ./missing.json\n      as: m\n  inputs:\n    n: number = 1\n---\n\n{{md.input.n}} :x[y]\n";
        let files: HashMap<String, String> = HashMap::new();
        let parsed = Processor::new("a.comp.md", text).parse(&ParseOptions::default());
        assert_eq!(parsed.tree().children.len(), 2);
        let resolved = parsed.resolve(files, (), &Ambient::default());
        assert_eq!(
            resolved.state.resolved.documents.len(),
            2,
            "with @compmd/core"
        );
        let validated = resolved.validate(&[]);
        assert_eq!(validated.diagnostics().len(), 1, "F002 for `x`");
        let one = context(json!({}));
        let loaded = validated.load(one, (), ());
        let values = |loaded: &Processor<Loaded>| -> Vec<serde_json::Value> {
            loaded
                .inputs()
                .iter()
                .map(|input| input.value.clone())
                .collect()
        };
        assert_eq!(values(&loaded), [json!(1)]);
        assert_eq!(loaded.diagnostics().len(), 2, "and F026 for the data");
        let two = context(json!({ "n": 2 }));
        let loaded = loaded.reload(two, (), ());
        assert_eq!(values(&loaded), [json!(2)]);
        assert_eq!(
            loaded.diagnostics().len(),
            2,
            "a reload drops stage 4's own"
        );
        let expanded = loaded.expand(&expand::Options::default());
        let written = |processor: &Processor<Expanded>| {
            serde_json::to_value(processor.expanded()).expect("a tree")["children"][0]["children"]
                [0]["value"]
                .clone()
        };
        assert_eq!(written(&expanded), json!("2 y"));
        assert_eq!(expanded.sources(), Needed::default());
        let three = context(json!({ "n": 3 }));
        let expanded = expanded.reload(three, (), ());
        assert_eq!(written(&expanded), json!("3 y"));
        let splice = expanded.splice();
        assert_eq!((splice.from, splice.to, splice.nodes.len()), (0, 1, 1));
        assert_eq!(expanded.diagnostics().len(), 2);
        assert_eq!(expanded.tree().children.len(), 2);
    }

    #[test]
    fn an_edit_after_text_of_two_blocks_was_joined_expands_the_whole_document() {
        let files: HashMap<String, String> = HashMap::new();
        let context = context(json!({}));
        let mut processor = Processor::new("a.comp.md", "One.\n\nTwo.\n")
            .parse(&ParseOptions::default())
            .resolve(files, (), &Ambient::default())
            .validate(&[])
            .load(context, (), ())
            .expand(&expand::Options::default());
        // As joining text across blocks leaves it.
        processor.state.result.blocks = None;
        let processor = processor.edit(&TextEdit {
            start: 9,
            end: 9,
            text: "!".into(),
        });
        let splice = processor.splice();
        assert_eq!((splice.from, splice.to, splice.nodes.len()), (0, 2, 2));
        let json = serde_json::to_value(processor.expanded()).expect("a tree");
        assert_eq!(json["children"][1]["children"][0]["value"], "Two!.");
    }
}
