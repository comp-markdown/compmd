//! What a host provides. The processor does no I/O: it reads the document's files, the packs, remote and `host:` data
//! and the time zones through these traits, each on its own, so a stage takes only the ones it uses and a host can
//! reuse an implementation, such as a pack store or a time zone database, in another host. `()` implements each as
//! "none", and maps implement the data traits for what a host holds in memory: `HashMap<String, String>` for files,
//! `HashMap<String, Vec<LoadedRelease>>` for packs, `HashMap<String, Fetched>` for remote data by URL and
//! `HashMap<String, Json>` for host data by source.
//!
//! ```
//! use compmd::host::{Packs, Release};
//! use compmd::pack::{PackName, Place};
//! use semver::Version;
//!
//! /// One pack, `@acme/kit@1.0.0`, with its manifest alone.
//! struct Kit;
//!
//! impl Packs for Kit {
//!     fn releases(&self, pack: &PackName) -> Vec<Release> {
//!         let kit = Release { version: Version::new(1, 0, 0), integrity: None, broken: None };
//!         if *pack == "@acme/kit" { vec![kit] } else { Vec::new() }
//!     }
//!
//!     fn pack_file(&self, place: &Place) -> Option<String> {
//!         let found = place.to_string() == "@acme/kit@1.0.0/package.json";
//!         found.then(|| r#"{"name": "@acme/kit", "version": "1.0.0"}"#.to_string())
//!     }
//! }
//! ```

use compmd_pack::{Integrity, PackName, Place};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use std::collections::{HashMap, HashSet};
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
    fn releases(&self, pack: &PackName) -> Vec<Release>;

    /// The file at `place`, in a release of a pack: `package.json`, the pack's document, or a file it imports.
    fn pack_file(&self, place: &Place) -> Option<String>;

    /// Whether the host is still getting `pack` (spec §6.4): an import of it that no release matches is pending, not
    /// F026, and its names' uses render their fallbacks marked as pending. None is, by default.
    fn pending(&self, _pack: &PackName) -> bool {
        false
    }
}

/// Remote data (the `remote data` feature, spec §8.6).
pub trait Fetch {
    /// What the host fetched for `url`, or `None` when it couldn't. Only `https:` URLs of origins the host allows
    /// reach this; the host fetches without credentials, and decides how long a copy is kept (`cache`).
    fn fetch(&self, url: &str) -> Option<Fetched>;

    /// Whether the host is still getting `url` (spec §8.6): when it can't give it yet, the import loads its
    /// `fallback` or `null`, without F026. None is, by default.
    fn pending(&self, _url: &str) -> bool {
        false
    }
}

/// Data a host provides (spec §8.6).
pub trait Data {
    /// The data `source` names, as JSON, or `None` when the host doesn't provide it. `source` is what follows
    /// `host:` in a `from`, such as `tasks?status=open`.
    fn data(&self, source: &str) -> Option<Json>;

    /// Whether the host is still getting `source`, as [`Fetch::pending`] for a URL. None is, by default.
    fn pending(&self, _source: &str) -> bool {
        false
    }
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
    pub integrity: Option<Integrity>,
    /// Why its files can't be used, when the host found they don't match its integrity: an import that resolves to
    /// it is F035 with this reason.
    #[serde(default)]
    pub broken: Option<String>,
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

/// What a render needs that the host hasn't given, for it to get before it processes or loads: each once, in the
/// order they are first written.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Needed {
    /// The `https:` URLs [`Fetch`] will be asked for, their values filled in.
    pub remote: Vec<String>,
    /// The sources [`Data`] will be asked for, without `host:`.
    pub host: Vec<String>,
    /// The `https:` URLs not fetched because their origin isn't one the host allows (F028), their values filled in:
    /// for a host that asks the reader about new origins before it fetches (spec §8.6), then validates again with the
    /// ones allowed.
    pub disallowed: Vec<String>,
    /// The packs imported, by the document or by a pack it loaded, that no release in [`Packs`] matches, for a host
    /// that gets packs from a registry (spec §6.4) to fetch before it resolves again; then the library packs those
    /// packs name in `peerDependencies` that none matches (companion, section 5). Packs the host's policy refuses
    /// aren't listed, nor are imports that name no pack.
    pub packs: Vec<NeededPack>,
}

