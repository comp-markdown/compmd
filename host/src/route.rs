//! Which file a request names, or why it is refused ([hosting], decision 8): the serving contract's decision, apart
//! from reading the file, so `respond` here and `@compmd/host`'s (this crate as WebAssembly) decide alike.
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use compmd_pack::{FilePath, Place};
use http::Method;

use crate::refusal::Refusal;

/// What a request asks for: a file of the frames, a pack's file, or nothing the host serves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    /// The frames' file at this path, in the bundle.
    Frame(FilePath),
    /// A pack's file, in the bundle or the store.
    Pack(Place),
    /// Refused, with why.
    Refused(Refusal),
}

/// What a `method` request for `path` asks for, from a bundle whose frames' hash is `hash`.
///
/// The paths it answers are `/frames/<hash>/<path>` and `/packs/<place>`, where a path is a [`FilePath`], which can't
/// leave its folder; another method than `GET` is refused with `405`, anything else with `404`.
///
/// ```
/// use compmd_host::{route, Refusal, Route};
///
/// let route = route(&http::Method::GET, "/frames/1df336e75685de8c/strict.html", "1df336e75685de8c");
/// assert!(matches!(route, Route::Frame(path) if path.as_str() == "strict.html"));
/// let refused = compmd_host::route(&http::Method::GET, "/frames/1df336e75685de8c/../x", "1df336e75685de8c");
/// assert_eq!(refused, Route::Refused(Refusal::NOT_SERVED));
/// ```
#[must_use]
pub fn route(method: &Method, path: &str, hash: &str) -> Route {
    if method != Method::GET {
        return Route::Refused(Refusal::ONLY_GET);
    }
    file_of(path, hash).unwrap_or(Route::Refused(Refusal::NOT_SERVED))
}

/// The file `path` names, if it names one.
fn file_of(path: &str, hash: &str) -> Option<Route> {
    let parts: Vec<&str> = path.strip_prefix('/')?.split('/').collect();
    match parts.as_slice() {
        ["frames", given, file @ ..] if *given == hash => {
            FilePath::new(file.join("/")).ok().map(Route::Frame)
        }
        ["packs", place @ ..] => Place::from_parts(place).map(Route::Pack),
        _ => None,
    }
}
