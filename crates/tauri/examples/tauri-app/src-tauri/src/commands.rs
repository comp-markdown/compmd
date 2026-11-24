//! The app's commands: its mode, its document, and the check's report.

use std::time::{SystemTime, UNIX_EPOCH};

use compmd::Ambient;
use compmd::policy::Policy;
use serde_json::Value;
use tauri::{AppHandle, Runtime, State};
use tauri_plugin_compmd::CompmdExt;

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

/// The document, processed now, with the packs the app serves, held to the bundle's policy and given its ambient
/// packs: no packs without a bundle.
///
/// # Errors
///
/// The bundle's policy, when the processor can't read it; the page's call fails with it.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri gives a command its arguments by value"
)]
pub fn document<R: Runtime>(app: AppHandle<R>) -> Result<Value, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis());
    let now = i128::try_from(now).unwrap_or(i128::MAX);
    let Some(packs) = app.compmd().packs() else {
        return Ok(crate::document::processed(
            now,
            (),
            &Ambient::default(),
            &Policy::default(),
        ));
    };
    let (policy, ambient) = (
        packs.policy().map_err(|error| error.to_string())?,
        packs.ambient(),
    );
    Ok(crate::document::processed(now, packs, &ambient, &policy))
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
