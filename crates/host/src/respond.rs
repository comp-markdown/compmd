//! The serving contract ([hosting], decision 8): a bundle's frames and the packs the bundle or the store holds, with
//! the headers that sandbox them, and a refusal, saying why, for anything else.
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use compmd_pack::FilePath;
use http::header::HeaderValue;
use http::{Request, Response};

use crate::bundle::Bundle;
use crate::headers::{refusal_headers, served_headers};
use crate::refusal::Refusal;
use crate::route::{Route, route};
use crate::store::Store;

/// The answer to `request`: the frames' file or pack file it names, from the bundle or, for a pack the bundle doesn't
/// hold, from `store`; a `405` for another method than `GET`; a `404` for anything else. Each refusal says why.
///
/// The paths it answers are `/frames/<hash>/<path>`, with the bundle's hash, and `/packs/<place>`, where a path is a
/// [`FilePath`], which can't leave its folder ([`route`]).
pub fn respond<B>(bundle: &Bundle, store: &dyn Store, request: &Request<B>) -> Response<Vec<u8>> {
    let hash = bundle.manifest().frames.hash.as_str();
    let found = match route(request.method(), request.uri().path(), hash) {
        Route::Frame(path) => bundle.frame_file(&path).map(|body| (body, path)),
        Route::Pack(place) => bundle
            .pack_file(&place)
            .or_else(|| store.file(&place))
            .map(|body| (body, place.path)),
        Route::Refused(refusal) => return refused(refusal),
    };
    found.map_or_else(
        || refused(Refusal::NOT_SERVED),
        |(body, path)| served(&path, body),
    )
}

/// `body`, the file at `path`, with its type and the headers.
fn served(path: &FilePath, body: Vec<u8>) -> Response<Vec<u8>> {
    with_headers(Response::new(body), served_headers(path.as_str()))
}

/// A refusal, saying why in plain text, to any origin.
fn refused(refusal: Refusal) -> Response<Vec<u8>> {
    let mut response = Response::new(refusal.why.as_bytes().to_vec());
    *response.status_mut() = refusal.status;
    with_headers(response, refusal_headers())
}

/// `response` with `headers`.
fn with_headers(
    mut response: Response<Vec<u8>>,
    headers: impl IntoIterator<Item = (&'static str, &'static str)>,
) -> Response<Vec<u8>> {
    for (name, value) in headers {
        response
            .headers_mut()
            .insert(name, HeaderValue::from_static(value));
    }
    response
}

#[cfg(test)]
mod tests;
