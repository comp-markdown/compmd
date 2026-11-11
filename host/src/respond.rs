//! The serving contract ([hosting], decision 8): a bundle's frames and the packs the bundle or the store holds, with
//! the headers that sandbox them, and a refusal, saying why, for anything else.
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use std::path::{Component, Path};

use http::header::{self, HeaderValue};
use http::{Method, Request, Response, StatusCode};

use crate::bundle::Bundle;
use crate::content_type::content_type;
use crate::place::Place;
use crate::store::Store;

/// The headers every served file has.
const HEADERS: [(&str, &str); 5] = [
    // Sandboxed even when a frame document is opened outside a frame.
    ("content-security-policy", "sandbox allow-scripts"),
    // A frame's module scripts come from an opaque origin, so they are cross-origin requests.
    (ALLOW_ORIGIN, "*"),
    // A page that checks how its frames are served (a host's check) reads these headers.
    ("access-control-expose-headers", "*"),
    ("x-content-type-options", "nosniff"),
    // A frame's path names its build, a pack's its version, so what it serves never changes.
    ("cache-control", "max-age=31536000, immutable"),
];

/// Readable from any origin: a file, and a refusal, so a page reads why.
const ALLOW_ORIGIN: &str = "access-control-allow-origin";

/// Why a request that isn't for a file the host serves is refused.
const NOT_SERVED: &str = "not a file of the frames or packs this host serves";

/// Why a request with another method is refused.
const ONLY_GET: &str = "the frames and packs are only read, with GET";

/// The answer to `request`: the frames' file or pack file it names, from the bundle or, for a pack the bundle doesn't
/// hold, from `store`; a `405` for another method than `GET`; a `404` for anything else. Each refusal says why.
///
/// The paths it answers are `/frames/<hash>/<path>`, with the bundle's hash, and `/packs/<name>@<version>/<path>`,
/// with no empty, `.` or `..` part.
pub fn respond<B>(bundle: &Bundle, store: &dyn Store, request: &Request<B>) -> Response<Vec<u8>> {
    if request.method() != Method::GET {
        return refused(StatusCode::METHOD_NOT_ALLOWED, ONLY_GET);
    }
    let path = request.uri().path();
    file(bundle, store, path).map_or_else(
        || refused(StatusCode::NOT_FOUND, NOT_SERVED),
        |(body, name)| served(&name, body),
    )
}

/// The bytes of the file `path` names, with the name its type comes from.
fn file(bundle: &Bundle, store: &dyn Store, path: &str) -> Option<(Vec<u8>, String)> {
    let rest = path.strip_prefix('/')?;
    let plain = rest.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
        // A part that is a path of its own (`C:` on Windows) would leave the folder.
        && Path::new(rest)
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
    if !plain {
        return None;
    }
    let parts: Vec<&str> = rest.split('/').collect();
    match parts.as_slice() {
        ["frames", hash, file @ ..]
            if *hash == bundle.manifest().frames.hash && !file.is_empty() =>
        {
            let name = file.join("/");
            Some((bundle.frame_file(&name)?, name))
        }
        ["packs", place @ ..] => {
            let place = Place::from_parts(place)?;
            let body = bundle
                .pack_file(&place)
                .or_else(|| store.file(&place.pack, &place.version, &place.path))?;
            Some((body, place.path))
        }
        _ => None,
    }
}

/// `body`, the file `name`, with its type and the headers.
fn served(name: &str, body: Vec<u8>) -> Response<Vec<u8>> {
    let mut response = with_type(StatusCode::OK, content_type(name), body);
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
