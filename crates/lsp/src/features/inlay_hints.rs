//! Values as inlay hints ([a document drawn in the editor], decision 9): beside each interpolation and each expression
//! param of the document, the value the processor's last render gave it (`compmd::evaluations`), as [`ValueLabel`]
//! writes it. A hint is after the interpolation's `}}` or the expression's `)`, padded on its left, and has no `kind`:
//! LSP's kinds are a type and a parameter, and without one "the client should fall back to a reasonable default".
//! An evaluation that failed has none, since its diagnostic says why; nor has what the render didn't reach.
//!
//! `compmd.render.values` turns them off; the server then answers with none.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

mod value_label;

use std::collections::HashMap;

use compmd::Processor;
use compmd::diagnostic::Location;
use compmd::processor::Expanded;
use compmd::tokens::{SyntaxToken, TokenKind};
use lsp_types::{ClientCapabilities, InlayHint, InlayHintLabel, InlayHintTooltip, Range};
pub(crate) use value_label::ValueLabel;

use super::Asked;

/// The inlay hints of the document `asked` holds whose position is in `range`, in written order.
pub(crate) fn inlay_hints(range: Range, asked: &Asked) -> Vec<InlayHint> {
    let Some(processor) = asked.processor else {
        return Vec::new();
    };
    let ends = ends(&processor.tokens());
    let hint = |evaluation: &compmd::evaluations::Evaluation| {
        let end = ends.get(&evaluation.at)?;
        let ValueLabel { label, tooltip } = ValueLabel::of(evaluation)?;
        let position = asked.encoding.position(asked.text, end.line, end.column);
        (range.start <= position && position <= range.end).then(|| InlayHint {
            position,
            label: InlayHintLabel::String(label),
            kind: None,
            text_edits: None,
            tooltip: tooltip.map(InlayHintTooltip::String),
            padding_left: Some(true),
            padding_right: None,
            data: None,
        })
    };
    processor.evaluations().iter().filter_map(hint).collect()
}

/// What the document's hints show, in written order, wherever they are: when it changes, the hints changed.
pub(crate) fn values_shown(processor: &Processor<Expanded>) -> Vec<ValueLabel> {
    processor
        .evaluations()
        .iter()
        .filter_map(ValueLabel::of)
        .collect()
}

/// Whether a client of `capabilities` takes `workspace/inlayHint/refresh`.
pub(crate) fn takes_refreshes(capabilities: &ClientCapabilities) -> bool {
    capabilities
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.inlay_hint.as_ref()?.refresh_support)
        == Some(true)
}

/// Where each interpolation and expression of `tokens` ends, past its `}}` or its `)`, by where its path or its `(`
/// starts, as an evaluation names it.
fn ends(tokens: &[SyntaxToken]) -> HashMap<Location, Location> {
    let mut ends = HashMap::new();
    let (mut path, mut paren) = (None, None);
    for token in tokens {
        let start = token.range.start;
        let opened = match token.kind {
            TokenKind::InterpolationPath => {
                path = Some(start);
                continue;
            }
            TokenKind::InterpolationBrace => path.take(),
            TokenKind::ExpressionParen => match paren.take() {
                None => {
                    paren = Some(start);
                    continue;
                }
                opened => opened,
            },
            _ => continue,
        };
        if let (Some(opened), Some(end)) = (opened, token.range.end) {
            ends.insert(opened, end);
        }
    }
    ends
}

#[cfg(test)]
mod tests;
