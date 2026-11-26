//! The preview's server over HTTP on `127.0.0.1`, as a browser asks it: the page under its token, the frames by the
//! serving contract, the `Host` check, the events, `request.json` and `registry/`.

use std::time::Duration;

use compmd_host::registry::{Client, LocalRegistry, Options, ServedPack};
use compmd_host::{Bundle, MANIFEST, NoStore};
use serde_json::json;

use super::testing::{ask, get, listen, next_event};
use super::*;

const HASH: &str = "1df336e75685de8c";

/// A preview with a page (`index.html`, `assets/index.js`) and a bundle of one frame, its registries `proxy`'s.
fn preview(proxy: Option<Proxy>) -> Preview {
    let manifest = format!(r#"{{"format": 1, "frames": {{"hash": "{HASH}"}}}}"#);
    let bundle = Bundle::from_files([
        (MANIFEST.to_owned(), manifest.into_bytes()),
        (
            format!("frames/{HASH}/strict.html"),
            b"<!doctype html>".to_vec(),
        ),
    ])
    .expect("a bundle");
    let page = [
        (
            "index.html",
            b"<!doctype html><html><head><title>Preview</title></head></html>".to_vec(),
        ),
        ("assets/index.js", b"export {};".to_vec()),
    ];
    Preview::start(PreviewFiles::new(page, bundle), Box::new(NoStore), proxy).expect("started")
}

/// The page's port and the path under its token.
fn page_of(preview: &Preview) -> (u16, String) {
    testing::page_of(&preview.url())
}

/// The frames' port, from the origin the page is given.
fn frames_port(preview: &Preview) -> u16 {
    let (port, token) = page_of(preview);
    let (_, _, page) = get(port, &token);
    let origin = page
        .split("origin: \"http://127.0.0.1:")
        .nth(1)
        .expect("the frames' origin");
    origin
        .split('"')
        .next()
        .and_then(|port| port.parse().ok())
        .expect("a port")
}

#[test]
fn the_page_is_under_its_token_and_given_the_frames_origin() {
    let preview = preview(None);
    let (port, token) = page_of(&preview);
    assert_eq!(token.len(), "/".len() + 32 + "/".len());
    let (status, head, page) = get(port, &token);
    assert_eq!(status, 200);
    assert!(head.contains("text/html"), "{head}");
    assert!(
        page.starts_with("<!doctype html><html><head><script>globalThis.__COMPMD__"),
        "{page}"
    );
    assert_ne!(frames_port(&preview), port, "an origin of their own");
    let (status, head, script) = get(port, &format!("{token}assets/index.js"));
    assert_eq!((status, script.as_str()), (200, "export {};"));
    assert!(head.contains("text/javascript"), "{head}");
    for elsewhere in [
        "/",
        "/assets/index.js",
        &format!("{token}none.js"),
        &format!("{token}../x"),
    ] {
        assert_eq!(get(port, elsewhere).0, 404, "{elsewhere}");
    }
    assert_eq!(
        ask(port, "POST", &token, Some(&format!("127.0.0.1:{port}"))).0,
        405
    );
}

#[test]
fn a_request_whose_host_isnt_the_servers_own_is_refused() {
    let preview = preview(None);
    let (port, token) = page_of(&preview);
    let frames = frames_port(&preview);
    for host in [
        Some(format!("localhost:{port}")),
        Some("evil.example".to_owned()),
        None,
    ] {
        assert_eq!(ask(port, "GET", &token, host.as_deref()).0, 403, "{host:?}");
    }
    let frame = format!("/frames/{HASH}/strict.html");
    assert_eq!(
        ask(frames, "GET", &frame, Some(&format!("127.0.0.1:{port}"))).0,
        403,
        "the page's host"
    );
}

#[test]
fn the_frames_are_served_by_the_contract_on_their_port() {
    let preview = preview(None);
    let frames = frames_port(&preview);
    let (status, head, body) = get(frames, &format!("/frames/{HASH}/strict.html"));
    assert_eq!((status, body.as_str()), (200, "<!doctype html>"));
    assert!(
        head.to_ascii_lowercase()
            .contains("content-security-policy"),
        "{head}"
    );
    assert_eq!(get(frames, "/frames/other/strict.html").0, 404);
    assert_eq!(
        get(frames, "/packs/@kit/notes@1.0.0/index.comp.md").0,
        404,
        "no store"
    );
}

#[test]
fn the_request_is_served_once_given() {
    let preview = preview(None);
    let (port, token) = page_of(&preview);
    let path = format!("{token}request.json");
    assert_eq!(get(port, &path).0, 503, "not yet");
    preview.set_request(&json!({ "path": "notes.comp.md", "files": {} }));
    let (status, head, body) = get(port, &path);
    assert_eq!(status, 200);
    assert!(head.contains("application/json"), "{head}");
    assert_eq!(
        serde_json::from_str::<Value>(&body).expect("JSON")["path"],
        "notes.comp.md"
    );
}

#[test]
fn each_change_and_a_new_request_are_events_and_a_new_page_gets_the_last_change() {
    let preview = preview(None);
    let (port, token) = page_of(&preview);
    let mut first = listen(port, &token);
    preview.change("# Harbour notes\n", 1);
    assert_eq!(
        next_event(&mut first),
        (
            "change".to_owned(),
            json!({ "text": "# Harbour notes\n", "line": 1 })
        )
    );
    preview.set_request(&json!({ "path": "notes.comp.md" }));
    preview.set_request(&json!({ "path": "notes.comp.md" }));
    preview.set_request(&json!({ "path": "notes.comp.md", "files": { "tide.csv": "level\n2\n" } }));
    assert_eq!(
        next_event(&mut first),
        ("reload".to_owned(), json!({})),
        "once, when it changed"
    );
    let mut late = listen(port, &token);
    assert_eq!(next_event(&mut late).1["text"], "# Harbour notes\n");
}

#[test]
fn the_registries_are_passed_on_under_registry() {
    let without = preview(None);
    let (port, token) = page_of(&without);
    assert_eq!(
        get(port, &format!("{token}registry/@kit%2fnotes")).0,
        404,
        "no registry named"
    );
    let notes = ServedPack::new("@kit/notes").version("1.0.0", &[("package.json", "{}")]);
    let registry = LocalRegistry::serve(&[notes]).expect("served");
    let options = Options::new([("@kit".to_owned(), registry.url().to_owned())].into());
    let with = preview(Some(Proxy::new(
        options,
        Client::new(Duration::from_secs(10)),
    )));
    let (port, token) = page_of(&with);
    let (status, _, body) = get(port, &format!("{token}registry/@kit%2fnotes"));
    assert_eq!(status, 200, "{body}");
    let metadata: Value = serde_json::from_str(&body).expect("JSON");
    let tarball = metadata["versions"]["1.0.0"]["dist"]["tarball"]
        .as_str()
        .unwrap_or_default();
    assert_eq!(
        tarball,
        format!("http://127.0.0.1:{port}{token}registry/-/tarball/@kit/notes/1.0.0")
    );
    let (status, _, _) = get(port, &format!("{token}registry/-/tarball/@kit/notes/1.0.0"));
    assert_eq!(status, 200);
}
