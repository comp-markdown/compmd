//! The CompMD component reader as WebAssembly (`compmd-components`), a module of its own beside the processor's: only
//! what checks components loads it ([the language server], Q5). `@compmd/processor/components` wraps it with types.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use compmd::Language;
use compmd::analysis::ComponentProps;
use compmd_components::ComponentReader;
use wasm_bindgen::prelude::*;

/// What the component of `language` (`jsx`, `tsx`, `svelte`) whose source is `code` takes, as JSON text
/// (`{props: [{name, type}], rest}`); `undefined` when its props can't be read, or the language takes none.
#[wasm_bindgen(js_name = readProps)]
#[must_use]
pub fn read_props(language: &str, code: &str) -> Option<String> {
    let props = ComponentReader.props(Language::from_name(language)?, code)?;
    serde_json::to_string(&props).ok()
}