/// What a render treated as pending (spec §6.4, §8.6): of what it needed, what the host said it was still getting,
/// for a host's UI ("loading 1 pack"). Each once, in the order they are first written.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Pending {
    /// The packs.
    pub packs: Vec<PackName>,
    /// The `https:` URLs, their values filled in.
    pub remote: Vec<String>,
    /// The `host:` sources, without `host:`.
    pub host: Vec<String>,
}

/// What a host keys its cached copy of a data import's source by (spec §8.6): the source's final URL or path, its
/// inputs filled in, so a copy is reused only for the same values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CacheKey {
    /// The import's name, its `as`: `sales` for `md.data.sales`.
    #[serde(rename = "as")]
    pub name: String,
    /// An `https:` URL, a `host:` source with `host:`, or a file's path: in the root folder (`data/sales.csv`) or in
    /// a pack (`@acme/kit@1.4.2/icons.json`).
    pub key: String,
}

/// A pack import, or a library a pack names, that [`Packs`] has no release for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NeededPack {
    /// The pack, `@acme/kit`.
    pub pack: PackName,
    /// The version or range after the name, as written (`1`, `^1.4.2`, `1.4.2`); `*` when none is. For a library,
    /// the range in `peerDependencies`.
    pub range: String,
    /// The import's `integrity`, when it pins one with an exact version: the release's tarball must have this hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity: Option<Integrity>,
    /// Whether it is a library pack a pack names, of which the processor takes the newest release that matches every
    /// range asked for it: a host that fetches one fetches that version when the registry has it.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub library: bool,
}

/// The newest of `versions` that `range` takes, as the processor reads the version or range after a pack's name in
/// an import's `from` (spec §6.1): `1.4.2` is exactly that version, `1` and `^1.4` ranges, `*` any. `None` when none
/// matches, or `range` isn't one. A host that fetches the packs [`Needed`] lists picks a version with this, so the
/// processor takes the release it was given.
///
/// ```
/// use compmd::host::newest_matching;
/// use semver::Version;
///
/// let versions = ["1.4.2", "1.5.0", "2.0.0"].map(|v| Version::parse(v).unwrap());
/// assert_eq!(newest_matching("1", &versions), Some(&versions[1]));
/// assert_eq!(newest_matching("1.4.2", &versions), Some(&versions[0]));
/// assert_eq!(newest_matching("^3", &versions), None);
/// ```
#[must_use]
pub fn newest_matching<'v>(range: &str, versions: &'v [Version]) -> Option<&'v Version> {
    let (range, _) = crate::resolve::range(range)?;
    versions
        .iter()
        .filter(|version| range.matches(version))
        .max()
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
    fn releases(&self, _pack: &PackName) -> Vec<Release> {
        Vec::new()
    }

    fn pack_file(&self, _place: &Place) -> Option<String> {
        None
    }
}

/// Packs held in memory, by name.
impl<S: BuildHasher> Packs for HashMap<String, Vec<LoadedRelease>, S> {
    fn releases(&self, pack: &PackName) -> Vec<Release> {
        self.get(pack.as_str())
            .into_iter()
            .flatten()
            .map(|loaded| loaded.release.clone())
            .collect()
    }

    fn pack_file(&self, place: &Place) -> Option<String> {
        self.get(place.release.pack.as_str())?
            .iter()
            .find(|loaded| loaded.release.version == place.release.version)?
            .files
            .get(place.path.as_str())
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
    fn releases(&self, pack: &PackName) -> Vec<Release> {
        (**self).releases(pack)
    }

    fn pack_file(&self, place: &Place) -> Option<String> {
        (**self).pack_file(place)
    }

    fn pending(&self, pack: &PackName) -> bool {
        (**self).pending(pack)
    }
}

/// Remote data a host shares with the processor.
impl<T: Fetch + ?Sized> Fetch for Rc<T> {
    fn fetch(&self, url: &str) -> Option<Fetched> {
        (**self).fetch(url)
    }

