//! The headers the serving contract answers with ([hosting], decision 8): a served file's, which sandbox it, and a
//! refusal's.
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use crate::content_type::content_type;

/// The headers every served file has, beside its type.
const SERVED: [(&str, &str); 5] = [
    // Sandboxed even when a frame document is opened outside a frame.
    ("content-security-policy", "sandbox allow-scripts"),
    // A frame's module scripts come from an opaque origin, so they are cross-origin requests.
    ("access-control-allow-origin", "*"),
    // A page that checks how its frames are served (a host's check) reads these headers.
    ("access-control-expose-headers", "*"),
    ("x-content-type-options", "nosniff"),
    // A frame's path names its build, a pack's its version, so what it serves never changes.
    ("cache-control", "max-age=31536000, immutable"),
];

/// The headers of the file at `path`, served: its type, by its extension, and those that sandbox it.
///
/// ```
/// let headers = compmd_host::served_headers("strict.html");
/// assert!(headers.contains(&("content-type", "text/html; charset=utf-8")));
/// assert!(headers.contains(&("content-security-policy", "sandbox allow-scripts")));
/// ```
#[must_use]
pub fn served_headers(path: &str) -> Vec<(&'static str, &'static str)> {
    let mut headers = vec![("content-type", content_type(path))];
    headers.extend(SERVED);
    headers
}

/// The headers of a refusal: plain text, readable from any origin, so a page reads why.
#[must_use]
pub fn refusal_headers() -> [(&'static str, &'static str); 2] {
    [
        ("content-type", "text/plain; charset=utf-8"),
        ("access-control-allow-origin", "*"),
    ]
}
