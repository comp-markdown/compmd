//! Stage 3, validating (spec §5.3, §11): every invocation against its component's signature and the common params,
//! slots, interpolation paths, the types of expressions, bound inputs, and data sources.
//!
//! The processed document is checked whole, top-level block by top-level block, so that an edit checks only the
//! blocks it changed; of the documents it imports, the definitions, since their bodies and fences are what is used of
//! them. Each definition is checked once, however often it is used (spec §10.4).

mod body;
mod data;
mod grammar;
pub(crate) mod params;
pub(crate) mod scope;
pub(crate) mod universal;

use crate::cel::{self, Program};
use crate::diagnostic::{Code, Diagnostic, Location, Stage};
use crate::lines::Lines;
use crate::parse::Spliced;
use crate::reserved;
use crate::resolve::{DocumentId, Resolved};
use crate::tree::{Node, Param, Point, Position};
use scope::Scope;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// The checked document (spec §11): the diagnostics of validating, and what they refused, which the later stages
/// treat as absent (spec §10.3).
#[derive(Clone, Debug, Default)]
pub(crate) struct Validated {
    /// What checking the documents before the processed one found.
    before: Vec<Diagnostic>,
    /// What checking each top-level block of the processed document found.
    blocks: Vec<Checks>,
    /// What checking the processed document's inputs and data found.
    imports: Checks,
    /// What checking the documents after the processed one found.
    after: Vec<Diagnostic>,
    refused: HashSet<Refused>,
    programs: Programs,
}

/// What checking a part of a document found.
#[derive(Clone, Debug, Default)]
struct Checks {
    diagnostics: Vec<Diagnostic>,
    refused: Vec<Refused>,
    /// The inputs it binds to params of host-language components, which `from` may not interpolate (F031).
    bound_by_hosts: Vec<String>,
    /// The expressions it parsed, which hold their programs.
    sources: Vec<String>,
}

/// Something validating refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Refused {
    /// A param, by where it starts in its document: it is treated as absent.
    Param(DocumentId, Point),
    /// A data import, by its place in `md.data`: the data is its `fallback`, or null.
    Data(DocumentId, usize),
    /// An invocation, by where it starts in its document: it renders its fallback.
    Invocation(DocumentId, Point),
}

impl Refused {
    fn document(self) -> DocumentId {
        match self {
            Self::Param(id, _) | Self::Data(id, _) | Self::Invocation(id, _) => id,
        }
    }
}

/// The expressions validating parsed, by their source, each with how many checked parts wrote it, for every render to
/// evaluate. A program is parsed once however often it is written, and dropped when no part writes it any more.
#[derive(Clone, Debug, Default)]
struct Programs(HashMap<String, (Rc<Program>, usize)>);

impl Programs {
    fn parse(&mut self, source: &str) -> Result<Rc<Program>, cel::Error> {
        if let Some((program, writers)) = self.0.get_mut(source) {
            *writers += 1;
            return Ok(Rc::clone(program));
        }
        let program = Rc::new(Program::parse(source)?);
        self.0.insert(source.to_string(), (Rc::clone(&program), 1));
        Ok(program)
    }

    /// A part that wrote `source` is gone.
    fn release(&mut self, source: &str) {
        if let Some((_, writers)) = self.0.get_mut(source) {
            *writers -= 1;
            if *writers == 0 {
                self.0.remove(source);
            }
        }
    }
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

    /// The parsed program of the expression `source`, if validating checked it.
    pub(crate) fn program(&self, source: &str) -> Option<&Rc<Program>> {
        self.programs.0.get(source).map(|(program, _)| program)
    }

    /// Whether the invocation at `position` in the document `id` renders its fallback: no binding of this processor
    /// runs its language or runtime (W102), or a required param is missing (F010) or refused.
    pub(crate) fn falls_back(&self, id: DocumentId, position: Option<Position>) -> bool {
        position.is_some_and(|position| {
            self.refused
                .contains(&Refused::Invocation(id, position.start))
        })
    }

    /// The diagnostics of stage 3, in order.
    pub(crate) fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> {
        self.before
            .iter()
            .chain(self.blocks.iter().flat_map(|block| &block.diagnostics))
            .chain(&self.imports.diagnostics)
            .chain(&self.after)
    }

    /// After an edit that changed the processed document's top-level blocks `spliced.blocks`: those blocks checked
    /// again, what is after them moved, and the document's data checked again, since the inputs bound to
    /// host-language components may have changed. Whether that changed which data imports are refused, so that the
    /// data has to be loaded again.
    pub(crate) fn splice(
        &mut self,
        resolved: &Resolved,
        spliced: &Spliced,
        allowed_origins: &[String],
    ) -> bool {
        let main = resolved.main;
        let moved = spliced.moved;
        for block in &mut self.blocks {
            for diagnostic in &mut block.diagnostics {
                moved.diagnostic(diagnostic);
            }
            for refused in &mut block.refused {
                if let Refused::Param(_, start) | Refused::Invocation(_, start) = refused {
                    moved.point(start);
                }
            }
        }
        let start = spliced.blocks.start;
        let blocks = &resolved.document(main).root.children[start..start + spliced.inserted];
        let checked = self.blocks_of(resolved, blocks);
        let removed: Vec<Checks> = self
            .blocks
            .splice(spliced.blocks.clone(), checked)
            .collect();
        for source in removed.iter().flat_map(|block| &block.sources) {
            self.programs.release(source);
        }
        self.refused.retain(|refused| refused.document() != main);
        let refused = self.blocks.iter().flat_map(|block| &block.refused);
        self.refused.extend(refused.copied());
        let data = std::mem::take(&mut self.imports.refused);
        self.check_imports(resolved, allowed_origins);
        data != self.imports.refused
    }

