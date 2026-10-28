//! The processor (spec §11), its stages as states: a [`Processor`] goes from [`New`] through [`Parsed`],
//! [`Resolved`], [`Validated`] and [`Loaded`] to [`Expanded`]. Each stage takes the processor and gives it back in its
//! next state, so the stages can only run in order; each state gives what its stage made, and every state the
//! diagnostics so far. Each stage takes what it uses: the parse options; the host's files and packs, and the ambient
//! packs; the origins remote data may come from; the render context, and the host's remote and `host:` data; the
//! expansion options.
//!
//! ```
//! use compmd::context::Context;
//! use compmd::expand::Options;
//! use compmd::{Ambient, ParseOptions, Processor};
//! use std::collections::HashMap;
//!
//! let text = "---\nteam: [Ana, Kenji]\n---\n\n::: each {in=(md.doc.team) as=p join=inline sep=\", \"}\n{{p}}\n::: /each\n";
//! let files: HashMap<String, String> = HashMap::new();
//! let context = Context { now: 0, tz: "UTC".into(), time: &(), inputs: Default::default() };
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

use crate::context::Context;
use crate::diagnostic::Diagnostic;
use crate::expanded::{self, Inputs};
use crate::host::{Data, Fetch, Files, Packs};
use crate::load::{self, Source};
use crate::parse::{self, ParseOptions};
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

/// After stage 1: the document's tree.
pub struct Parsed {
    path: String,
    text: String,
    root: tree::Root,
}

/// After stage 2: the document, everything it imports, and what each name stands for.
pub struct Resolved<'h> {
    store: Store<'h>,
    resolved: resolve::Resolved,
}

/// After stage 3: the checked documents, and what checking refused.
pub struct Validated<'h> {
    store: Store<'h>,
    resolved: resolve::Resolved,
    checks: validate::Validated,
    /// How many diagnostics stages 1 to 3 gave, which a reload keeps.
    kept: usize,
}

/// After stage 4a: the inputs' values and the data, for one render context.
pub struct Loaded<'h, 'c> {
    validated: Validated<'h>,
    context: &'c Context<'c>,
    data: load::Loaded,
}

/// After stage 4b: the expanded tree.
pub struct Expanded<'h> {
    validated: Validated<'h>,
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
            let parsed = parse::parse(&text, options);
            diagnostics.extend(parsed.diagnostics);
            Parsed {
                path,
                text,
                root: parsed.root,
            }
        })
    }
}

impl Processor<Parsed> {
    /// The document's tree.
    #[must_use]
    pub fn tree(&self) -> &tree::Root {
        &self.state.root
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
        let store = Store { files, packs };
        self.then(|Parsed { path, text, root }, diagnostics| {
            let mut resolved = resolve::resolve(&path, &text, root, store, ambient);
            diagnostics.append(&mut resolved.diagnostics);
            Resolved { store, resolved }
        })
    }
}

impl<'h> Processor<Resolved<'h>> {
    /// Stage 3, validating, with the origins remote data may come from (`https://api.example.com`; any other is
    /// F028): checks every invocation, slot, interpolation path, expression, bound input and data source.
    #[must_use]
    pub fn validate(self, allowed_origins: &[String]) -> Processor<Validated<'h>> {
        self.then(|Resolved { store, resolved }, diagnostics| {
            let mut checks = validate::validate(&resolved, allowed_origins);
            diagnostics.append(&mut checks.diagnostics);
            Validated {
                store,
                resolved,
                checks,
                kept: diagnostics.len(),
            }
        })
    }
}

impl<'h> Processor<Validated<'h>> {
    /// The remote and `host:` sources a render with `context` needs, in the order they are written, for the host
    /// to fetch before it loads.
    #[must_use]
    pub fn sources(&self, context: &Context) -> Vec<Source> {
        load::sources(&self.state.resolved, &self.state.checks, context)
    }

    /// Stage 4a, loading, for a render with `context`: the inputs' values and every data import, with the host's
    /// remote data from `fetch` and its `host:` data from `data`.
    #[must_use]
    pub fn load<'c>(
        self,
        context: &'c Context<'c>,
        fetch: &impl Fetch,
        data: &impl Data,
    ) -> Processor<Loaded<'h, 'c>> {
        self.then(|validated, diagnostics| {
            let host = load::Host {
                store: validated.store,
                fetch,
                data,
            };
            let mut data = load::load(&validated.resolved, &validated.checks, context, &host);
            diagnostics.append(&mut data.diagnostics);
            Loaded {
                validated,
                context,
                data,
            }
        })
    }
}

impl<'h> Processor<Loaded<'h, '_>> {
    /// The processed document's inputs, with the values the render uses.
    #[must_use]
    pub fn inputs(&self) -> &Inputs {
        &self.state.data.inputs
    }

    /// Stage 4b, evaluating: the expanded tree.
    #[must_use]
    pub fn expand(self, options: &expand::Options) -> Processor<Expanded<'h>> {
        self.then(
            |Loaded {
                 validated,
                 context,
                 data,
             },
             diagnostics| {
                let expanded = expand::expand(
                    &validated.resolved,
                    &validated.checks,
                    &data,
                    context,
                    options,
                );
                diagnostics.extend(expanded.diagnostics);
                Expanded {
                    validated,
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
        context: &'d Context<'d>,
        fetch: &impl Fetch,
        data: &impl Data,
    ) -> Processor<Loaded<'h, 'd>> {
        self.then(|loaded, diagnostics| back(loaded.validated, diagnostics))
            .load(context, fetch, data)
    }
}

impl Processor<Expanded<'_>> {
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
    pub fn reload(self, context: &Context, fetch: &impl Fetch, data: &impl Data) -> Self {
        let options = self.state.options.clone();
        self.then(|expanded, diagnostics| back(expanded.validated, diagnostics))
            .load(context, fetch, data)
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

    fn context(inputs: serde_json::Value) -> Context<'static> {
        let inputs: Map<String, serde_json::Value> =
            serde_json::from_value(inputs).expect("an object");
        Context {
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
        assert_eq!(loaded.inputs().0[0].value, json!(1));
        assert_eq!(loaded.diagnostics().len(), 2, "and F026 for the data");
        let two = context(json!({ "n": 2 }));
        let loaded = loaded.reload(&two, &(), &());
        assert_eq!(loaded.inputs().0[0].value, json!(2));
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
