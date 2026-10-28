//! Stage 3, validating (spec §5.3, §11): every invocation against its component's signature and the common params,
//! slots, interpolation paths, the types of expressions, bound inputs, and data sources.
//!
//! The processed document is checked whole; of the documents it imports, the definitions, since their bodies and
//! fences are what is used of them. Each definition is checked once, however often it is used (spec §10.4).

mod body;
mod data;
mod grammar;
mod params;
mod scope;
mod universal;

use crate::diagnostic::{Code, Diagnostic, Location, Stage};
use crate::lines::Lines;
use crate::reserved;
use crate::resolve::{DocumentId, Resolved};
use crate::tree::Node;
use scope::Scope;
use std::collections::HashSet;

/// What validating needs besides the resolved document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Options {
    /// The origins remote data may come from (`https://api.example.com`); any other is F028 (spec §8.6).
    pub allowed_origins: Vec<String>,
}

/// Validates a resolved document, stage 3 of spec §11, and gives its diagnostics.
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
/// let diagnostics = validate(&resolved, &Options::default());
/// assert_eq!(diagnostics[0].code.as_str(), "F009");
/// ```
#[must_use]
pub fn validate(resolved: &Resolved, options: &Options) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for id in resolved.ids() {
        let document = resolved.document(id);
        let mut checker = Checker {
            resolved,
            id,
            lines: Lines::new(&document.text),
            file: resolved.file(id),
            bound_by_hosts: HashSet::new(),
            diagnostics: &mut diagnostics,
        };
        if id == resolved.main {
            checker.nodes(
                &document.root.children,
                &Scope::document(&document.settings),
            );
            checker.inputs();
            checker.data(&options.allowed_origins);
        } else {
            for node in &document.root.children {
                if let Node::ComponentDefinition(definition) = node {
                    checker.definition(definition);
                }
            }
        }
    }
    diagnostics
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
    diagnostics: &'d mut Vec<Diagnostic>,
}

impl Checker<'_, '_> {
    fn report(&mut self, code: Code, at: Location, message: impl Into<String>) {
        self.diagnostics.push(
            Diagnostic::new(code, at, message)
                .in_stage(Stage::Validate)
                .in_file(self.file.clone()),
        );
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
