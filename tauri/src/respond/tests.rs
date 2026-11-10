use tauri::http::{Request, Response, StatusCode};

use crate::frames::Frames;
use crate::frames::tests::{HASH, named, resources};

fn get(path: &str) -> Request<Vec<u8>> {
    Request::get(format!("compmd://localhost{path}"))
        .body(Vec::new())
        .expect("a request")
}

fn header<'a>(response: &'a Response<Vec<u8>>, name: &str) -> &'a str {
    response
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
}

#[test]
fn a_file_of_the_build_is_served_sandboxed_with_its_type() {
    let folder = resources(&named());
    let frames = Frames::read(&folder.0);
    let page = frames.respond(&get(&format!("/frames/{HASH}/strict.html")));
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
    let script = frames.respond(&get(&format!("/frames/{HASH}/chunks/runtime.js?v=1")));
    assert_eq!(script.status(), StatusCode::OK);
    assert_eq!(
        header(&script, "content-type"),
        "text/javascript; charset=utf-8"
    );
}

#[test]
fn nothing_outside_the_build_is_served_and_the_refusal_says_why() {
    let folder = resources(&named());
    let frames = Frames::read(&folder.0);
    for path in [
        "/frames/../secret.txt".to_owned(),
        format!("/frames/{HASH}/../secret.txt"),
        format!("/frames/{HASH}/./strict.html"),
        format!("/frames/{HASH}/chunks//runtime.js"),
        format!("/frames/{HASH}/"),
        format!("/frames/{HASH}/chunks"),
        format!("/frames/{HASH}/missing.js"),
        format!("/frames/{HASH}strict.html"),
        "/frames/0000000000000000/strict.html".to_owned(),
        "/frames/frames.json".to_owned(),
        "/frames.json".to_owned(),
        "/secret.txt".to_owned(),
        format!("/{HASH}/strict.html"),
    ] {
        let refused = frames.respond(&get(&path));
        assert_eq!(refused.status(), StatusCode::NOT_FOUND, "{path}");
        assert_eq!(
            refused.body(),
            b"not a file of the sandbox's frames this app was built with"
        );
        assert_eq!(
            header(&refused, "content-type"),
            "text/plain; charset=utf-8"
        );
        assert_eq!(header(&refused, "access-control-allow-origin"), "*");
    }
}

#[test]
fn only_get_is_answered() {
    let folder = resources(&named());
    let frames = Frames::read(&folder.0);
    let post = Request::post(format!("compmd://localhost/frames/{HASH}/strict.html"))
        .body(Vec::new())
        .expect("a request");
    let refused = frames.respond(&post);
    assert_eq!(refused.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(
        refused.body(),
        b"the sandbox's frames are only read, with GET"
    );
    assert_eq!(header(&refused, "access-control-allow-origin"), "*");
}

#[test]
fn without_a_build_everything_is_refused() {
    let folder = resources("{}");
    let frames = Frames::read(&folder.0);
    let page = frames.respond(&get(&format!("/frames/{HASH}/strict.html")));
    assert_eq!(page.status(), StatusCode::NOT_FOUND);
}
