//! The scheme's handler (docs/tauri-plugin.md, decision 4): the build's files with the headers that sandbox them, and
//! a refusal, saying why, for anything else.

use std::path::{Component, Path};

use tauri::http::header::{self, HeaderValue};
use tauri::http::{Method, Request, Response, StatusCode};

use crate::content_type::content_type;
use crate::frames::{Frames, PREFIX};

/// The headers every file of the frames is served with.
const HEADERS: [(&str, &str); 5] = [
    // Sandboxed even when a frame document is opened outside a frame.
    ("content-security-policy", "sandbox allow-scripts"),
    // A frame's module scripts come from an opaque origin, so they are cross-origin requests.
    (ALLOW_ORIGIN, "*"),
    // A page that checks how its frames are served (an app's self-check) reads these headers.
    ("access-control-expose-headers", "*"),
    ("x-content-type-options", "nosniff"),
    // A path names its build, so what it serves never changes.
    ("cache-control", "max-age=31536000, immutable"),
];

/// Readable from any origin: a file, and a refusal, so a page reads why.
const ALLOW_ORIGIN: &str = "access-control-allow-origin";

/// Why a request that isn't for a file of the build is refused.
const NOT_FRAMES: &str = "not a file of the sandbox's frames this app was built with";

/// Why a request with another method is refused.
const ONLY_GET: &str = "the sandbox's frames are only read, with GET";

impl Frames {
    /// The answer to `request`: the file of the build it names, or a refusal saying why.
    #[must_use]
    pub fn respond(&self, request: &Request<Vec<u8>>) -> Response<Vec<u8>> {
        if request.method() != Method::GET {
            return refused(StatusCode::METHOD_NOT_ALLOWED, ONLY_GET);
        }
        let path = request.uri().path();
        self.file(path).map_or_else(
            || refused(StatusCode::NOT_FOUND, NOT_FRAMES),
            |body| served(Path::new(path), body),
        )
    }

    /// The bytes of the file `path` names in the build, when it names one: `/frames/<hash>/<parts>`, with no empty,
    /// `.` or `..` part.
    fn file(&self, path: &str) -> Option<Vec<u8>> {
        let build = self.build()?;
        let rest = path
            .strip_prefix(PREFIX)?
            .strip_prefix(build.hash.as_str())?
            .strip_prefix('/')?;
        let parts: Vec<&str> = rest.split('/').collect();
        let plain = parts
            .iter()
            .all(|part| !part.is_empty() && *part != "." && *part != "..")
            // A part that is a path of its own (`C:` on Windows) would leave the folder.
            && Path::new(rest)
                .components()
                .all(|component| matches!(component, Component::Normal(_)));
        if plain {
            build.files.read(&parts)
        } else {
            None
        }
    }
}

/// `body`, the file at `path`, with its type and the frames' headers.
fn served(path: &Path, body: Vec<u8>) -> Response<Vec<u8>> {
    let mut response = with_type(StatusCode::OK, content_type(path), body);
    let headers = response.headers_mut();
    for (name, value) in HEADERS {
        headers.insert(name, HeaderValue::from_static(value));
    }
    response
}

/// A refusal with `status`, saying `why` in plain text, to any origin.
fn refused(status: StatusCode, why: &str) -> Response<Vec<u8>> {
    let mut response = with_type(status, "text/plain; charset=utf-8", why.as_bytes().to_vec());
    response
        .headers_mut()
        .insert(ALLOW_ORIGIN, HeaderValue::from_static("*"));
    response
}

/// `body` with `status` and the content type `kind`.
fn with_type(status: StatusCode, kind: &'static str, body: Vec<u8>) -> Response<Vec<u8>> {
    let mut response = Response::new(body);
    *response.status_mut() = status;
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(kind));
    response
}

#[cfg(test)]
mod tests;
