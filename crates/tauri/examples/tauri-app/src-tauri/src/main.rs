//! The smallest Tauri app showing a CompMD document with `tauri-plugin-compmd` (docs/tauri-plugin.md, decision 9):
//! `cargo tauri dev` shows it; `--check` and `--check-broken` check it in the webview and exit `0` or `1`.

mod commands;
mod document;
mod mode;

use std::thread;
use std::time::Duration;

use mode::Mode;

/// How long a check may take before the app gives up on the page's report, and fails.
const CHECK_TIMEOUT: Duration = Duration::from_secs(90);

fn main() {
    let mode = Mode::from_args(std::env::args());
    app(tauri::Builder::default(), mode)
        .setup(move |app| {
            if mode.checks() {
                let handle = app.handle().clone();
                thread::spawn(move || {
                    thread::sleep(CHECK_TIMEOUT);
                    eprintln!("compmd-tauri-example: failed: the page didn't report within {CHECK_TIMEOUT:?}");
                    handle.exit(2);
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("the app runs");
}

/// The app in `mode`, on `builder`: the plugin, as an app adds it, and the example's commands.
fn app<R: tauri::Runtime>(builder: tauri::Builder<R>, mode: Mode) -> tauri::Builder<R> {
    builder
        .plugin(mode.plugin())
        .manage(mode)
        .invoke_handler(tauri::generate_handler![
            commands::mode,
            commands::document,
            commands::report
        ])
}

#[cfg(test)]
mod tests;
