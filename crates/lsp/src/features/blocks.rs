//! `compmd/blocks` ([the language server], *Blocks, symbols, folding*; [content languages], decision 3): each code
//! block a code renderer renders, with its language, its renderer and the range of its text, for an editor package to
//! hand the requests inside it to the writer's server for that language.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#blocks-symbols-folding
//! [content languages]: https://github.com/comp-markdown/compmd/blob/main/docs/content-languages.md

use compmd::Processor;
use compmd::processor::Expanded;
use lsp_types::Range;
use serde::Serialize;

use crate::encoding::Encoding;

/// The request's method.
pub(crate) const METHOD: &str = "compmd/blocks";

/// A rendered block, as the request gives it.
#[derive(Debug, PartialEq, Serialize)]
pub(crate) struct Block {
    /// Its language, as its fence writes it (`mermaid`).
    pub(crate) language: String,
    /// The code renderer that renders it, by the name the document gives it.
    pub(crate) component: String,
    /// Where its text is.
    pub(crate) range: Range,
}

/// The rendered blocks of `processor`'s document, whose text is `text`.
pub(crate) fn blocks(
    processor: &Processor<Expanded>,
    text: &str,
    encoding: Encoding,
) -> Vec<Block> {
    processor
        .rendered_blocks()
        .into_iter()
        .map(|block| Block {
            range: encoding.span(text, &block.range),
            language: block.language,
            component: block.component,
        })
        .collect()
}
