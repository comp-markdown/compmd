//! The processor's packs over the bundle and the store ([hosting], decision 7): a host that processes documents in Rust
//! resolves imports from exactly the files the frames load.
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use compmd::host::{Packs, Release};
use semver::Version;

use crate::bundle::Bundle;
use crate::place::Place;
use crate::store::Store;

/// The bundle's packs, then the store's, as `compmd::host::Packs`.
///
/// ```
/// # use compmd_host::{Bundle, HostPacks, NoStore};
/// # let bundle = Bundle::from_files([("manifest.json", br#"{"format": 1, "frames": {"hash": "1df336e75685de8c"}}"#.to_vec())])?;
/// let packs = HostPacks::new(&bundle, &NoStore);
/// // where the host processes: `.resolve((), packs, &ambient)`
/// # drop(packs);
/// # Ok::<(), compmd_host::BundleError>(())
/// ```
#[derive(Clone, Copy)]
pub struct HostPacks<'a> {
    bundle: &'a Bundle,
    store: &'a dyn Store,
}

impl<'a> HostPacks<'a> {
    /// The packs of `bundle`, then of `store`.
    #[must_use]
    pub const fn new(bundle: &'a Bundle, store: &'a dyn Store) -> Self {
        Self { bundle, store }
    }
}

impl Packs for HostPacks<'_> {
    fn releases(&self, pack: &str) -> Vec<Release> {
        let bundled = self
            .bundle
            .manifest()
            .packs
            .iter()
            .filter(|bundled| bundled.name == pack)
            .map(|bundled| Release {
                version: bundled.version.clone(),
                integrity: bundled.integrity.clone(),
                broken: None,
            });
        let mut releases: Vec<Release> = bundled.collect();
        for stored in self.store.releases(pack) {
            if !releases
                .iter()
                .any(|release| release.version == stored.version)
            {
                releases.push(Release {
                    version: stored.version,
                    integrity: Some(stored.integrity),
                    broken: None,
                });
            }
        }
        releases
    }

    fn pack_file(&self, pack: &str, version: &Version, path: &str) -> Option<String> {
        let place = Place {
            pack: pack.to_owned(),
            version: version.clone(),
            path: path.to_owned(),
        };
        let bytes = self
            .bundle
            .pack_file(&place)
            .or_else(|| self.store.file(pack, version, path))?;
        String::from_utf8(bytes).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::tests::bundle;
    use crate::store::{NoStore, StoredRelease};

    /// A store holding `@kit/mermaid` 1.2.0 (the bundle's too) and 1.3.0, and a binary file.
    struct Kept;

    impl Store for Kept {
        fn releases(&self, pack: &str) -> Vec<StoredRelease> {
            if pack != "@kit/mermaid" {
                return Vec::new();
            }
            [(1, 2), (1, 3)]
                .into_iter()
                .map(|(major, minor)| StoredRelease {
                    version: Version::new(major, minor, 0),
                    integrity: format!("sha512-{minor}"),
                })
                .collect()
        }

        fn file(&self, _pack: &str, version: &Version, path: &str) -> Option<Vec<u8>> {
            match (version.minor, path) {
                (3, "package.json") => Some(b"{}".to_vec()),
                (3, "logo.png") => Some(vec![0xff, 0xfe]),
                _ => None,
            }
        }
    }

    #[test]
    fn the_bundles_release_comes_first_and_the_stores_others_after() {
        let bundle = bundle();
        let packs = HostPacks::new(&bundle, &Kept);
        let releases = packs.releases("@kit/mermaid");
        let versions: Vec<String> = releases
            .iter()
            .map(|release| release.version.to_string())
            .collect();
        assert_eq!(versions, ["1.2.0", "1.3.0"]);
        assert_eq!(
            releases[0].integrity.as_deref(),
            Some("sha512-abc"),
            "the bundle's integrity"
        );
        assert_eq!(releases[1].integrity.as_deref(), Some("sha512-3"));
        assert_eq!(packs.releases("@kit/none"), []);
    }

    #[test]
    fn a_file_is_read_from_the_bundle_or_the_store_as_text() {
        let bundle = bundle();
        let packs = HostPacks::new(&bundle, &Kept);
        assert_eq!(
            packs
                .pack_file("@kit/mermaid", &Version::new(1, 2, 0), "package.json")
                .as_deref(),
            Some(r#"{"name": "@kit/mermaid"}"#)
        );
        assert_eq!(
            packs
                .pack_file("@kit/mermaid", &Version::new(1, 3, 0), "package.json")
                .as_deref(),
            Some("{}")
        );
        assert_eq!(
            packs.pack_file("@kit/mermaid", &Version::new(1, 3, 0), "logo.png"),
            None,
            "not text"
        );
        assert_eq!(
            HostPacks::new(&bundle, &NoStore).pack_file(
                "@kit/mermaid",
                &Version::new(1, 3, 0),
                "package.json"
            ),
            None
        );
    }
}
