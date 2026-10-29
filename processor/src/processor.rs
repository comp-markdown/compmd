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
//!
//! let text = "---\nteam: [Ana, Kenji]\n---\n\n::: each {in=(md.doc.team) as=p join=inline sep=\", \"}\n{{p}}\n::: /each\n";
//! let files: HashMap<String, String> = HashMap::new();
//! let context = RenderContext { now: 0, tz: "UTC".into(), time: &(), inputs: Default::default() };
//! let processor = Processor::new("team.comp.md", text)
//!     .parse(&ParseOptions::default())
//!     .resolve(&files, &(), &Ambient::default())
//!     .validate(&[])
//!     .load(&context, &(), &())
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
use crate::parse::{self, ParseOptions, TextEdit};
use crate::resolve::{self, Ambient, Store};
use crate::{expand, tree, validate};

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
pub struct Resolved<'h> {
    store: Store<'h>,
    resolved: resolve::Resolved,
    edit_context: EditContext,
}

/// What an edit needs to run stages 1 and 2 again: what stage 1 was given and made, and the ambient packs.
struct EditContext {
    path: String,
    text: String,
    options: ParseOptions,
    /// The parse of `text`, whose tree the resolved document shares.
    parsed: parse::Parsed,
    ambient: Ambient,
    /// The parses of the documents resolving loaded, which it reuses while their text stays the same.
    parses: resolve::Parses,
}

/// After stage 3: the checked documents, and what checking refused.
pub struct Validated<'h> {
    store: Store<'h>,
    resolved: resolve::Resolved,
    checks: validate::Validated,
    /// How many diagnostics stages 1 to 3 gave, which a reload keeps.
    kept: usize,
    edit_context: EditContext,
    allowed_origins: Vec<String>,
}

/// What stage 4 runs with: the render context, and the host's remote and `host:` data.
#[derive(Clone, Copy)]
struct RenderInputs<'c> {
    context: &'c RenderContext<'c>,
    fetch: &'c dyn Fetch,
    data: &'c dyn Data,
}

/// After stage 4a: the inputs' values and the data, for one render context.
pub struct Loaded<'h, 'c> {
    validated: Validated<'h>,
    render: RenderInputs<'c>,
    data: load::Loaded,
}

/// After stage 4b: the expanded tree.
pub struct Expanded<'h, 'c> {
    validated: Validated<'h>,
    render: RenderInputs<'c>,
    options: expand::Options,
    root: expanded::Root,
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
    /// binds every name.
    #[must_use]
    pub fn resolve<'h>(
        self,
        files: &'h impl Files,
        packs: &'h impl Packs,
        ambient: &Ambient,
    ) -> Processor<Resolved<'h>> {
        self.resolve_in(
            Store { files, packs },
            ambient.clone(),
            resolve::Parses::default(),
        )
    }

    fn resolve_in(
        self,
        store: Store<'_>,
        ambient: Ambient,
        mut parses: resolve::Parses,
    ) -> Processor<Resolved<'_>> {
        self.then(|parsed, diagnostics| {
            let Parsed {
                path,
                text,
                document,
                options,
            } = parsed;
            let mut resolved = resolve::resolve(
                &path,
                &text,
                document.for_resolving(),
                store,
                &ambient,
                &mut parses,
            );
            let edit_context = EditContext {
                path,
                text,
                options,
                parsed: document,
                ambient,
                parses,
            };
            diagnostics.append(&mut resolved.diagnostics);
            Resolved {
                store,
                resolved,
                edit_context,
            }
        })
    }
}

impl<'h> Processor<Resolved<'h>> {
    /// Stage 3, validating, with the origins remote data may come from (`https://api.example.com`; any other is
    /// F028): checks every invocation, slot, interpolation path, expression, bound input and data source.
    #[must_use]
    pub fn validate(self, allowed_origins: &[String]) -> Processor<Validated<'h>> {
        self.then(
            |Resolved {
                 store,
                 resolved,
                 edit_context,
             },
             diagnostics| {
                let mut checks = validate::validate(&resolved, allowed_origins);
                diagnostics.append(&mut checks.diagnostics);
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

impl<'h> Processor<Validated<'h>> {
    /// The remote and `host:` sources a render with `context` needs, each once, in the order they are first written,
    /// for the host to fetch before it loads.
    #[must_use]
    pub fn sources(&self, context: &RenderContext) -> Needed {
        load::sources(&self.state.resolved, &self.state.checks, context)
    }

    /// Stage 4a, loading, for a render with `context`: the inputs' values and every data import, with the host's
    /// remote data from `fetch` and its `host:` data from `data`. The processor keeps all three for a later edit.
    #[must_use]
    pub fn load<'c>(
        self,
        context: &'c RenderContext<'c>,
        fetch: &'c impl Fetch,
        data: &'c impl Data,
    ) -> Processor<Loaded<'h, 'c>> {
        self.load_with(RenderInputs {
            context,
            fetch,
            data,
        })
    }

    fn load_with(self, render: RenderInputs<'_>) -> Processor<Loaded<'h, '_>> {
        self.then(|validated, diagnostics| {
            let host = load::Host {
                store: validated.store,
                fetch: render.fetch,
                data: render.data,
            };
            let mut data = load::load(
                &validated.resolved,
                &validated.checks,
                render.context,
                &host,
            );
            diagnostics.append(&mut data.diagnostics);
            Loaded {
                validated,
                render,
                data,
            }
        })
    }
}

impl<'h, 'c> Processor<Loaded<'h, 'c>> {
    /// The processed document's inputs, with the values the render uses.
    #[must_use]
    pub fn inputs(&self) -> &Inputs {
        &self.state.data.inputs
    }

    /// Stage 4b, evaluating: the expanded tree.
    #[must_use]
    pub fn expand(self, options: &expand::Options) -> Processor<Expanded<'h, 'c>> {
        self.then(
            |Loaded {
                 validated,
                 render,
                 data,
             },
             diagnostics| {
                let expanded = expand::expand(
                    &validated.resolved,
                    &validated.checks,
                    &data,
                    render.context,
                    options,
                );
                diagnostics.extend(expanded.diagnostics);
                Expanded {
                    validated,
                    render,
                    options: options.clone(),
                    root: expanded.root,
                }
            },
        )
    }

    /// Loads again for a render with `context`, `fetch` and `data`, without running stages 1 to 3 again.
    #[must_use]
    pub fn reload<'d>(
        self,
        context: &'d RenderContext<'d>,
        fetch: &'d impl Fetch,
        data: &'d impl Data,
    ) -> Processor<Loaded<'h, 'd>> {
        self.then(|loaded, diagnostics| back(loaded.validated, diagnostics))
            .load(context, fetch, data)
    }
}

impl<'h> Processor<Expanded<'h, '_>> {
    /// The processed document's tree (spec §12.1).
    #[must_use]
    pub fn tree(&self) -> &tree::Root {
        let resolved = &self.state.validated.resolved;
        &resolved.document(resolved.main).root
    }