    /// What checking each of `blocks`, the processed document's, finds.
    fn blocks_of<'r>(&mut self, resolved: &'r Resolved, blocks: &'r [Node]) -> Vec<Checks> {
        let main = resolved.main;
        let scope = Scope::body(resolved, main);
        let mut checker = Checker::new(resolved, main, &mut self.programs);
        blocks
            .iter()
            .map(|block| {
                checker.nodes(std::slice::from_ref(block), &scope);
                std::mem::take(&mut checker.found)
            })
            .collect()
    }

    /// Checks the processed document's inputs and data imports, which may not interpolate inputs its blocks bind to
    /// host-language components.
    fn check_imports(&mut self, resolved: &Resolved, allowed_origins: &[String]) {
        let bound = self
            .blocks
            .iter()
            .flat_map(|block| block.bound_by_hosts.iter().cloned())
            .collect();
        let mut checker = Checker::new(resolved, resolved.main, &mut self.programs);
        checker.bound_by_hosts = bound;
        checker.inputs();
        checker.data(allowed_origins);
        self.imports = checker.found;
        self.refused.extend(self.imports.refused.iter().copied());
    }
}

/// Validates a resolved document, stage 3 of spec §11: gives its diagnostics, and what they refused. Remote data
/// from an origin other than `allowed_origins` is F028 (spec §8.6).
pub(crate) fn validate(resolved: &Resolved, allowed_origins: &[String]) -> Validated {
    let mut validated = Validated::default();
    for id in resolved.ids() {
        let document = resolved.document(id);
        if id == resolved.main {
            validated.blocks = validated.blocks_of(resolved, &document.root.children);
            let refused = validated.blocks.iter().flat_map(|block| &block.refused);
            validated.refused.extend(refused.copied());
            validated.check_imports(resolved, allowed_origins);
            continue;
        }
        let mut checker = Checker::new(resolved, id, &mut validated.programs);
        for node in &document.root.children {
            if let Node::ComponentDefinition(definition) = node {
                checker.definition(definition);
            }
        }
        checker.bound_by_hosts = checker.found.bound_by_hosts.iter().cloned().collect();
        checker.data(allowed_origins);
        let found = checker.found;
        validated.refused.extend(found.refused);
        let list = if id < resolved.main {
            &mut validated.before
        } else {
            &mut validated.after
        };
        list.extend(found.diagnostics);
    }
    validated
}

/// Checks one document, or part of one.
struct Checker<'r, 'd> {
    resolved: &'r Resolved,
    id: DocumentId,
    lines: Lines<'r>,
    /// How diagnostics name the document.
    file: Option<String>,
    /// The inputs the document binds to params of host-language components, which `from` may not interpolate (F031).
    bound_by_hosts: HashSet<String>,
    found: Checks,
    programs: &'d mut Programs,
}

impl<'r, 'd> Checker<'r, 'd> {
    fn new(resolved: &'r Resolved, id: DocumentId, programs: &'d mut Programs) -> Self {
        Self {
            resolved,
            id,
            lines: resolved.document(id).lines(),
            file: resolved.file(id),
            bound_by_hosts: HashSet::new(),
            found: Checks::default(),
            programs,
        }
    }
}

impl Checker<'_, '_> {
    fn report(&mut self, code: Code, at: Location, message: impl Into<String>) {
        self.found.diagnostics.push(
            Diagnostic::new(code, at, message)
                .in_stage(Stage::Validate)
                .in_file(self.file.clone()),
        );
    }

    /// The expression `source` parsed, kept for every render to evaluate; parsed once however often it is written.
    fn parse(&mut self, source: &str) -> Result<Rc<Program>, cel::Error> {
        let program = self.programs.parse(source)?;
        self.found.sources.push(source.to_string());
        Ok(program)
    }

    /// Records that `param` is treated as absent.
    fn refuse_param(&mut self, param: &Param) {
        if let Some(position) = param.position {
            self.found
                .refused
                .push(Refused::Param(self.id, position.start));
        }
    }

    /// Records that the invocation at `position` renders its fallback.
    fn fall_back(&mut self, position: Option<Position>) {
        if let Some(position) = position {
            self.found
                .refused
                .push(Refused::Invocation(self.id, position.start));
        }
    }

    /// Records that the data import at `index` loads its fallback, or null.
    fn refuse_data(&mut self, index: usize) {
        self.found.refused.push(Refused::Data(self.id, index));
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
