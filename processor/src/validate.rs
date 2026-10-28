//! Stage 3, validating (spec §5.3, §11): every invocation against its component's signature and the common params,
//! slots, interpolation paths, the types of expressions, bound inputs, and data sources.
//!
//! The processed document is checked whole; of the documents it imports, the definitions, since their bodies and
//! fences are what is used of them. Each definition is checked once, however often it is used (spec §10.4).

mod body;
mod data;
mod grammar;
pub(crate) mod params;
pub(crate) mod scope;
pub(crate) mod universal;

use crate::diagnostic::{Code, Diagnostic, Location, Stage};
use crate::lines::Lines;
use crate::reserved;
use crate::resolve::{DocumentId, Resolved};
use crate::tree::{Node, Param, Point};
use scope::Scope;
use std::collections::HashSet;

/// What validating needs besides the resolved document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// The origins remote data may come from (`https://api.example.com`); any other is F028 (spec §8.6).
    pub allowed_origins: Vec<String>,
}

/// The checked document (spec §11): the diagnostics of validating, and what they refused, which the later stages
/// treat as absent (spec §10.3).
#[derive(Clone, Debug, Default)]
pub struct Validated {
    /// The diagnostics of stage 3.
    pub diagnostics: Vec<Diagnostic>,
    refused: HashSet<Refused>,
}

/// Something validating refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Refused {
    /// A param, by where it starts in its document: it is treated as absent.
    Param(DocumentId, Point),
    /// A data import, by its place in `md.data`: the data is its `fallback`, or null.
    Data(DocumentId, usize),
}

impl Validated {
    /// Whether the param of the document `id` was refused.
    pub(crate) fn refuses_param(&self, id: DocumentId, param: &Param) -> bool {
        param
            .position
            .is_some_and(|position| self.refused.contains(&Refused::Param(id, position.start)))
    }

    /// Whether the data import at `index` of the document `id` was refused.
    pub(crate) fn refuses_data(&self, id: DocumentId, index: usize) -> bool {
        self.refused.contains(&Refused::Data(id, index))
    }
}

/// Validates a resolved document, stage 3 of spec §11: gives its diagnostics, and what they refused.
///
/// ```
/// use compmd::resolve::{Sources, resolve};
/// use compmd::validate::{Options, validate};
/// use compmd::{ParseOptions, parse};
/// use std::collections::HashMap;
///
/// let text = "::: row {gap=bigger}\nText.\n::: /row\n";
/// let files: HashMap<String, String> = HashMap::new();
/// let sources = Sources { files: &files, registry: &(), ambient: Vec::new(), report_ambient: false };
/// let resolved = resolve("notes.comp.md", text, parse(text, &ParseOptions::default()), &sources);
/// let validated = validate(&resolved, &Options::default());
/// assert_eq!(validated.diagnostics[0].code.as_str(), "F009");
/// ```
#[must_use]
pub fn validate(resolved: &Resolved, options: &Options) -> Validated {
    let mut validated = Validated::default();
    for id in resolved.ids() {
        let document = resolved.document(id);
        let mut checker = Checker {
            resolved,
            id,
            lines: Lines::new(&document.text),
            file: resolved.file(id),
            bound_by_hosts: HashSet::new(),
            validated: &mut validated,
        };
        if id == resolved.main {
            checker.nodes(
                &document.root.children,
                &Scope::document(&document.settings),
            );
            checker.inputs();
        } else {
            for node in &document.root.children {
                if let Node::ComponentDefinition(definition) = node {
                    checker.definition(definition);
                }
            }
        }
        checker.data(&options.allowed_origins);
    }
    validated
}

/// Checks one document.
struct Checker<'r, 'd> {
    resolved: &'r Resolved,
    id: DocumentId,
    lines: Lines<'r>,
    /// How diagnostics name the document.
    file: Option<String>,
    /// The inputs bound to params of host-language components, which `from` may not interpolate (F031).
    bound_by_hosts: HashSet<String>,
    validated: &'d mut Validated,
}

impl Checker<'_, '_> {
    fn report(&mut self, code: Code, at: Location, message: impl Into<String>) {
        self.validated.diagnostics.push(
            Diagnostic::new(code, at, message)
                .in_stage(Stage::Validate)
                .in_file(self.file.clone()),
        );
    }

    /// Records that `param` is treated as absent.
    fn refuse_param(&mut self, param: &Param) {
        if let Some(position) = param.position {
            self.validated
                .refused
                .insert(Refused::Param(self.id, position.start));
        }
    }

    /// Records that the data import at `index` loads its fallback, or null.
    fn refuse_data(&mut self, index: usize) {
        self.validated.refused.insert(Refused::Data(self.id, index));
    }

    /// F013 for an input named what expressions can't use.
    fn inputs(&mut self) {
        let document = self.resolved.document(self.id);
        for input in &document.settings.inputs {
            if let Some(why) = reserved::as_variable(&input.declaration.name) {
                self.report(Code::F013, input.at, why);
            }
        }
    }
}
