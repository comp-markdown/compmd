use compmd_host::Bundle;
use tauri::http::{Request, StatusCode};
use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};

use super::*;

const HASH: &str = "1df336e75685de8c";

/// A bundle built for `source`, with a frame document and a pack's file.
pub fn bundle(source: Option<&str>) -> Bundle {
    let source = source.map_or_else(String::new, |source| format!(r#", "source": "{source}""#));
    let manifest = format!(
        r#"{{"format": 1, "frames": {{"hash": "{HASH}"}}{source}, "packs": [{{"pack": "@kit/bars", "version": "1.0.0", "trust": "strict"}}]}}"#
    );
    Bundle::from_files([
        ("manifest.json".to_owned(), manifest.into_bytes()),
        (
            format!("frames/{HASH}/strict.html"),
            b"<!doctype html>".to_vec(),
        ),
        (
            "packs/@kit/bars@1.0.0/package.json".to_owned(),
            br#"{"name": "@kit/bars"}"#.to_vec(),
        ),
    ])
    .expect("a bundle")
}

/// An app on Tauri's mock runtime with `plugin`.
fn app(plugin: TauriPlugin<MockRuntime>) -> tauri::App<MockRuntime> {
    mock_builder()
        .plugin(plugin)
        .build(mock_context(noop_assets()))
        .expect("the app builds")
}

/// A webview's `GET` of `path` on the scheme.
fn get(path: &str) -> Request<Vec<u8>> {
    Request::get(format!("compmd://localhost{path}"))
        .body(Vec::new())
        .expect("a request")
}

#[test]
fn the_bundle_is_served_on_its_scheme_through_the_contract() {
    let app = app(init(bundle(Some("compmd://localhost/"))));
    let compmd = app.compmd();
    assert_eq!(compmd.origin(), Some(origin::origin_of("compmd").as_str()));
    let frame = compmd.respond(&get(&format!("/frames/{HASH}/strict.html")));
    assert_eq!(frame.status(), StatusCode::OK);
    assert_eq!(frame.body(), b"<!doctype html>");
    assert_eq!(
        frame.headers()["content-security-policy"],
        "sandbox allow-scripts"
    );
    let pack = compmd.respond(&get("/packs/@kit/bars@1.0.0/package.json"));
    assert_eq!(pack.status(), StatusCode::OK);
    assert_eq!(
        compmd.respond(&get("/manifest.json")).status(),
        StatusCode::NOT_FOUND
    );
}

#[test]
fn the_store_is_in_the_apps_data_folder() {
    let app = app(init(bundle(Some("compmd://localhost/"))));
    let releases = app
        .compmd()
        .store()
        .releases(&"@kit/other".parse().expect("a name"));
    assert_eq!(releases, []);
}

#[test]
fn without_a_data_folder_the_store_holds_nothing() {
    let store = store_in(None);
    assert_eq!(store.releases(&"@kit/bars".parse().expect("a name")), []);
}

#[test]
fn an_app_without_a_bundle_serves_nothing_and_says_why() {
    let app = app(init(None));
    let compmd = app.compmd();
    assert!(compmd.bundle().is_none());
    assert_eq!(compmd.origin(), None);
    let refused = compmd.respond(&get(&format!("/frames/{HASH}/strict.html")));
    assert_eq!(refused.status(), StatusCode::NOT_FOUND);
    assert!(String::from_utf8_lossy(refused.body()).contains("@compmd/vite"));
}

#[cfg(feature = "processor")]
#[test]
fn the_processor_gets_the_packs_the_frames_load() {
    let app = app(init(bundle(Some("compmd://localhost/"))));
    assert!(app.compmd().packs().is_some());
    assert!(self::app(init(None)).compmd().packs().is_none());
}

#[test]
fn the_page_gets_the_origin_from_a_script_as_createsandbox_reads_it() {
    assert_eq!(
        origin::init_script("compmd://localhost"),
        "globalThis.__COMPMD__ = Object.freeze({ origin: \"compmd://localhost\" });"
    );
}

#[test]
fn the_origin_is_the_schemes_as_the_platform_serves_it() {
    assert_eq!(origin::origin_on("frames", true), "http://frames.localhost");
    assert_eq!(origin::origin_on("frames", false), "frames://localhost");
    let over_http = cfg!(any(windows, target_os = "android"));
    assert_eq!(
        origin::origin_of("frames"),
        origin::origin_on("frames", over_http)
    );
}

#[test]
fn a_schemes_name_is_lowercase_letters_digits_and_plus_minus_dot() {
    for good in ["compmd", "my-frames", "app+frames", "a.b", "x1"] {
        assert!(origin::is_scheme(good), "{good}");
    }
    for bad in ["", "1app", "App", "my frames", "a/b", "a:b", "-a"] {
        assert!(!origin::is_scheme(bad), "{bad}");
    }
}

#[test]
#[should_panic(expected = "built without a `source`")]
fn a_bundle_built_for_http_is_a_mistake_in_the_app() {
    drop(init::<MockRuntime>(bundle(None)));
}

#[test]
#[should_panic(expected = "isn't a scheme's name")]
fn a_bad_scheme_is_a_mistake_in_the_app() {
    drop(init::<MockRuntime>(bundle(Some("Not+A://localhost/"))));
}

#[test]
fn the_embedded_files_are_a_bundle_or_none() {
    static FILES: &[(&str, &[u8])] = &[(
        "manifest.json",
        br#"{"format": 1, "frames": {"hash": "1df336e75685de8c"}, "source": "compmd://localhost/"}"#,
    )];
    assert!(embedded(Some(FILES)).is_some());
    assert!(embedded(None).is_none());
    assert!(embedded(Some(&[])).is_none(), "no manifest");
}
