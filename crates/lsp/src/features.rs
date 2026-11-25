//! What a client asks about a document ([the language server], *The features*): each a lookup in the document's
//! processor, answered by the document's worker, never another run.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#the-features

pub(crate) mod blocks;
mod completion;
mod definition;
mod folding;
mod hover;
mod import_edit;
mod markdown;
pub(crate) mod semantic_tokens;
mod signature_help;
mod symbols;
#[cfg(test)]
mod testing;

use compmd::Processor;
use compmd::processor::Expanded;
use lsp_types::request::{
    Completion, DocumentSymbolRequest, FoldingRangeRequest, GotoDefinition, HoverRequest,
    Request as _, SemanticTokensFullRequest, SemanticTokensRangeRequest, SignatureHelpRequest,
};
use lsp_types::{
    Position, Range, SemanticTokensParams, SemanticTokensRangeParams, TextDocumentIdentifier,
    TextDocumentPositionParams, Uri,
};
use serde::Deserialize;
use serde_json::Value;

use crate::encoding::Encoding;
use crate::pack_components::PackComponent;
use crate::places::Places;

/// A request about one document.
#[derive(Debug, PartialEq)]
pub(crate) enum DocumentRequest {
    /// Its semantic tokens: all of them, or those in a range.
    SemanticTokens(Option<Range>),
    /// What may be written at a place.
    Completion(Position),
    /// What is at a place.
    Hover(Position),
    /// What is being written at a place: a function's arguments, a component's params.
    SignatureHelp(Position),
    /// Where what a place names is defined.
    Definition(Position),
    /// Its headings, containers, leaves, slots and definitions, nested.
    Symbols,
    /// Its foldable ranges.
    Folding,
    /// The code blocks its code renderers render (`compmd/blocks`).
    Blocks,
}

/// A request's parameters when it is about a document as a whole.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OfDocument {
    text_document: TextDocumentIdentifier,
}

/// The document `params` names, asked `request`.
fn of_document(
    params: Value,
    request: DocumentRequest,
) -> Result<(Uri, DocumentRequest), serde_json::Error> {
    serde_json::from_value(params).map(|OfDocument { text_document }| (text_document.uri, request))
}

impl DocumentRequest {
    /// The request `method` with `params` as one about a document, with the document's URI; `None` for a method that
    /// is no request about a document, an error for parameters that aren't the method's.
    pub(crate) fn read(
        method: &str,
        params: Value,
    ) -> Option<Result<(Uri, Self), serde_json::Error>> {
        let at = |params: Value| {
            serde_json::from_value(params).map(
                |TextDocumentPositionParams {
                     text_document,
                     position,
                 }| { (text_document.uri, position) },
            )
        };
        let read = match method {
            SemanticTokensFullRequest::METHOD => serde_json::from_value(params).map(
                |SemanticTokensParams { text_document, .. }| {
                    (text_document.uri, Self::SemanticTokens(None))
                },
            ),
            SemanticTokensRangeRequest::METHOD => serde_json::from_value(params).map(
                |SemanticTokensRangeParams {
                     text_document,
                     range,
                     ..
                 }| { (text_document.uri, Self::SemanticTokens(Some(range))) },
            ),
            Completion::METHOD => {
                at(params).map(|(uri, position)| (uri, Self::Completion(position)))
            }
            HoverRequest::METHOD => at(params).map(|(uri, position)| (uri, Self::Hover(position))),
            SignatureHelpRequest::METHOD => {
                at(params).map(|(uri, position)| (uri, Self::SignatureHelp(position)))
            }
            GotoDefinition::METHOD => {
                at(params).map(|(uri, position)| (uri, Self::Definition(position)))
            }
            DocumentSymbolRequest::METHOD => of_document(params, Self::Symbols),
            FoldingRangeRequest::METHOD => of_document(params, Self::Folding),
            blocks::METHOD => of_document(params, Self::Blocks),
            _ => return None,
        };
        Some(read)
    }
}

/// What a request is answered from: what the document's worker knows.
pub(crate) struct Asked<'a> {
    /// The document's processor; `None` when it couldn't be processed (its project can't be read), which answers with
    /// nothing.
    pub(crate) processor: Option<&'a Processor<Expanded>>,
    /// The document's URI.
    pub(crate) uri: &'a Uri,
    /// Its text.
    pub(crate) text: &'a str,
    /// How the client counts positions.
    pub(crate) encoding: Encoding,
    /// Where the files the processor names are.
    pub(crate) places: &'a Places,
    /// The components the project's packs export, for auto-import.
    pub(crate) pack_components: &'a [PackComponent],
}

impl Asked<'_> {
    /// The code point offset of `position`.
    fn offset(&self, position: Position) -> usize {
        self.encoding.offset(self.text, position)
    }
}

/// The answer to `request`, from what `asked` holds.
pub(crate) fn answer(request: &DocumentRequest, asked: &Asked) -> Value {
    let Asked {
        processor,
        text,
        encoding,
        ..
    } = *asked;
    let hint_at = |position: &Position| processor?.hint_at(asked.offset(*position));
    let answer = match request {
        DocumentRequest::SemanticTokens(range) => {
            let tokens = processor.map(Processor::tokens).unwrap_or_default();
            serde_json::to_value(semantic_tokens::encode(&tokens, text, encoding, *range))
        }
        DocumentRequest::Completion(position) => {
            let items = hint_at(position)
                .map(|hint| completion::completions(&hint, asked))
                .unwrap_or_default();
            serde_json::to_value(items)
        }
        DocumentRequest::Hover(position) => serde_json::to_value(
            hint_at(position).and_then(|hint| hover::hover(&hint, text, encoding)),
        ),
        DocumentRequest::SignatureHelp(position) => serde_json::to_value(
            processor
                .and_then(|processor| processor.signature_at(asked.offset(*position)))
                .map(|help| signature_help::signature_help(&help)),
        ),
        DocumentRequest::Definition(position) => serde_json::to_value(
            processor
                .and_then(|processor| processor.definition_at(asked.offset(*position)))
                .and_then(|defined| definition::location(&defined, asked)),
        ),
        DocumentRequest::Symbols => serde_json::to_value(
            processor
                .map(|processor| symbols::symbols(&processor.tree().children, text, encoding))
                .unwrap_or_default(),
        ),
        DocumentRequest::Folding => serde_json::to_value(
            processor
                .map(|processor| folding::folding(&processor.tree().children))
                .unwrap_or_default(),
        ),
        DocumentRequest::Blocks => serde_json::to_value(
            processor
                .map(|processor| blocks::blocks(processor, text, encoding))
                .unwrap_or_default(),
        ),
    };
    answer.unwrap_or(Value::Null)
}
