//! Examples (spec §7.8): the processed document's ` ```compmd ` blocks with `render=output` or `render=both`, each
//! processed as a document of its own beside it, stage by stage. An example resolves from the processed document's
//! place, with its host's files, packs and policy, and loads with its render context, but its own inputs. What it
//! reports and the origins of what it expands are moved from the block's text to the processed document's, where the
//! block's text is written. Inside an example, a ` ```compmd ` block that would render shows its source (W123).

use super::{EditContext, Expanded, HostFiles, Loaded, Processor, RenderInputs, Validated, back};
use crate::AllowedOrigins;
use crate::diagnostic::{Code, Diagnostic, Location, Range};
use crate::expanded::{self, Root};
use crate::host::{Needed, Pending};
use crate::lines::LineStarts;
use crate::load;
use crate::resolve::{self, HostComponent, Resolved};
use crate::tree::{self, Node, Point};
use crate::{context::RenderContext, expand};
use std::collections::HashMap;
use std::rc::Rc;

/// The language whose blocks are examples.
const COMPMD: &str = "compmd";

/// The processed document's examples, in the order they are written.
#[derive(Default)]
pub(super) struct Examples(Vec<Example>);

/// One example.
struct Example {
    /// Its place among the examples, from 1.
    number: usize,
    /// Where its block's fence starts in the processed document, which the expanded tree finds it by.
    fence: Point,
    /// Where its text is in the processed document.
    place: Place,
    /// Its processing, at the stage the processed document is at; `None` only while it moves to the next.
    stage: Option<Stage>,
    /// How many of its diagnostics the processed document has taken so far.
    reported: usize,
}

/// An example's processing, at a stage; boxed, as a processor is large.
enum Stage {
    Validated(Box<Processor<Validated>>),
    Loaded(Box<Processor<Loaded>>),
    Expanded(Box<Processor<Expanded>>),
}

impl Stage {
    fn diagnostics(&self) -> &[Diagnostic] {
        match self {
            Self::Validated(processor) => processor.diagnostics(),
            Self::Loaded(processor) => processor.diagnostics(),
            Self::Expanded(processor) => processor.diagnostics(),
        }
    }

    /// What stages 1 to 3 made.
    fn validated(&self) -> &Validated {
        match self {
            Self::Validated(processor) => &processor.state,
            Self::Loaded(processor) => &processor.state.validated,
            Self::Expanded(processor) => &processor.state.validated,
        }
    }
}

/// Where an example's text is in the processed document: its lines are the lines after the fence's, each indented as
/// the fence is, which the block's text leaves out.
struct Place {
    /// The fence's line.
    fence_line: usize,
    /// The fence's indentation, in code points.
    indent: usize,
    /// Where each of the example's lines starts in the processed document, in code points.
    starts: Vec<usize>,
}

impl Place {
    fn new(fence: Point, lines: usize, starts: &LineStarts) -> Self {
        Self {
            fence_line: fence.line,
            indent: fence.column - 1,
            starts: (1..=lines)
                .map(|line| starts.start(fence.line + line).unwrap_or_default())
                .collect(),
        }
    }

    fn location(&self, location: Location) -> Location {
        Location::new(
            self.fence_line + location.line,
            location.column + self.indent,
        )
    }

    fn point(&self, point: Point) -> Point {
        let start = self.starts.get(point.line - 1).copied().unwrap_or_default();
        Point {
            line: self.fence_line + point.line,
            column: point.column + self.indent,
            offset: start + self.indent + point.column - 1,
        }
    }

    fn range(&self, range: &mut Range) {
        range.start = self.location(range.start);
        range.end = range.end.map(|end| self.location(end));
    }

    /// `diagnostic` of the example, at its place in the processed document; one in another document stays.
    fn diagnostic(&self, diagnostic: &Diagnostic) -> Diagnostic {
        let mut moved = diagnostic.clone();
        if moved.file.is_none() {
            self.range(&mut moved.range);
            if let Some(fix) = &mut moved.fix {
                self.range(&mut fix.range);
            }
        }
        for related in moved
            .related
            .iter_mut()
            .filter(|related| related.file.is_none())
        {
            self.range(&mut related.range);
        }
        moved
    }

