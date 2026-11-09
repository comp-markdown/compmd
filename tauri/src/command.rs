//! The `base` command (docs/tauri-plugin.md, decision 5): the page asks for the frames' `base`, and never builds it.

use tauri::{AppHandle, Runtime, command};

use crate::CompmdExt;

/// The frames' `base` for `@compmd/sandbox`'s `createSandbox`, or `null` when the app was built without its frames.
#[command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri gives a command its arguments by value"
)]
pub fn base<R: Runtime>(app: AppHandle<R>) -> Option<String> {
    app.compmd().base()
}
