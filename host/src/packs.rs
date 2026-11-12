//! The processor's packs over the bundle and the store ([hosting], decision 7): a host that processes documents in Rust
//! resolves imports from exactly the files the frames load.
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use std::sync::Arc;

use compmd::host::{Packs, Release};
use compmd_pack::{PackName, Place};

use crate::bundle::Bundle;
use crate::store::Store;

/// The bundle's packs, then the store's, as `compmd::host::Packs`. It shares the bundle and the store, which the host
/// keeps, so the processor, which keeps its packs while it processes, can be given it.
///
/// ```
/// # use std::sync::Arc;
/// # use compmd::{Ambient, ParseOptions, Processor};
/// # use compmd_host::{Bundle, HostPacks, NoStore};
/// # let bundle = Bundle::from_files([("manifest.json", br#"{"format": 1, "frames": {"hash": "1df336e75685de8c"}}"#.to_vec())])?;
/// let packs = HostPacks::new(Arc::new(bundle), Arc::new(NoStore));
/// let resolved = Processor::new("a.comp.md", "# A\n").parse(&ParseOptions::default()).resolve((), packs, &Ambient::default());
/// # drop(resolved);
/// # Ok::<(), compmd_host::BundleError>(())
/// ```
#[derive(Clone)]
pub struct HostPacks {
    bundle: Arc<Bundle>,
    store: Arc<dyn Store>,
}

impl HostPacks {
    /// The packs of `bundle`, then of `store`.
    #[must_use]
    pub fn new(bundle: Arc<Bundle>, store: Arc<dyn Store>) -> Self {
        Self { bundle, store }
    }
}

impl Packs for HostPacks {
    fn releases(&self, pack: &PackName) -> Vec<Release> {
        let bundled = self
            .bundle
            .manifest()
            .packs
            .iter()
            .filter(|bundled| bundled.release.pack == *pack)
            .map(|bundled| Release {
                version: bundled.release.version.clone(),
                integrity: bundled.integrity.clone(),
                broken: None,
            });
        let mut releases: Vec<Release> = bundled.collect();
        for stored in self.store.releases(pack) {
            if !releases
                .iter()
                .any(|release| release.version == stored.release.version)
            {
                releases.push(Release {
                    version: stored.release.version,
                    integrity: Some(stored.integrity),
                    broken: None,
                });
            }
        }
        releases
    }

    fn pack_file(&self, place: &Place) -> Option<String> {
        let bytes = self
            .bundle
            .pack_file(place)
            .or_else(|| self.store.file(place))?;
        String::from_utf8(bytes).ok()
    }
}

#[cfg(test)]
mod tests {
    use compmd_pack::ReleaseId;

    use super::*;
    use crate::bundle::tests::{bundle, place};
    use crate::store::{NoStore, StoredRelease};

    /// A store holding `@kit/mermaid` 1.2.0 (the bundle's too) and 1.3.0, and a binary file.
    struct Kept;

    impl Store for Kept {
        fn releases(&self, pack: &PackName) -> Vec<StoredRelease> {
            if *pack != "@kit/mermaid" {
                return Vec::new();
            }
            [2, 3]
                .into_iter()
                .map(|minor| StoredRelease {
                    release: format!("@kit/mermaid@1.{minor}.0")
                        .parse::<ReleaseId>()
                        .expect("a release"),
                    integrity: format!("sha512-{minor}").parse().expect("an integrity"),
                })
                .collect()
        }

        fn file(&self, place: &Place) -> Option<Vec<u8>> {
            match place.to_string().as_str() {
                "@kit/mermaid@1.3.0/package.json" => Some(b"{}".to_vec()),
                "@kit/mermaid@1.3.0/logo.png" => Some(vec![0xff, 0xfe]),
                _ => None,
            }
        }
    }

    #[test]
    fn the_bundles_release_comes_first_and_the_stores_others_after() {
        let packs = HostPacks::new(Arc::new(bundle()), Arc::new(Kept));
        let releases = packs.releases(&"@kit/mermaid".parse().expect("a name"));
        let versions: Vec<String> = releases
            .iter()
            .map(|release| release.version.to_string())
            .collect();
        assert_eq!(versions, ["1.2.0", "1.3.0"]);
        let integrity = |text: &str| Some(text.parse().expect("an integrity"));
        assert_eq!(
            releases[0].integrity,
            integrity("sha512-abc"),
            "the bundle's integrity"
        );
        assert_eq!(releases[1].integrity, integrity("sha512-3"));
        assert_eq!(packs.releases(&"@kit/none".parse().expect("a name")), []);
    }

    #[test]
    fn a_file_is_read_from_the_bundle_or_the_store_as_text() {
        let bundle = Arc::new(bundle());
        let packs = HostPacks::new(Arc::clone(&bundle), Arc::new(Kept));
        assert_eq!(
            packs
                .pack_file(&place("@kit/mermaid@1.2.0/package.json"))
                .as_deref(),
            Some(r#"{"name": "@kit/mermaid"}"#)
        );
        assert_eq!(
            packs
                .pack_file(&place("@kit/mermaid@1.3.0/package.json"))
                .as_deref(),
            Some("{}")
        );
        assert_eq!(
            packs.pack_file(&place("@kit/mermaid@1.3.0/logo.png")),
            None,
            "not text"
        );
        assert_eq!(
            HostPacks::new(bundle, Arc::new(NoStore))
                .pack_file(&place("@kit/mermaid@1.3.0/package.json")),
            None
        );
    }
}
