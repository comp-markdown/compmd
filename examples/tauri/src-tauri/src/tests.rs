//! The app's wiring on Tauri's mock runtime, with its own capabilities: the app's page reaches its commands, and a
//! page on the frames' scheme reaches none (docs/tauri-plugin.md, decision 8); the plugin serves the bundle.

use serde_json::{Value, json};
use tauri::ipc::{CallbackFn, InvokeBody};
use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response};
use tauri::webview::InvokeRequest;
use tauri::{WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_compmd::CompmdExt;

use super::*;

/// The app's window on the mock runtime, in `mode`.
fn window(mode: Mode) -> (tauri::App<MockRuntime>, WebviewWindow<MockRuntime>) {
    let app = app(tauri::test::mock_builder(), mode)
        .build(tauri::generate_context!(test = true))
        .expect("the app builds");
    let window = WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
        .build()
        .expect("the window opens");
    (app, window)
}

/// Calls `command` with `args` from a page at `url`, as the app's own page does: with Tauri's invoke key.
fn invoke(
    window: &WebviewWindow<MockRuntime>,
    url: &str,
    command: &str,
    args: Value,
) -> Result<Value, Value> {
    invoke_with(window, url, command, args, INVOKE_KEY)
}

/// Calls `command` with `args` from a page at `url`, with `key` as the invoke key.
fn invoke_with(
    window: &WebviewWindow<MockRuntime>,
    url: &str,
    command: &str,
    args: Value,
    key: &str,
) -> Result<Value, Value> {
    let request = InvokeRequest {
        cmd: command.to_owned(),
        callback: CallbackFn(0),
        error: CallbackFn(1),
        url: url.parse().expect("a URL"),
        body: InvokeBody::Json(args),
        headers: tauri::http::HeaderMap::default(),
        invoke_key: key.to_owned(),
    };
    get_ipc_response(window, request).map(|body| body.deserialize().expect("JSON"))
}

#[test]
fn the_apps_page_reaches_its_commands_and_the_document_its_packs() {
    let (app, window) = window(Mode::Check);
    assert_eq!(
        invoke(&window, "tauri://localhost", "mode", json!({})),
        Ok(json!("check"))
    );
    // Processed over the packs of the bundle the build embedded, as the frames load them.
    let processed = invoke(&window, "tauri://localhost", "document", json!({})).expect("processed");
    assert_eq!(
        processed["diagnostics"],
        json!([]),
        "{}",
        processed["diagnostics"]
    );
    let compmd = app.compmd();
    assert!(
        compmd.bundle().is_some(),
        "the build embedded no bundle: build the page first (`pnpm build`)"
    );
    assert!(
        compmd
            .origin()
            .is_some_and(|origin| origin.contains("compmd"))
    );
}

#[test]
fn the_broken_mode_serves_the_broken_bundle() {
    let (app, _window) = window(Mode::CheckBroken);
    let bundle = app
        .compmd()
        .bundle()
        .expect("the broken bundle: build the page first (`pnpm build`)");
    let runtime = format!("/frames/{}/runtime.js", bundle.manifest().frames.hash);
    let request = tauri::http::Request::get(runtime)
        .body(Vec::<u8>::new())
        .expect("a request");
    assert_eq!(
        app.compmd().respond(&request).status(),
        tauri::http::StatusCode::NOT_FOUND
    );
}

#[test]
fn a_frame_without_the_invoke_key_reaches_no_command() {
    let (_app, window) = window(Mode::Check);
    // Tauri counts the frames' scheme as one of the app's own, so the key is what a frame lacks: Tauri injects it
    // into the main frame only (docs/tauri-plugin.md, decision 8).
    for origin in [
        "compmd://localhost/frames/1df336e75685de8c/strict.html",
        "http://compmd.localhost/frames/1df336e75685de8c/strict.html",
        "tauri://localhost",
    ] {
        for command in ["mode", "document", "report"] {
            for key in ["", "a guess"] {
                // Tauri drops a call with the wrong key without answering it, which the mock's helper takes as a
                // panic: either way, the command didn't run.
                let call = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    invoke_with(
                        &window,
                        origin,
                        command,
                        json!({ "ok": true, "message": "" }),
                        key,
                    )
                }));
                assert!(
                    !matches!(call, Ok(Ok(_))),
                    "{command} from {origin} with the key {key:?}: {call:?}"
                );
            }
        }
    }
}