    fn pending(&self, url: &str) -> bool {
        (**self).pending(url)
    }
}

/// Host data a host shares with the processor.
impl<T: Data + ?Sized> Data for Rc<T> {
    fn data(&self, source: &str) -> Option<Json> {
        (**self).data(source)
    }

    fn pending(&self, source: &str) -> bool {
        (**self).pending(source)
    }
}

/// A host's packs, remote data or host data, `held`, with the sources it is still getting, `pending` (spec §6.4,
/// §8.6): pack names, URLs, or `host:` sources without `host:`.
///
/// ```
/// use compmd::host::{Packs, WithPending};
/// use std::collections::HashSet;
///
/// let packs = WithPending { held: (), pending: HashSet::from(["@kit/mermaid".to_string()]) };
/// assert!(packs.pending(&"@kit/mermaid".parse()?));
/// assert!(!packs.pending(&"@kit/charts".parse()?));
/// # Ok::<(), compmd::pack::Invalid>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WithPending<T> {
    /// What the host holds.
    pub held: T,
    /// What it is still getting.
    pub pending: HashSet<String>,
}

impl<T: Packs> Packs for WithPending<T> {
    fn releases(&self, pack: &PackName) -> Vec<Release> {
        self.held.releases(pack)
    }

    fn pack_file(&self, place: &Place) -> Option<String> {
        self.held.pack_file(place)
    }

    fn pending(&self, pack: &PackName) -> bool {
        self.pending.contains(pack.as_str())
    }
}

impl<T: Fetch> Fetch for WithPending<T> {
    fn fetch(&self, url: &str) -> Option<Fetched> {
        self.held.fetch(url)
    }

    fn pending(&self, url: &str) -> bool {
        self.pending.contains(url)
    }
}

impl<T: Data> Data for WithPending<T> {
    fn data(&self, source: &str) -> Option<Json> {
        self.held.data(source)
    }

    fn pending(&self, source: &str) -> bool {
        self.pending.contains(source)
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

    fn name(text: &str) -> PackName {
        text.parse().expect("a pack's name")
    }

    fn place(text: &str) -> Place {
        text.parse().expect("a place")
    }

    #[test]
    fn the_unit_host_has_nothing_but_utc() {
        assert_eq!(().file("a.comp.md"), None);
        assert_eq!(().releases(&name("@acme/kit")), Vec::new());
        assert_eq!(().pack_file(&place("@acme/kit@1.0.0/a")), None);
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
        let releases = packs.releases(&name("@acme/charts"));
        assert_eq!(releases.len(), 2);
        assert_eq!(
            releases[0]
                .integrity
                .as_ref()
                .map(ToString::to_string)
                .as_deref(),
            Some("sha512-x")
        );
        assert_eq!(
            packs.pack_file(&place("@acme/charts@1.4.2/package.json")),
            Some("{}".to_string())
        );
        assert_eq!(packs.pack_file(&place("@acme/charts@2.0.0/a")), None);
        assert_eq!(packs.releases(&name("@acme/maps")), Vec::new());
    }

    #[test]
    fn the_newest_matching_version_is_the_processors_choice() {
        let versions =
            ["1.0.0", "1.2.0", "2.0.0-beta.1"].map(|v| Version::parse(v).expect("a version"));
        assert_eq!(
            newest_matching("*", &versions),
            Some(&versions[1]),
            "not a prerelease"
        );
        assert_eq!(newest_matching("~1.0", &versions), Some(&versions[0]));
        assert_eq!(newest_matching("soon", &versions), None);
        assert_eq!(newest_matching("1", &[]), None);
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
        assert_eq!(none.releases(&name("@acme/kit")), Vec::new());
        assert_eq!(none.pack_file(&place("@acme/kit@1.0.0/a")), None);
        assert_eq!(none.fetch("https://a.org/x.json"), None);
        assert_eq!(none.data("tasks"), None);
    }
}
