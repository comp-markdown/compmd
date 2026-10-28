//! The processor (spec §11), its stages as states: a [`Processor`] goes from [`New`] through [`Parsed`],
//! [`Resolved`], [`Validated`] and [`Loaded`] to [`Expanded`]. Each stage takes the processor and gives it back in its
//! next state, so the stages can only run in order; each state gives what its stage made, and every state the
//! diagnostics so far. Each stage takes what it uses: the parse options, the origins remote data may come from, the
//! render context, the expansion options.
//!
//! ```
//! use compmd::context::Context;
//! use compmd::expand::Options;
//! use compmd::resolve::Sources;
//! use compmd::{ParseOptions, Processor};
//! use std::collections::HashMap;
//!
//! let text = "---\nteam: [Ana, Kenji]\n---\n\n::: each {in=(md.doc.team) as=p join=inline sep=\", \"}\n{{p}}\n::: /each\n";
//! let files: HashMap<String, String> = HashMap::new();
//! let sources = Sources::new(&files);
//! let context = Context { now: 0, tz: "UTC".into(), zones: &(), inputs: Default::default() };
//! let processor = Processor::new("team.comp.md", text, &sources)
//!     .parse(&ParseOptions::default())
//!     .resolve()
//!     .validate(&[])
//!     .load(&context)
//!     .expand(&Options::default());
//! assert!(processor.diagnostics().is_empty());
//! let json = serde_json::to_value(processor.expanded()).unwrap();
//! assert_eq!(json["children"][0]["children"][0]["value"], "Ana, Kenji");
//! ```

use crate::context::Context;
use crate::diagnostic::Diagnostic;
use crate::expanded::{self, Inputs};
use crate::load::{self, Source};
use crate::process::Processed;
use crate::resolve::{self, Sources};
use crate::{ParseOptions, expand, tree, validate};

/// A document on its way through the stages, in the state `S`.
pub struct Processor<'s, S> {
    sources: &'s Sources<'s>,
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
pub struct Resolved {
    resolved: resolve::Resolved,
}

/// After stage 3: the checked documents, and what checking refused.
pub struct Validated {
    resolved: resolve::Resolved,
    checks: validate::Validated,
    /// How many diagnostics stages 1 to 3 gave, which a reload keeps.
    kept: usize,
}

/// After stage 4a: the inputs' values and the data, for one render context.
pub struct Loaded<'c> {
    validated: Validated,
    context: &'c Context<'c>,
    data: load::Loaded,
}

/// After stage 4b: the expanded tree.
pub struct Expanded {
    validated: Validated,
    options: expand::Options,
    root: expanded::Root,
}

impl<'s, S> Processor<'s, S> {
    /// The diagnostics of every stage run so far, in stage order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// The next state, which `step` makes from this one, adding the diagnostics it finds.
    fn then<T>(
        self,
        step: impl FnOnce(S, &'s Sources<'s>, &mut Vec<Diagnostic>) -> T,
    ) -> Processor<'s, T> {
        let Processor {
            sources,
            mut diagnostics,
            state,
        } = self;
        let state = step(state, sources, &mut diagnostics);
        Processor {
            sources,
            diagnostics,
            state,
        }
    }
}

impl<'s> Processor<'s, New> {
    /// A processor for the document at `path` in the project, with `text`; its imports and data come from
    /// `sources`.
    #[must_use]
    pub fn new(path: impl Into<String>, text: impl Into<String>, sources: &'s Sources<'s>) -> Self {
        Self {
            sources,
            diagnostics: Vec::new(),
            state: New {
                path: path.into(),
                text: text.into(),
            },
        }
    }

    /// Stage 1, parsing: the document's tree (spec §12.1).
    #[must_use]
    pub fn parse(self, options: &ParseOptions) -> Processor<'s, Parsed> {
        self.then(|New { path, text }, _, diagnostics| {
            let parsed = crate::parse(&text, options);
            diagnostics.extend(parsed.diagnostics);
            Parsed {
                path,
                text,
                root: parsed.root,
            }
        })
    }
}

impl<'s> Processor<'s, Parsed> {
    /// The document's tree.
    #[must_use]
    pub fn tree(&self) -> &tree::Root {
        &self.state.root
    }

    /// Stage 2, resolving: loads the document's imports and their imports, `@compmd/core` and the ambient packs,
    /// and binds every name.
    #[must_use]
    pub fn resolve(self) -> Processor<'s, Resolved> {
        self.then(|Parsed { path, text, root }, sources, diagnostics| {
            let mut resolved = resolve::resolve(&path, &text, root, sources);
            diagnostics.append(&mut resolved.diagnostics);
            Resolved { resolved }
        })
    }
}

impl<'s> Processor<'s, Resolved> {
    /// The documents and the components their names stand for.
    #[must_use]
    pub fn resolved(&self) -> &resolve::Resolved {
        &self.state.resolved
    }

