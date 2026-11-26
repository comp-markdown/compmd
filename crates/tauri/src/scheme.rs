//! Where a webview's requests on the scheme come in. Only a webview calls it, so the unit tests can't: the example
//! app's check covers it (docs/tauri-plugin.md, decision 9), and coverage leaves this file out. What it calls,
//! `Compmd::respond`, is tested in full.

use std::thread;

use tauri::http::Request;
use tauri::{Runtime, UriSchemeContext, UriSchemeResponder};

use crate::CompmdExt;
use crate::compmd::REGISTRY;

/// Answers a webview's `request` on the scheme (`Compmd::respond`): at once, but under `registry/`, which waits on a
/// registry and is answered on a thread of its own.
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri gives a scheme's handler its arguments by value"
)]
pub fn serve<R: Runtime>(
    context: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app = context.app_handle().clone();
    if !request.uri().path().starts_with(REGISTRY) {
        return responder.respond(app.compmd().respond(&request));
    }
    thread::spawn(move || responder.respond(app.compmd().respond(&request)));
}