    /// The origins of `nodes` and what they hold, at their places in the processed document.
    fn origins(&self, nodes: &mut [expanded::Node]) {
        for node in nodes {
            if let Some(origin) = &mut node.origin {
                origin.start = self.point(origin.start);
                origin.end = self.point(origin.end);
            }
            if let expanded::Content::ComponentContainer(container) = &mut node.content {
                for slot in &mut container.slots {
                    if let Some(origin) = &mut slot.origin {
                        origin.start = self.point(origin.start);
                        origin.end = self.point(origin.end);
                    }
                }
            }
            for list in node.content.lists_mut() {
                self.origins(list);
            }
        }
    }
}

/// The ` ```compmd ` blocks of `nodes` that render, at any depth of the body, in written order: not those in a
/// definition's body, which is another document's text.
fn blocks<'t>(nodes: &'t [Node], found: &mut Vec<&'t tree::Code>) {
    for node in nodes {
        match node {
            Node::ComponentDefinition(_) | Node::ComponentSignature(_) => {}
            Node::Code(code) if renders(code) => found.push(code),
            Node::ComponentContainer(container) => {
                blocks(&container.children, found);
                for slot in &container.slots {
                    blocks(&slot.children, found);
                }
            }
            _ => blocks(node.children(), found),
        }
    }
}

/// Whether `code` is a ` ```compmd ` block that renders.
fn renders(code: &tree::Code) -> bool {
    code.lang.as_deref() == Some(COMPMD)
        && code.info.as_ref().is_some_and(|info| info.render.draws())
}

