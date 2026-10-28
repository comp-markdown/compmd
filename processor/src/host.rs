//! What a host provides. The processor does no I/O: it reads the document's files, the packs, remote and `host:` data
//! and the time zones through these traits, each on its own, so a stage takes only the ones it uses and a host can
//! reuse an implementation, such as a pack store or a time zone database, in another host. `()` implements each as
//! "none".
//!
//! ```
//! use compmd::host::{Packs, Release};
//! use semver::Version;
//!
//! /// One pack, `@acme/kit@1.0.0`, with its manifest alone.
//! struct Kit;
//!
//! impl Packs for Kit {
//!     fn releases(&self, pack: &str) -> Vec<Release> {
//!         let kit = Release { version: Version::new(1, 0, 0), integrity: None };
//!         if pack == "@acme/kit" { vec![kit] } else { Vec::new() }
//!     }
//!
//!     fn pack_file(&self, pack: &str, version: &Version, path: &str) -> Option<String> {
//!         let found = pack == "@acme/kit" && *version == Version::new(1, 0, 0) && path == "package.json";
//!         found.then(|| r#"{"name": "@acme/kit", "version": "1.0.0"}"#.to_string())
//!     }
//! }
//! ```

use semver::Version;
use serde::Deserialize;
use serde_json::Value as Json;
use std::collections::HashMap;
use std::hash::BuildHasher;

/// The files of the document's root folder.
pub trait Files {
    /// The file at `path` under the document's root folder, which relative paths can't leave (spec §8.6), or `None`
    /// when there is none. `path` is `/`-separated.
    fn file(&self, path: &str) -> Option<String>;
}

/// The packs a document can import (spec §6.4): npm packages with a `package.json` and a CompMD document.
pub trait Packs {
    /// Every release of `pack`, in any order.
    fn releases(&self, pack: &str) -> Vec<Release>;

    /// The file at `path` in a release of `pack`: `package.json`, the pack's document, or a file it imports.
    fn pack_file(&self, pack: &str, version: &Version, path: &str) -> Option<String>;
}

/// Remote data (the `remote data` feature, spec §8.6).
pub trait Fetch {
    /// What the host fetched for `url`, or `None` when it couldn't. Only `https:` URLs of origins the host allows
    /// reach this; the host fetches without credentials, and decides how long a copy is kept (`cache`).
    fn fetch(&self, url: &str) -> Option<Fetched>;
}

/// Data a host provides (spec §8.6).
pub trait Data {
    /// The data `source` names, as JSON, or `None` when the host doesn't provide it. `source` is what follows
    /// `host:` in a `from`, such as `tasks?status=open`.
    fn data(&self, source: &str) -> Option<Json>;
}

/// The time zones the host knows: the browser's `Intl`, an app's own zone data. The crate carries none.
pub trait Time {
    /// The offset of `zone` from UTC at `at`, in seconds east of UTC, or `None` when the zone is unknown. `zone` is
    /// `md.tz`: an IANA name such as `Europe/Berlin`, or `UTC`; `at` is in seconds since 1970-01-01T00:00:00Z.
    fn offset(&self, zone: &str, at: i64) -> Option<i32>;
}

/// A release of a pack, as a registry lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    /// Its version.
    pub version: Version,
    /// The integrity hash the registry publishes for it (`sha512-…`), which an import's `integrity` must equal.
    pub integrity: Option<String>,
}

/// What a remote source gave.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fetched {
    /// The body, as text.
    pub text: String,
    /// The `Content-Type` header, which decides the format; without one, the URL's extension does.
    pub content_type: Option<String>,
}

/// No files.
impl Files for () {
    fn file(&self, _path: &str) -> Option<String> {
        None
    }
}

/// Files held in memory, as a browser hands them over.
impl<S: BuildHasher> Files for HashMap<String, String, S> {
    fn file(&self, path: &str) -> Option<String> {
        self.get(path).cloned()
    }
}

/// No packs.
impl Packs for () {
    fn releases(&self, _pack: &str) -> Vec<Release> {
        Vec::new()
    }

    fn pack_file(&self, _pack: &str, _version: &Version, _path: &str) -> Option<String> {
        None
    }
}

/// No remote data: every URL fails, so data imports from URLs load their fallback.
impl Fetch for () {
    fn fetch(&self, _url: &str) -> Option<Fetched> {
        None
    }
}

/// No host data: every `host:` source is missing.
impl Data for () {
    fn data(&self, _source: &str) -> Option<Json> {
        None
    }
}

/// No time zone data: only `UTC` is known.
impl Time for () {
    fn offset(&self, zone: &str, _at: i64) -> Option<i32> {
        (zone == "UTC").then_some(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_unit_host_has_nothing_but_utc() {
        assert_eq!(().file("a.comp.md"), None);
        assert_eq!(().releases("@acme/kit"), Vec::new());
        assert_eq!(().pack_file("@acme/kit", &Version::new(1, 0, 0), "a"), None);
        assert_eq!(().fetch("https://a.org/x.json"), None);
        assert_eq!(().data("tasks"), None);
        assert_eq!(().offset("UTC", 0), Some(0));
        assert_eq!(().offset("Europe/Berlin", 0), None);
    }
}