    /// Stage 3, validating, with the origins remote data may come from (`https://api.example.com`; any other is
    /// F028): checks every invocation, slot, interpolation path, expression, bound input and data source.
    #[must_use]
    pub fn validate(self, allowed_origins: &[String]) -> Processor<'s, Validated> {
        self.then(|Resolved { resolved }, _, diagnostics| {
            let mut checks = validate::validate(&resolved, allowed_origins);
            diagnostics.append(&mut checks.diagnostics);
            Validated {
                resolved,
                checks,
                kept: diagnostics.len(),
            }
        })
    }
}

impl<'s> Processor<'s, Validated> {
    /// The documents and the components their names stand for.
    #[must_use]
    pub fn resolved(&self) -> &resolve::Resolved {
        &self.state.resolved
    }

    /// The remote and `host:` sources a render with `context` needs, in the order they are written, for the host
    /// to fetch before it loads.
    #[must_use]
    pub fn sources(&self, context: &Context) -> Vec<Source> {
        load::sources(&self.state.resolved, &self.state.checks, context)
    }

    /// Stage 4a, loading, for a render with `context`: the inputs' values and every data import.
    #[must_use]
    pub fn load<'c>(self, context: &'c Context<'c>) -> Processor<'s, Loaded<'c>> {
        self.then(|validated, sources, diagnostics| {
            let mut data = load::load(&validated.resolved, &validated.checks, context, sources);
            diagnostics.append(&mut data.diagnostics);
            Loaded {
                validated,
                context,
                data,
            }
        })
    }
}

impl<'s> Processor<'s, Loaded<'_>> {
    /// The processed document's inputs, with the values the render uses.
    #[must_use]
    pub fn inputs(&self) -> &Inputs {
        &self.state.data.inputs
    }

    /// Stage 4b, evaluating: the expanded tree.
    #[must_use]
    pub fn expand(self, options: &expand::Options) -> Processor<'s, Expanded> {
        self.then(
            |Loaded {
                 validated,
                 context,
                 data,
             },
             _,
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

    /// Loads again for a render with `context`, without running stages 1 to 3 again.
    #[must_use]
    pub fn reload<'d>(self, context: &'d Context<'d>) -> Processor<'s, Loaded<'d>> {
        self.then(|loaded, _, diagnostics| back(loaded.validated, diagnostics))
            .load(context)
    }
}

impl Processor<'_, Expanded> {
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

    /// Loads and expands again for a render with `context`, with the same options, without running stages 1 to 3
    /// again.
    #[must_use]
    pub fn reload(self, context: &Context) -> Self {
        let options = self.state.options.clone();
        self.then(|expanded, _, diagnostics| back(expanded.validated, diagnostics))
            .load(context)
            .expand(&options)
    }
}

/// The validated state again, with the diagnostics of stage 4 dropped.
fn back(validated: Validated, diagnostics: &mut Vec<Diagnostic>) -> Validated {
    diagnostics.truncate(validated.kept);
    validated
}

impl From<Processor<'_, Expanded>> for Processed {
    fn from(processor: Processor<'_, Expanded>) -> Self {
        let Expanded {
            validated, root, ..
        } = processor.state;
        let main = validated.resolved.main;
        Self {
            parse: validated.resolved.into_document(main).root,
            expanded: root,
            diagnostics: processor.diagnostics,
        }
    }
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
            zones: &(),
            inputs,
        }
    }

    #[test]
    fn each_state_gives_what_its_stage_made_and_reloads_keep_stages_1_to_3() {
        let text = "---\nmd:\n  data:\n    - from: ./missing.json\n      as: m\n  inputs:\n    n: number = 1\n---\n\n{{md.input.n}} :x[y]\n";
        let files: HashMap<String, String> = HashMap::new();
        let sources = Sources::new(&files);
        let parsed = Processor::new("a.comp.md", text, &sources).parse(&ParseOptions::default());
        assert_eq!(parsed.tree().children.len(), 2);
        let resolved = parsed.resolve();
        assert_eq!(resolved.resolved().documents.len(), 2, "with @compmd/core");
        let validated = resolved.validate(&[]);
        assert_eq!(validated.diagnostics().len(), 1, "F002 for `x`");
        let one = context(json!({}));
        let loaded = validated.load(&one);
        assert_eq!(loaded.inputs().0[0].value, json!(1));
        assert_eq!(loaded.diagnostics().len(), 2, "and F026 for the data");
        let two = context(json!({ "n": 2 }));
        let loaded = loaded.reload(&two);
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
        let expanded = expanded.reload(&three);
        assert_eq!(written(&expanded), json!("3 y"));
        assert_eq!(expanded.diagnostics().len(), 2);
        assert_eq!(expanded.tree().children.len(), 2);
    }
}
