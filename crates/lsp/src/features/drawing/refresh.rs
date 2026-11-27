//! `compmd/drawing/refresh` ([a document drawn in the editor], decision 4): the server asks the client to ask for the
//! plans of the documents it draws again, as `workspace/inlayHint/refresh` does; no params, no result. It is sent only
//! to a client that declared `refreshSupport`.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

/// The request.
pub(crate) enum Refresh {}

impl lsp_types::request::Request for Refresh {
    type Params = ();
    type Result = ();
    const METHOD: &'static str = "compmd/drawing/refresh";
}
