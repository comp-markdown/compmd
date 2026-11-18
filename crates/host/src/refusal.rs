//! Why the serving contract refuses a request ([hosting], decision 8), with the status that says it.
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use http::StatusCode;

/// A refused request: its status, and why, which the answer says in plain text to any origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Refusal {
    /// `404` or `405`.
    pub status: StatusCode,
    /// Why, in plain text.
    pub why: &'static str,
}

impl Refusal {
    /// A request that isn't for a file the host serves.
    pub const NOT_SERVED: Self = Self {
        status: StatusCode::NOT_FOUND,
        why: "not a file of the frames or packs this host serves",
    };

    /// A request with another method than `GET`.
    pub const ONLY_GET: Self = Self {
        status: StatusCode::METHOD_NOT_ALLOWED,
        why: "the frames and packs are only read, with GET",
    };
}
