//! A host's component reader, a JavaScript function, as the processor's `ComponentProps` (component analysis, W104).

use compmd::Language;
use compmd::analysis::{ComponentProps, Props};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    /// A component reader: what the component of a language with some code takes, as `Props`' JSON, or `undefined`
    /// when it can't be read. `@compmd/processor/components` gives one.
    #[wasm_bindgen(typescript_type = "(language: string, code: string) => string | undefined")]
    pub type ReadProps;

    /// Calls the reader.
    #[wasm_bindgen(method, structural, js_name = call)]
    fn call(this: &ReadProps, this_arg: &JsValue, language: &str, code: &str) -> Option<String>;
}

/// The host's reader, for the processor.
pub(crate) struct Reader(pub(crate) ReadProps);

impl ComponentProps for Reader {
    fn props(&self, language: Language, code: &str) -> Option<Props> {
        let json = self.0.call(&JsValue::NULL, language.name(), code)?;
        serde_json::from_str(&json).ok()
    }
}
