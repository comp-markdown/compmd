//! What a client asks about a document ([the language server], *The features*): each a lookup in the document's
//! processor, answered by the document's worker, never another run.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#the-features

pub(crate) mod blocks;
pub(crate) mod broken_links;
pub(crate) mod code_actions;
mod completion;
mod definition;
pub(crate) mod drawing;
pub(crate) mod file_renames;
mod folding;
mod formatting;
mod hover;
mod import_edit;
pub(crate) mod inlay_hints;
mod link_completion;
mod link_definition;
mod link_hover;
mod link_targets;
mod markdown;
pub(crate) mod on_type_formatting;
mod references;
mod rename;
pub(crate) mod semantic_tokens;
mod signature_help;
mod symbols;
mod tables;
#[cfg(test)]
pub(crate) mod testing;
mod wiki_completion;
pub(crate) mod workspace_symbols;

use std::path::Path;
use std::sync::Arc;

use compmd::Processor;
use compmd::processor::Expanded;
use lsp_types::request::{
    CodeActionRequest, Completion, DocumentSymbolRequest, FoldingRangeRequest, Formatting,
    GotoDefinition, HoverRequest, InlayHintRequest, OnTypeFormatting, PrepareRenameRequest,
    RangeFormatting, References, Rename, Request as _, SemanticTokensFullRequest,
    SemanticTokensRangeRequest, SignatureHelpRequest,
};
use lsp_types::{
    CodeActionKind, CodeActionParams, DocumentOnTypeFormattingParams,
    DocumentRangeFormattingParams, InlayHintParams, Position, Range, ReferenceParams, RenameParams,
    SemanticTokensParams, SemanticTokensRangeParams, TextDocumentIdentifier,
    TextDocumentPositionParams, Uri,
};
use serde::Deserialize;
use serde_json::Value;

use crate::encoding::Encoding;
use crate::own_packs::Unregistered;
use crate::pack_components::PackComponent;
use crate::places::Places;
use crate::project_index::ProjectIndex;
use crate::settings::LinkSettings;

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
    /// What refers to what is at a place, with its own place when `with_declaration`.
    References {
        position: Position,
        with_declaration: bool,
    },
    /// What a rename at a place renames.
    PrepareRename(Position),
    /// What is at a place renamed `new_name`.
    Rename {
        position: Position,
        new_name: String,
    },
    /// Its headings, containers, leaves, slots and definitions, nested.
    Symbols,
    /// Its foldable ranges.
    Folding,
    /// The code blocks its code renderers render (`compmd/blocks`).
    Blocks,
    /// The drawing plan of a range (`compmd/drawing`), made with `planning`, as the server's settings and the client's
    /// primitives are when it asks.
    Drawing {
        range: Range,
        planning: Arc<drawing::Planning>,
    },
    /// What the server offers to do at a range, of the kinds `only` names (any when `None`).
    CodeActions {
        range: Range,
        only: Option<Vec<CodeActionKind>>,
    },
    /// The edits that format it: all of it, or a range.
    Formatting(Option<Range>),
    /// The edits after `typed` was typed just before `position`.
    OnTypeFormatting { position: Position, typed: String },
    /// The values beside its interpolations and expressions in a range.
    InlayHints(Range),
}

/// A request's parameters when it is about a document as a whole.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OfDocument {
    text_document: TextDocumentIdentifier,
}

