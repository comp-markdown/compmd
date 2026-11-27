//! `compmd/drawing` ([a document drawn in the editor], decisions 1 to 6 and 11): the drawing plan of a range of a
//! document, made from the processor's tree, positions and tokens, which an editor's renderer draws without knowing
//! Markdown or CompMD. The plan uses only the primitives the client declared at `initialize` (decision 3), each mark
//! with its style and the lines it gives way on, in the order they start; its ranges and positions are in the encoding
//! agreed at `initialize`.
//!
//! The plan is always of the document as the server holds it, which has every change the client sent before the
//! request; its `version` says which, and a renderer drops a plan older than its buffer (decision 4). The request's
//! own version isn't checked.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

mod box_border;
mod callout;
mod checkbox;
mod client_drawing;
mod code_block;
mod container;
mod frontmatter;
mod glyphs;
mod heading;
mod inline_code;
mod layout;
mod leaf;
mod line_span;
mod list;
mod mark;
mod param_spans;
mod planner;
mod planning;
mod primitive;
mod quote;
mod refresh;
mod rule;
mod source;
mod style;
mod table;
mod walk;

pub(crate) use client_drawing::ClientDrawing;
pub(crate) use layout::Layout;
pub(crate) use planning::Planning;
pub(crate) use refresh::Refresh;
pub(crate) use source::Source;

use compmd::Processor;
use compmd::processor::Expanded;
use compmd::tree::Node;
use lsp_types::{Range, Uri, VersionedTextDocumentIdentifier};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Asked;
use line_span::LineSpan;
use mark::Mark;
use planner::Planner;
use walk::Nesting;

/// The request's method.
pub(crate) const METHOD: &str = "compmd/drawing";

/// The request's parameters.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DrawingParams {
    text_document: VersionedTextDocumentIdentifier,
    range: Range,
}

/// The document and the range `params` asks the plan of.
pub(crate) fn read(params: Value) -> Result<(Uri, Range), serde_json::Error> {
    serde_json::from_value(params).map(
        |DrawingParams {
             text_document,
             range,
         }| (text_document.uri, range),
    )
}

/// The answer: the plan of the document at `version`.
#[derive(Debug, PartialEq, Serialize)]
pub(crate) struct Plan {
    pub(crate) version: i32,
    pub(crate) marks: Vec<Mark>,
}

/// The plan of the lines of `range` of the document `asked` is about, made with `planning`: no marks when the drawing
/// is off or the document couldn't be processed.
pub(crate) fn plan(range: Range, planning: &Planning, asked: &Asked) -> Plan {
    let drawn = planning.render.enabled && (planning.render.markdown_files || !asked.markdown_file);
    let marks = match asked.processor {
        Some(processor) if drawn => marks(processor, range, planning, asked),
        _ => Vec::new(),
    };
    Plan {
        version: asked.version,
        marks,
    }
}

fn marks(
    processor: &Processor<Expanded>,
    range: Range,
    planning: &Planning,
    asked: &Asked,
) -> Vec<Mark> {
    let tokens = processor.tokens();
    let mut planner = Planner::new(Source::new(asked.text, asked.encoding), planning, &tokens);
    let lines = LineSpan::of_range(&range);
    for node in &processor.tree().children {
        if lines_of(node).is_none_or(|of| of.meets(lines)) {
            walk::draw(std::slice::from_ref(node), &mut planner, Nesting::default());
        }
    }
    planner.finish(lines)
}

/// The lines a block of the document's top level is drawn on: a container's from its opener to its closer.
fn lines_of(node: &Node) -> Option<LineSpan> {
    let end = match node {
        Node::ComponentContainer(container) => container.close_position?.end,
        _ => node.position()?.end,
    };
    let start = node.position()?.start;
    Some(LineSpan::of(Source::place(start).0, Source::place(end).0))
}

#[cfg(test)]
mod tests;