    /// The expanded tree (spec §12.3).
    #[must_use]
    pub fn expanded(&self) -> &expanded::Root {
        &self.state.root
    }

    /// Loads and expands again for a render with `context`, `fetch` and `data`, with the same options, without
    /// running stages 1 to 3 again.
    #[must_use]
    pub fn reload<'d>(
        self,
        context: &'d RenderContext<'d>,
        fetch: &'d impl Fetch,
        data: &'d impl Data,
    ) -> Processor<Expanded<'h, 'd>> {
        let options = self.state.options.clone();
        self.then(|expanded, diagnostics| back(expanded.validated, diagnostics))
            .load(context, fetch, data)
            .expand(&options)
    }

    /// The same processor after `edit` of its text: the trees and diagnostics a processor of the new text gives,
    /// with everything it was given before, the parse options, the host's files, packs and data, the ambient packs,
    /// the allowed origins, the render context and the expansion options.
    ///
    /// Parsing only parses again the blocks the edit touches and one block either side, and moves the blocks after
    /// them; an edit to the frontmatter or to the link and footnote definitions parses the whole text again.
    ///
    /// ```
    /// use compmd::context::RenderContext;
    /// use compmd::expand::Options;
    /// use compmd::{Ambient, ParseOptions, Processor, TextEdit};
    /// use std::collections::HashMap;
    ///
    /// let files: HashMap<String, String> = HashMap::new();
    /// let context = RenderContext { now: 0, tz: "UTC".into(), time: &(), inputs: Default::default() };
    /// let processor = Processor::new("a.comp.md", "Hello world\n")
    ///     .parse(&ParseOptions::default())
    ///     .resolve(&files, &(), &Ambient::default())
    ///     .validate(&[])
    ///     .load(&context, &(), &())
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
            options,
            ..
        } = self.state;
        let Validated {
            store,
            resolved,
            edit_context: context,
            allowed_origins,
            ..
        } = validated;
        // Once the resolved documents are gone, the parse holds the only reference to the tree and changes it in
        // place.
        drop(resolved);
        let (text, document) =
            parse::reparse(context.parsed, &context.text, edit, &context.options);
        let mut diagnostics = Vec::new();
        let parsed = Parsed::new(
            context.path,
            text,
            document,
            context.options,
            &mut diagnostics,
        );
        Processor {
            diagnostics,
            state: parsed,
        }
        .resolve_in(store, context.ambient, context.parses)
        .validate(&allowed_origins)
        .load_with(render)
        .expand(&options)
    }
}

/// The validated state again, with the diagnostics of stage 4 dropped.
fn back<'h>(validated: Validated<'h>, diagnostics: &mut Vec<Diagnostic>) -> Validated<'h> {
    diagnostics.truncate(validated.kept);
    validated
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Map, json};
    use std::collections::HashMap;

    fn context(inputs: serde_json::Value) -> RenderContext<'static> {
        let inputs: Map<String, serde_json::Value> =
            serde_json::from_value(inputs).expect("an object");
        RenderContext {
            now: 0,
            tz: "UTC".into(),
            time: &(),
            inputs,
        }
    }

    #[test]
    fn each_state_gives_what_its_stage_made_and_reloads_keep_stages_1_to_3() {
        let text = "---\nmd:\n  data:\n    - from: ./missing.json\n      as: m\n  inputs:\n    n: number = 1\n---\n\n{{md.input.n}} :x[y]\n";
        let files: HashMap<String, String> = HashMap::new();
        let parsed = Processor::new("a.comp.md", text).parse(&ParseOptions::default());
        assert_eq!(parsed.tree().children.len(), 2);
        let resolved = parsed.resolve(&files, &(), &Ambient::default());
        assert_eq!(
            resolved.state.resolved.documents.len(),
            2,
            "with @compmd/core"
        );
        let validated = resolved.validate(&[]);
        assert_eq!(validated.diagnostics().len(), 1, "F002 for `x`");
        let one = context(json!({}));
        let loaded = validated.load(&one, &(), &());
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
        let loaded = loaded.reload(&two, &(), &());
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
        let three = context(json!({ "n": 3 }));
        let expanded = expanded.reload(&three, &(), &());
        assert_eq!(written(&expanded), json!("3 y"));
        assert_eq!(expanded.diagnostics().len(), 2);
        assert_eq!(expanded.tree().children.len(), 2);
    }
}
