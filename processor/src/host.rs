//! What a host provides. The processor does no I/O: it reads the document's files, the packs, remote and `host:` data
//! and the time zones through these traits, each on its own, so a stage takes only the ones it uses and a host can
//! reuse an implementation, such as a pack store or a time zone database, in another host. `()` implements each as
//! "none", and maps implement the data traits for what a host holds in memory: `HashMap<String, String>` for files,
//! `HashMap<String, Vec<LoadedRelease>>` for packs, `HashMap<String, Fetched>` for remote data by URL and
//! `HashMap<String, Json>` for host data by source.
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
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::rc::Rc;

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
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Release {
    /// Its version.
    pub version: Version,
    /// The integrity hash the registry publishes for it (`sha512-…`), which an import's `integrity` must equal.
    #[serde(default)]
    pub integrity: Option<String>,
}

/// A release of a pack with all its files, as an in-memory host holds it: `HashMap<String, Vec<LoadedRelease>>` is a
/// registry of packs by name.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct LoadedRelease {
    /// The release.
    #[serde(flatten)]
    pub release: Release,
    /// Its files by path: `package.json`, the pack's document, and what it imports.
    pub files: HashMap<String, String>,
}

/// The remote and `host:` sources a render needs, for a host to fetch before it loads: each once, in the order they
/// are first written.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Needed {
    /// The `https:` URLs [`Fetch`] will be asked for, their values filled in.
    pub remote: Vec<String>,
    /// The sources [`Data`] will be asked for, without `host:`.
    pub host: Vec<String>,
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

/// Packs held in memory, by name.
impl<S: BuildHasher> Packs for HashMap<String, Vec<LoadedRelease>, S> {
    fn releases(&self, pack: &str) -> Vec<Release> {
        self.get(pack)
            .into_iter()
            .flatten()
            .map(|loaded| loaded.release.clone())
            .collect()
    }

    fn pack_file(&self, pack: &str, version: &Version, path: &str) -> Option<String> {
        self.get(pack)?
            .iter()
            .find(|loaded| loaded.release.version == *version)?
            .files
            .get(path)
            .cloned()
    }
}

/// No remote data: every URL fails, so data imports from URLs load their fallback.
impl Fetch for () {
    fn fetch(&self, _url: &str) -> Option<Fetched> {
        None
    }
}

/// Remote data fetched beforehand, by URL.
impl<S: BuildHasher> Fetch for HashMap<String, Fetched, S> {
    fn fetch(&self, url: &str) -> Option<Fetched> {
        self.get(url).cloned()
    }
}

/// No host data: every `host:` source is missing.
impl Data for () {
    fn data(&self, _source: &str) -> Option<Json> {
        None
    }
}

/// Host data held in memory, by source without `host:`.
impl<S: BuildHasher> Data for HashMap<String, Json, S> {
    fn data(&self, source: &str) -> Option<Json> {
        self.get(source).cloned()
    }
}

/// Files a host shares with the processor and keeps a handle to.
impl<T: Files + ?Sized> Files for Rc<T> {
    fn file(&self, path: &str) -> Option<String> {
        (**self).file(path)
    }
}

/// Packs a host shares with the processor.
impl<T: Packs + ?Sized> Packs for Rc<T> {
    fn releases(&self, pack: &str) -> Vec<Release> {
        (**self).releases(pack)
    }

    fn pack_file(&self, pack: &str, version: &Version, path: &str) -> Option<String> {
        (**self).pack_file(pack, version, path)
    }
}

/// Remote data a host shares with the processor.
impl<T: Fetch + ?Sized> Fetch for Rc<T> {
    fn fetch(&self, url: &str) -> Option<Fetched> {
        (**self).fetch(url)
    }
}

/// Host data a host shares with the processor.
impl<T: Data + ?Sized> Data for Rc<T> {
    fn data(&self, source: &str) -> Option<Json> {
        (**self).data(source)
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
    use serde_json::json;

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

    #[test]
    fn packs_in_memory_serve_their_releases() {
        let packs: HashMap<String, Vec<LoadedRelease>> = serde_json::from_value(json!({
            "@acme/charts": [
                { "version": "1.4.2", "integrity": "sha512-x", "files": { "package.json": "{}" } },
                { "version": "1.5.0", "files": {} }
            ]
        }))
        .expect("packs");
        let releases = packs.releases("@acme/charts");
        assert_eq!(releases.len(), 2);
        assert_eq!(releases[0].integrity.as_deref(), Some("sha512-x"));
        let version = Version::new(1, 4, 2);
        assert_eq!(
            packs.pack_file("@acme/charts", &version, "package.json"),
            Some("{}".to_string())
        );
        assert_eq!(
            packs.pack_file("@acme/charts", &Version::new(2, 0, 0), "a"),
            None
        );
        assert_eq!(packs.releases("@acme/maps"), Vec::new());
    }

    #[test]
    fn data_in_memory_is_served_by_url_and_source() {
        let fetched = Fetched {
            text: "{}".to_string(),
            content_type: Some("application/json".to_string()),
        };
        let remote = HashMap::from([("https://a.org/x.json".to_string(), fetched.clone())]);
        assert_eq!(remote.fetch("https://a.org/x.json"), Some(fetched));
        assert_eq!(remote.fetch("https://a.org/y.json"), None);
        let host = HashMap::from([("tasks".to_string(), json!([1]))]);
        assert_eq!(host.data("tasks"), Some(json!([1])));
        assert_eq!(host.data("notes"), None);
    }

    #[test]
    fn a_shared_host_serves_what_it_holds() {
        let files = Rc::new(HashMap::from([("a".to_string(), "A".to_string())]));
        assert_eq!(files.file("a"), Some("A".to_string()));
        let none = Rc::new(());
        assert_eq!(none.releases("@acme/kit"), Vec::new());
        assert_eq!(
            none.pack_file("@acme/kit", &Version::new(1, 0, 0), "a"),
            None
        );
        assert_eq!(none.fetch("https://a.org/x.json"), None);
        assert_eq!(none.data("tasks"), None);
    }
}
