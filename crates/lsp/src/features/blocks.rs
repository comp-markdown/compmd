//! `compmd/blocks` ([the language server], *Blocks, symbols, folding*; [content languages], decision 3): each code
//! block a code renderer renders, with its language, its renderer and the range of its text, for an editor package to
//! hand the requests inside it to the writer's server for that language; and the language's other names, from the
//! manifest of the pack the renderer comes from (`compmd.languages`, the companion §2), so the package finds that
//! server.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#blocks-symbols-folding
//! [content languages]: https://github.com/comp-markdown/compmd/blob/main/docs/content-languages.md

use compmd::Processor;
use compmd::processor::Expanded;
use lsp_types::Range;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Asked;

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
    /// The language's other names, as its pack lists them.
    #[serde(flatten)]
    pub(crate) names: Names,
}

/// The names other tools know a language by (the companion §2): none when its pack lists none.
#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Names {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) aliases: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) language_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) extensions: Vec<String>,
}

/// The rendered blocks of `processor`'s document, the one `asked` is about.
pub(crate) fn blocks(processor: &Processor<Expanded>, asked: &Asked) -> Vec<Block> {
    processor
        .rendered_blocks()
        .into_iter()
        .map(|block| {
            let names = block
                .pack
                .as_ref()
                .and_then(|release| names_in(asked, &release.to_string(), &block.language))
                .unwrap_or_default();
            Block {
                range: asked.encoding.span(asked.text, &block.range),
                language: block.language,
                component: block.component,
                names,
            }
        })
        .collect()
}

/// The names of `language` that the manifest of the pack `release` lists, when the pack is on the disk.
fn names_in(asked: &Asked, release: &str, language: &str) -> Option<Names> {
    let path = asked.places.path_of(&format!("{release}/package.json"))?;
    let manifest: Value = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    let entry = manifest
        .pointer("/compmd/languages")?
        .as_array()?
        .iter()
        .find(|entry| entry.get("id").and_then(Value::as_str) == Some(language))?;
    serde_json::from_value(entry.clone()).ok()
}
