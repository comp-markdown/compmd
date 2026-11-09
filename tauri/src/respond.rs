//! The scheme's handler (docs/tauri-plugin.md, decision 4): the build's files with the headers that sandbox them, and
//! a refusal, saying why, for anything else.

use std::fs;
use std::path::{Component, Path, PathBuf};

use tauri::http::header::{self, HeaderValue};
use tauri::http::{Method, Request, Response, StatusCode};

use crate::content_type::content_type;
use crate::frames::{Frames, PREFIX};

/// The headers every file of the frames is served with.
const HEADERS: [(&str, &str); 4] = [
    // Sandboxed even when a frame document is opened outside a frame.
    ("content-security-policy", "sandbox allow-scripts"),
    // A frame's module scripts come from an opaque origin, so they are cross-origin requests.
    ("access-control-allow-origin", "*"),
    ("x-content-type-options", "nosniff"),
    // A path names its build, so what it serves never changes.
    ("cache-control", "max-age=31536000, immutable"),
];

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
        self.file(request.uri().path())
            .and_then(|path| Some((fs::read(&path).ok()?, path)))
            .map_or_else(
                || refused(StatusCode::NOT_FOUND, NOT_FRAMES),
                |(body, path)| served(&path, body),
            )
    }

    /// The file `path` names in the build, when it names one: `/frames/<hash>/<parts>`, with no empty, `.` or `..`
    /// part.
    fn file(&self, path: &str) -> Option<PathBuf> {
        let build = self.build()?;
        let rest = path
            .strip_prefix(PREFIX)?
            .strip_prefix(build.hash.as_str())?
            .strip_prefix('/')?;
        let parts: Vec<&str> = rest.split('/').collect();
        if parts
            .iter()
            .any(|part| part.is_empty() || *part == "." || *part == "..")
        {
            return None;
        }
        let file = parts
            .iter()
            .fold(build.folder.clone(), |path, part| path.join(part));
        // A part that is a path of its own (`C:` on Windows) would leave the folder.
        Path::new(rest)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
            .then_some(file)
            .filter(|file| file.is_file())
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

/// A refusal with `status`, saying `why` in plain text.
fn refused(status: StatusCode, why: &str) -> Response<Vec<u8>> {
    with_type(status, "text/plain; charset=utf-8", why.as_bytes().to_vec())
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
