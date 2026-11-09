//! The app's commands: its mode, its document, and the check's report.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tauri::{AppHandle, Runtime, State};

use crate::mode::Mode;

/// How the app runs, as the page reads it.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri gives a command its arguments by value"
)]
pub fn mode(mode: State<'_, Mode>) -> &'static str {
    mode.name()
}

/// The document, processed now.
#[tauri::command]
#[must_use]
pub fn document() -> Value {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis());
    crate::document::processed(i128::try_from(now).unwrap_or(i128::MAX))
}

/// The check's outcome: printed, then the app exits `0` when it passed and `1` when it didn't.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri gives a command its arguments by value"
)]
pub fn report<R: Runtime>(app: AppHandle<R>, ok: bool, message: String) {
    if ok {
        println!("compmd-tauri-example: passed: {message}");
    } else {
        eprintln!("compmd-tauri-example: failed: {message}");
    }
    app.exit(i32::from(!ok));
}