impl Examples {
    /// The examples of `resolved`'s processed document, each processed through stage 3 with the processed document's
    /// `store`, `context` and `allowed_origins`, their diagnostics added to `diagnostics`. An example makes none: its
    /// blocks that would render are W123.
    pub(super) fn of(
        resolved: &Resolved,
        store: &HostFiles,
        context: &EditContext,
        allowed_origins: &AllowedOrigins,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Self {
        let document = resolved.document(resolved.main);
        let mut found = Vec::new();
        blocks(&document.root.children, &mut found);
        if context.example {
            for code in found {
                diagnostics.push(
                    Diagnostic::new(
                        Code::W123,
                        tree::start_of(code.position),
                        "an example inside an example shows its source; render it where it is written as a document",
                    )
                    .in_stage(crate::diagnostic::Stage::Validate),
                );
            }
            return Self::default();
        }
        let examples = found
            .into_iter()
            .enumerate()
            .filter_map(|(index, code)| {
                let fence = code.position?.start;
                let lines = code.value.split('\n').count();
                let processor = Processor::new(context.path.clone(), code.value.clone())
                    .parse_example(&context.options)
                    .resolve_in(
                        HostFiles {
                            files: Rc::clone(&store.files),
                            packs: Rc::clone(&store.packs),
                        },
                        context.ambient.clone(),
                        context.policy.clone(),
                        resolve::Parses::default(),
                    )
                    .validate(allowed_origins);
                let mut example = Example {
                    number: index + 1,
                    fence,
                    place: Place::new(fence, lines, &document.line_starts),
                    stage: Some(Stage::Validated(Box::new(processor))),
                    reported: 0,
                };
                example.report(diagnostics);
                Some(example)
            })
            .collect();
        Self(examples)
    }

    /// Whether the processed document has none.
    pub(super) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Stage 4a for each example, with `render`'s context and data, but the example's own inputs: the values `render`
    /// gives for its number, or else their defaults.
    pub(super) fn load(&mut self, render: &RenderInputs, diagnostics: &mut Vec<Diagnostic>) {
        for example in &mut self.0 {
            if let Some(Stage::Validated(processor)) = example.stage.take() {
                let inputs = RenderInputs {
                    context: RenderContext {
                        inputs: render
                            .examples
                            .get(&example.number)
                            .cloned()
                            .unwrap_or_default(),
                        ..render.context.clone()
                    },
                    fetch: Rc::clone(&render.fetch),
                    data: Rc::clone(&render.data),
                    examples: HashMap::new(),
                };
                example.stage = Some(Stage::Loaded(Box::new(processor.load_with(inputs))));
            }
            example.report(diagnostics);
        }
    }

    /// Stage 4b for each example: its expanded root, by where its fence starts in the processed document, in code
    /// points, with its number and its origins in the processed document.
    pub(super) fn expand(
        &mut self,
        options: &expand::Options,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> HashMap<usize, Root> {
        let mut roots = HashMap::new();
        for example in &mut self.0 {
            if let Some(Stage::Loaded(processor)) = example.stage.take() {
                example.stage = Some(Stage::Expanded(Box::new(processor.expand(options))));
            }
            example.report(diagnostics);
            if let Some(Stage::Expanded(processor)) = &example.stage {
                let mut root = processor.expanded().clone();
                root.example = Some(example.number);
                example.place.origins(&mut root.children);
                roots.insert(example.fence.offset, root);
            }
        }
        roots
    }

    /// Each example back to stage 3, its later diagnostics dropped, as the processed document goes back.
    pub(super) fn back(&mut self) {
        for example in &mut self.0 {
            let validated = match example.stage.take() {
                Some(Stage::Loaded(processor)) => {
                    processor.then(|loaded, found| back(loaded.validated, found))
                }
                Some(Stage::Expanded(processor)) => {
                    processor.then(|expanded, found| back(expanded.validated, found))
                }
                Some(Stage::Validated(processor)) => *processor,
                None => continue,
            };
            example.reported = validated.diagnostics().len();
            example.stage = Some(Stage::Validated(Box::new(validated)));
        }
    }

    /// The examples' host-language components, each with its example's number.
    pub(super) fn host_components(&self) -> Vec<HostComponent> {
        self.each()
            .flat_map(|(number, validated)| {
                let mut components = validated.resolved.host_components(validated.store.store());
                for component in &mut components {
                    component.example = Some(number);
                }
                components
            })
            .collect()
    }

    /// What the examples need besides what the processed document does, for a render with `context`, added to
    /// `needed`, each once.
    pub(super) fn sources(&self, context: &RenderContext, needed: &mut Needed) {
        let context = RenderContext {
            inputs: serde_json::Map::new(),
            ..context.clone()
        };
        for (_, validated) in self.each() {
            let theirs = load::sources(
                &validated.resolved,
                &validated.checks,
                &context,
                &validated.allowed_origins,
            );
            add(&mut needed.remote, theirs.remote);
            add(&mut needed.host, theirs.host);
            add(&mut needed.disallowed, theirs.disallowed);
            add(&mut needed.packs, theirs.packs);
        }
    }

    /// What the examples treated as pending, added to `pending`, each once.
    pub(super) fn pending(&self, pending: &mut Pending) {
        for example in &self.0 {
            if let Some(Stage::Expanded(processor)) = &example.stage {
                let theirs = processor.pending();
                add(&mut pending.packs, theirs.packs);
                add(&mut pending.remote, theirs.remote);
                add(&mut pending.host, theirs.host);
            }
        }
    }

    /// Each example's number, with what its stages 1 to 3 made.
    fn each(&self) -> impl Iterator<Item = (usize, &Validated)> {
        self.0
            .iter()
            .filter_map(|example| Some((example.number, example.stage.as_ref()?.validated())))
    }
}

impl Example {
    /// Adds what it reported since the last time to `diagnostics`, at their places in the processed document.
    fn report(&mut self, diagnostics: &mut Vec<Diagnostic>) {
        let Some(stage) = &self.stage else { return };
        let found = stage.diagnostics();
        diagnostics.extend(
            found[self.reported.min(found.len())..]
                .iter()
                .map(|diagnostic| self.place.diagnostic(diagnostic)),
        );
        self.reported = found.len();
    }
}

/// `theirs` added to `ours`, each once.
fn add<T: PartialEq>(ours: &mut Vec<T>, theirs: Vec<T>) {
    for item in theirs {
        if !ours.contains(&item) {
            ours.push(item);
        }
    }
}
