//! The CompMD processor as WebAssembly. A request and its result cross as JSON text (see `request`), and a document
//! kept open for an editor takes edits in UTF-16 positions and gives what changed (see `document`); time zone offsets
//! come from the page's `Intl`, so the module carries no zone data. The npm package `@compmd/processor` wraps it with
//! types.

mod document;
mod request;
mod text;

use compmd::host::Time;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/zones.js")]
extern "C" {
    /// The offset of an IANA zone from UTC at a time in seconds since 1970, in seconds east of UTC; `undefined` for
    /// a zone `Intl` doesn't know.
    #[wasm_bindgen(js_name = zoneOffset)]
    fn zone_offset(zone: &str, seconds: f64) -> Option<f64>;
}

/// The page's time zones, through `Intl`.
struct Intl;

impl Time for Intl {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        reason = "times in CEL's range and offsets in seconds are exact as doubles"
    )]
    fn offset(&self, zone: &str, at: i64) -> Option<i32> {
        zone_offset(zone, at as f64).map(|seconds| seconds as i32)
    }
}

/// Runs stages 1 to 4b on `request`: `{parse, expanded, diagnostics}`.
///
/// # Errors
///
/// A request that isn't one, or whose document isn't one of its files.
#[wasm_bindgen]
pub fn process(request: &str) -> Result<String, JsError> {
    Ok(request::process(request, Rc::new(Intl))?)
}

/// The remote and `host:` sources a render of `request` needs: `{remote, host}`.
///
/// # Errors
///
/// A request that isn't one, or whose document isn't one of its files.
#[wasm_bindgen]
pub fn sources(request: &str) -> Result<String, JsError> {
    Ok(request::sources(request, Rc::new(Intl))?)
}

/// What the frontmatter of `request`'s document gives its render: `{doc, settings, data, inputs}`.
///
/// # Errors
///
/// A request that isn't one, or whose document isn't one of its files.
#[wasm_bindgen]
pub fn frontmatter(request: &str) -> Result<String, JsError> {
    Ok(request::frontmatter(request, Rc::new(Intl))?)
}

/// What a plain CommonMark and GFM renderer shows of `text` (spec §14), as HTML.
#[wasm_bindgen]
#[must_use]
pub fn plain(text: &str) -> String {
    compmd::plain::html(text)
}

/// A document kept open for an editor. After each change, `splice`, `diagnostics` and `components` give what it
/// did, each as JSON text.
#[wasm_bindgen(js_name = Document)]
pub struct Handle(Option<document::Document>);

#[wasm_bindgen(js_class = Document)]
impl Handle {
    /// Processes `request`'s document and keeps it open.
    ///
    /// # Errors
    ///
    /// A request that isn't one, or whose document isn't one of its files.
    #[wasm_bindgen(constructor)]
    pub fn open(request: &str) -> Result<Handle, JsError> {
        Ok(Self(Some(document::Document::open(
            request,
            Rc::new(Intl),
        )?)))
    }

    /// The last change's splice of the expanded tree, `{from, to, nodes, lines, offset, inputs, edit}`; after
    /// opening, the whole tree.
    ///
    /// # Errors
    ///
    /// None in practice: the trees always serialize.
    pub fn splice(&self) -> Result<String, JsError> {
        Ok(self.document()?.splice()?)
    }

    /// Every diagnostic of the document, in stage order.
    ///
    /// # Errors
    ///
    /// None in practice, as `splice`.
    pub fn diagnostics(&self) -> Result<String, JsError> {
        Ok(self.document()?.diagnostics()?)
    }

    /// The host-language components, when the last change changed them (and after opening).
    ///
    /// # Errors
    ///
    /// None in practice, as `splice`.
    pub fn components(&self) -> Result<Option<String>, JsError> {
        Ok(self.document()?.components()?)
    }

    /// The document's inputs after the last change, by name: each one's type, members and value.
    ///
    /// # Errors
    ///
    /// None in practice, as `splice`.
    pub fn inputs(&self) -> Result<String, JsError> {
        Ok(self.document()?.inputs()?)
    }

    /// The text's UTF-16 `start..end` became `text`.
    ///
    /// # Errors
    ///
    /// A call after one that panicked, which left no document.
    pub fn edit(&mut self, start: usize, end: usize, text: String) -> Result<(), JsError> {
        self.change(|document| document.edit(start, end, text))
    }

    /// The remote and `host:` sources a render needs now: `{remote, host}`.
    ///
    /// # Errors
    ///
    /// None in practice, as `splice`.
    pub fn sources(&self) -> Result<String, JsError> {
        Ok(self.document()?.sources()?)
    }

    /// Loads and expands again with the data the host fetched, `{remote, host}`.
    ///
    /// # Errors
    ///
    /// Data that isn't JSON of that shape.
    pub fn reload(&mut self, fetched: &str) -> Result<(), JsError> {
        let data = document::Data::from_json(fetched)?;
        self.change(|document| document.reload(data))
    }
}

impl Handle {
    fn document(&self) -> Result<&document::Document, JsError> {
        self.0.as_ref().ok_or_else(closed)
    }

    fn change(
        &mut self,
        step: impl FnOnce(document::Document) -> document::Document,
    ) -> Result<(), JsError> {
        self.0 = Some(step(self.0.take().ok_or_else(closed)?));
        Ok(())
    }
}

/// A call after one that panicked, which left no document.
fn closed() -> JsError {
    JsError::new("the document was lost to an earlier error")
}
