//! Where a webview's requests on the scheme come in. Only a webview calls it, so the unit tests can't: the example
//! app's check covers it (docs/tauri-plugin.md, decision 9), and coverage leaves this file out. What it calls,
//! `Frames::respond`, is tested in full.

use tauri::http::{Request, Response};
use tauri::{Runtime, UriSchemeContext};

use crate::CompmdExt;

/// The answer to a webview's `request` on the scheme: the frames the app serves (`Frames::respond`).
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri gives a scheme's handler its arguments by value"
)]
pub fn serve<R: Runtime>(
    context: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    context.app_handle().compmd().frames.respond(&request)
}
