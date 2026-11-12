use compmd_pack::PackName;
use http::{Request, Response, StatusCode};

use super::*;
use crate::bundle::tests::{HASH, bundle};
use crate::store::{NoStore, StoredRelease};

/// A store holding `@kit/charts` 2.0.0 with `index.js`.
struct Charts;

impl Store for Charts {
    fn releases(&self, _pack: &PackName) -> Vec<StoredRelease> {
        Vec::new()
    }

    fn file(&self, place: &Place) -> Option<Vec<u8>> {
        (place.to_string() == "@kit/charts@2.0.0/index.js").then(|| b"export default 2".to_vec())
    }
}

#[test]
fn the_store_in_these_tests_lists_nothing() {
    assert_eq!(Charts.releases(&"@kit/charts".parse().expect("a name")), []);
}

fn get(path: &str) -> Request<()> {
    Request::get(path).body(()).expect("a request")
}

fn header<'a>(response: &'a Response<Vec<u8>>, name: &str) -> &'a str {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
}

#[test]
fn a_frames_file_is_served_sandboxed_with_its_type() {
    let bundle = bundle();
    let page = respond(
        &bundle,
        &NoStore,
        &get(&format!("/frames/{HASH}/strict.html")),
    );
    assert_eq!(page.status(), StatusCode::OK);
    assert_eq!(page.body(), b"<!doctype html>");
    assert_eq!(header(&page, "content-type"), "text/html; charset=utf-8");
    assert_eq!(
        header(&page, "content-security-policy"),
        "sandbox allow-scripts"
    );
    assert_eq!(header(&page, "access-control-allow-origin"), "*");
    assert_eq!(header(&page, "access-control-expose-headers"), "*");
    assert_eq!(header(&page, "x-content-type-options"), "nosniff");
    assert_eq!(
        header(&page, "cache-control"),
        "max-age=31536000, immutable"
    );
    let script = respond(
        &bundle,
        &NoStore,
        &get(&format!("/frames/{HASH}/chunks/runtime.js?v=1")),
    );
    assert_eq!(script.status(), StatusCode::OK);
    assert_eq!(
        header(&script, "content-type"),
        "text/javascript; charset=utf-8"
    );
}

#[test]
fn a_packs_file_comes_from_the_bundle_or_the_store() {
    let bundle = bundle();
    let bundled = respond(
        &bundle,
        &NoStore,
        &get("/packs/@kit/mermaid@1.2.0/dist/mermaid.js"),
    );
    assert_eq!(bundled.status(), StatusCode::OK);
    assert_eq!(bundled.body(), b"export default 1");
    assert_eq!(
        header(&bundled, "content-type"),
        "text/javascript; charset=utf-8"
    );
    assert_eq!(
        header(&bundled, "cache-control"),
        "max-age=31536000, immutable"
    );
    let stored = respond(&bundle, &Charts, &get("/packs/@kit/charts@2.0.0/index.js"));
    assert_eq!(stored.status(), StatusCode::OK);
    assert_eq!(stored.body(), b"export default 2");
    let held_by_neither = respond(&bundle, &NoStore, &get("/packs/@kit/charts@2.0.0/index.js"));
    assert_eq!(held_by_neither.status(), StatusCode::NOT_FOUND);
}

#[test]
fn nothing_else_is_served_and_the_refusal_says_why() {
    let bundle = bundle();
    for path in [
        "/frames/../secret.txt".to_owned(),
        format!("/frames/{HASH}/../secret.txt"),
        format!("/frames/{HASH}/./strict.html"),
        format!("/frames/{HASH}/chunks//runtime.js"),
        format!("/frames/{HASH}/"),
        format!("/frames/{HASH}"),
        format!("/frames/{HASH}/chunks"),
        format!("/frames/{HASH}/missing.js"),
        format!("/frames/{HASH}strict.html"),
        "/frames/0000000000000000/strict.html".to_owned(),
        "/manifest.json".to_owned(),
        "/secret.txt".to_owned(),
        format!("/{HASH}/strict.html"),
        "/packs/@kit/other@1.0.0/index.js".to_owned(),
        "/packs/@kit/mermaid@1.2.0/../../secret.txt".to_owned(),
        "/packs/@kit/mermaid@1.2.0".to_owned(),
        "/packs/kit/mermaid@1.2.0/package.json".to_owned(),
        "/".to_owned(),
    ] {
        let refused = respond(&bundle, &NoStore, &get(&path));
        assert_eq!(refused.status(), StatusCode::NOT_FOUND, "{path}");
        assert_eq!(
            refused.body(),
            b"not a file of the frames or packs this host serves"
        );
        assert_eq!(
            header(&refused, "content-type"),
            "text/plain; charset=utf-8"
        );
        assert_eq!(header(&refused, "access-control-allow-origin"), "*");
        assert_eq!(
            header(&refused, "cache-control"),
            "",
            "a refusal isn't cached"
        );
    }
}

#[test]
fn a_path_without_its_leading_slash_is_refused() {
    let bundle = bundle();
    let request = Request::get("compmd://localhost")
        .body(())
        .expect("a request");
    assert_eq!(
        respond(&bundle, &NoStore, &request).status(),
        StatusCode::NOT_FOUND
    );
}

#[test]
fn only_get_is_answered() {
    let bundle = bundle();
    let post = Request::post(format!("/frames/{HASH}/strict.html"))
        .body(())
        .expect("a request");
    let refused = respond(&bundle, &NoStore, &post);
    assert_eq!(refused.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(
        refused.body(),
        b"the frames and packs are only read, with GET"
    );
    assert_eq!(header(&refused, "access-control-allow-origin"), "*");
}
