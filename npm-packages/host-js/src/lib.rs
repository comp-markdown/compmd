//! `compmd-host`'s serving decision as WebAssembly, for `@compmd/host` ([hosting], decision 8): which file a request
//! names, with the headers it is served with, or why it is refused. The package reads the file itself, from its bundle
//! or store, over the Fetch API; Rust decides, so `compmd-host` and `@compmd/host` answer alike.
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use std::collections::BTreeMap;

use compmd_host::{Refusal, Route, refusal_headers, served_headers};
use serde::Serialize;
use wasm_bindgen::prelude::wasm_bindgen;

/// What `@compmd/host` does with a request, as JSON.
#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum Answer {
    /// Serve the frames' file at `path`, with `headers`.
    Frame {
        path: String,
        headers: BTreeMap<&'static str, &'static str>,
    },
    /// Serve the file at `path` of the pack release `release`, from the bundle or the store, with `headers`.
    Pack {
        release: String,
        path: String,
        headers: BTreeMap<&'static str, &'static str>,
    },
    /// Refuse it with `status`, saying `why`, with `headers`.
    Refused {
        status: u16,
        why: &'static str,
        headers: BTreeMap<&'static str, &'static str>,
    },
}

impl From<Route> for Answer {
    fn from(route: Route) -> Self {
        match route {
            Route::Frame(path) => Self::Frame {
                headers: served_headers(path.as_str()).into_iter().collect(),
                path: path.as_str().to_string(),
            },
            Route::Pack(place) => Self::Pack {
                headers: served_headers(place.path.as_str()).into_iter().collect(),
                release: place.release.to_string(),
                path: place.path.as_str().to_string(),
            },
            Route::Refused(refusal) => refused(refusal),
        }
    }
}

/// `refusal` as an answer.
fn refused(refusal: Refusal) -> Answer {
    Answer::Refused {
        status: refusal.status.as_u16(),
        why: refusal.why,
        headers: refusal_headers().into_iter().collect(),
    }
}

/// `answer` as JSON; an answer always serializes, its keys and values being strings and numbers.
fn json(answer: &Answer) -> String {
    serde_json::to_string(answer).unwrap_or_default()
}

/// What to do with a `method` request for `path`, from a bundle whose frames' hash is `hash`, as JSON:
/// `{"kind": "frame", "path", "headers"}`, `{"kind": "pack", "release", "path", "headers"}` or
/// `{"kind": "refused", "status", "why", "headers"}`.
#[wasm_bindgen]
#[must_use]
pub fn route(method: &str, path: &str, hash: &str) -> String {
    let method = http::Method::from_bytes(method.as_bytes()).unwrap_or(http::Method::POST);
    json(&compmd_host::route(&method, path, hash).into())
}

/// The refusal of a request for a file the bundle and the store don't have, as JSON, as [`route`] gives one.
#[wasm_bindgen(js_name = notServed)]
#[must_use]
pub fn not_served() -> String {
    json(&refused(Refusal::NOT_SERVED))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn parsed(text: &str) -> Value {
        serde_json::from_str(text).unwrap_or(Value::Null)
    }

    #[test]
    fn a_frame_and_a_pack_file_with_their_headers() {
        let frame = parsed(&route(
            "GET",
            "/frames/1df336e75685de8c/strict.html",
            "1df336e75685de8c",
        ));
        assert_eq!(frame["kind"], json!("frame"));
        assert_eq!(frame["path"], json!("strict.html"));
        assert_eq!(
            frame["headers"]["content-type"],
            json!("text/html; charset=utf-8")
        );
        assert_eq!(
            frame["headers"]["content-security-policy"],
            json!("sandbox allow-scripts")
        );
        let pack = parsed(&route(
            "GET",
            "/packs/@kit/mermaid@1.2.0/dist/mermaid.js",
            "x",
        ));
        assert_eq!(pack["kind"], json!("pack"));
        assert_eq!(pack["release"], json!("@kit/mermaid@1.2.0"));
        assert_eq!(pack["path"], json!("dist/mermaid.js"));
    }

    #[test]
    fn refusals_say_why() {
        let other = parsed(&route("POST", "/frames/h/strict.html", "h"));
        assert_eq!(other["status"], json!(405));
        let unknown = parsed(&route("BAD METHOD", "/frames/h/strict.html", "h"));
        assert_eq!(unknown["status"], json!(405));
        let elsewhere = parsed(&route("GET", "/elsewhere", "h"));
        assert_eq!(elsewhere, parsed(&not_served()));
        assert_eq!(elsewhere["status"], json!(404));
        assert_eq!(
            elsewhere["headers"]["access-control-allow-origin"],
            json!("*")
        );
    }
}
