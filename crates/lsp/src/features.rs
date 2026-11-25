//! What a client asks about a document ([the language server], *The features*): each a lookup in the document's
//! processor, answered by the document's worker, never another run.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#the-features

pub(crate) mod semantic_tokens;

use compmd::Processor;
use compmd::processor::Expanded;
use lsp_types::Range;
use serde_json::Value;

use crate::encoding::Encoding;

/// A request about one document.
#[derive(Debug, PartialEq)]
pub(crate) enum DocumentRequest {
    /// Its semantic tokens: all of them, or those in a range.
    SemanticTokens(Option<Range>),
}

/// The answer to `request` about the document whose text is `text`, from its `processor`; `None` when the document
/// couldn't be processed (its project can't be read), which answers as nothing.
pub(crate) fn answer(
    request: &DocumentRequest,
    processor: Option<&Processor<Expanded>>,
    text: &str,
    encoding: Encoding,
) -> Value {
    match request {
        DocumentRequest::SemanticTokens(range) => {
            let tokens = processor.map(Processor::tokens).unwrap_or_default();
            serde_json::to_value(semantic_tokens::encode(&tokens, text, encoding, *range))
                .unwrap_or(Value::Null)
        }
    }
}