/// The document and the place `params` names, asked what `request` makes of the place.
fn at_position(
    params: Value,
    request: fn(Position) -> DocumentRequest,
) -> Result<(Uri, DocumentRequest), serde_json::Error> {
    serde_json::from_value(params).map(
        |TextDocumentPositionParams {
             text_document,
             position,
         }| (text_document.uri, request(position)),
    )
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
            Completion::METHOD => at_position(params, Self::Completion),
            HoverRequest::METHOD => at_position(params, Self::Hover),
            SignatureHelpRequest::METHOD => at_position(params, Self::SignatureHelp),
            GotoDefinition::METHOD => at_position(params, Self::Definition),
            References::METHOD => serde_json::from_value(params).map(
                |ReferenceParams {
                     text_document_position,
                     context,
                     ..
                 }| {
                    let request = Self::References {
                        position: text_document_position.position,
                        with_declaration: context.include_declaration,
                    };
                    (text_document_position.text_document.uri, request)
                },
            ),
            PrepareRenameRequest::METHOD => at_position(params, Self::PrepareRename),
            Rename::METHOD => serde_json::from_value(params).map(
                |RenameParams {
                     text_document_position,
                     new_name,
                     ..
                 }| {
                    let request = Self::Rename {
                        position: text_document_position.position,
                        new_name,
                    };
                    (text_document_position.text_document.uri, request)
                },
            ),
            DocumentSymbolRequest::METHOD => of_document(params, Self::Symbols),
            FoldingRangeRequest::METHOD => of_document(params, Self::Folding),
            blocks::METHOD => of_document(params, Self::Blocks),
            CodeActionRequest::METHOD => serde_json::from_value(params).map(
                |CodeActionParams {
                     text_document,
                     range,
                     context,
                     ..
                 }| {
                    let only = context.only;
                    (text_document.uri, Self::CodeActions { range, only })
                },
            ),
            Formatting::METHOD => of_document(params, Self::Formatting(None)),
            RangeFormatting::METHOD => serde_json::from_value(params).map(
                |DocumentRangeFormattingParams {
                     text_document,
                     range,
                     ..
                 }| { (text_document.uri, Self::Formatting(Some(range))) },
            ),
            OnTypeFormatting::METHOD => serde_json::from_value(params).map(
                |DocumentOnTypeFormattingParams {
                     text_document_position,
                     ch,
                     ..
                 }| {
                    let TextDocumentPositionParams {
                        text_document,
                        position,
                    } = text_document_position;
                    let request = Self::OnTypeFormatting {
                        position,
                        typed: ch,
                    };
                    (text_document.uri, request)
                },
            ),
            InlayHintRequest::METHOD => serde_json::from_value(params).map(
                |InlayHintParams {
                     text_document,
                     range,
                     ..
                 }| { (text_document.uri, Self::InlayHints(range)) },
            ),
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
    /// Whether it is opened as plain Markdown, a `.md` file (CompMD only with an `md` key).
    pub(crate) plain_markdown_file: bool,
    /// Its file.
    pub(crate) path: &'a Path,
    /// Its root's project index; `None` where there is none (a test of one document).
    pub(crate) index: Option<&'a ProjectIndex>,
    /// Whether links and headings are the server's, and which of their features (`links`).
    pub(crate) links: LinkSettings,
    /// Its text.
    pub(crate) text: &'a str,
    /// The editor's version of the text.
    pub(crate) version: i32,
    /// How the client counts positions.
    pub(crate) encoding: Encoding,
    /// Where the files the processor names are.
    pub(crate) places: &'a Places,
    /// The components the project's packs export, for auto-import.
    pub(crate) pack_components: &'a [PackComponent],
    /// Its imports of CompMD's own packs its site can't load, when its configuration names no registry for them.
    pub(crate) unregistered: Option<&'a Unregistered>,
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
            let items = asked
                .links
                .completes()
                .then(|| link_completion::link_completions(*position, asked))
                .flatten()
                .or_else(|| hint_at(position).map(|hint| completion::completions(&hint, asked)))
                .unwrap_or_default();
            serde_json::to_value(items)
        }
        DocumentRequest::Hover(position) => serde_json::to_value(
            hint_at(position)
                .and_then(|hint| hover::hover(&hint, text, encoding))
                .or_else(|| {
                    asked
                        .links
                        .enabled
                        .then(|| link_hover::link_hover(*position, asked))
                        .flatten()
                }),
        ),
        DocumentRequest::SignatureHelp(position) => serde_json::to_value(
            processor
                .and_then(|processor| processor.signature_at(asked.offset(*position)))
                .map(|help| signature_help::signature_help(&help)),
        ),
        DocumentRequest::Definition(position) => serde_json::to_value(
            processor
                .and_then(|processor| processor.definition_at(asked.offset(*position)))
                .and_then(|defined| definition::location(&defined, asked))
                .or_else(|| {
                    asked
                        .links
                        .enabled
                        .then(|| link_definition::link_definition(*position, asked))
                        .flatten()
                }),
        ),
        DocumentRequest::References {
            position,
            with_declaration,
        } => serde_json::to_value(if asked.links.enabled {
            references::references(*position, *with_declaration, asked)
        } else {
            Vec::new()
        }),
        DocumentRequest::PrepareRename(position) => {
            serde_json::to_value(rename::prepare_rename(*position, asked))
        }
        DocumentRequest::Rename { position, new_name } => {
            serde_json::to_value(rename::rename(*position, new_name, asked))
        }
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
                .map(|processor| blocks::blocks(processor, asked))
                .unwrap_or_default(),
        ),
        DocumentRequest::Drawing { range, planning } => {
            serde_json::to_value(drawing::plan(*range, planning, asked))
        }
        DocumentRequest::CodeActions { range, only } => {
            serde_json::to_value(code_actions::code_actions(*range, only.as_deref(), asked))
        }
        DocumentRequest::Formatting(range) => {
            serde_json::to_value(formatting::formatting(*range, asked))
        }
        DocumentRequest::OnTypeFormatting { position, typed } => serde_json::to_value(
            on_type_formatting::on_type_formatting(*position, typed, asked),
        ),
        DocumentRequest::InlayHints(range) => {
            serde_json::to_value(inlay_hints::inlay_hints(*range, asked))
        }
    };
    answer.unwrap_or(Value::Null)
}
