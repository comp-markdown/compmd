//! The processor's packs over the bundle and the store ([hosting], decision 7): a host that processes documents in Rust
//! resolves imports from exactly the files the frames load.
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use std::sync::Arc;

use compmd::Ambient;
use compmd::host::{Packs, Release};
use compmd::policy::Policy;
use compmd_pack::{PackName, Place};
use serde::Deserialize;

use crate::bundle::Bundle;
use crate::manifest_error::ManifestError;
use crate::store::Store;

/// The bundle's packs, then the store's, as `compmd::host::Packs`, with the policy and the ambient packs the bundle's
/// configuration made, so a document processed in Rust is held to what the page's path holds it to ([hosting],
/// decision 7). It shares the bundle and the store, which the host keeps, so the processor, which keeps its packs while
/// it processes, can be given it.
///
/// ```
/// # use std::sync::Arc;
/// # use compmd::{ParseOptions, Processor};
/// # use compmd_host::{Bundle, HostPacks, NoStore};
/// # let bundle = Bundle::from_files([("manifest.json", br#"{"format": 1, "frames": {"hash": "1df336e75685de8c"}}"#.to_vec())])?;
/// let packs = HostPacks::new(Arc::new(bundle), Arc::new(NoStore));
/// let (policy, ambient) = (packs.policy()?, packs.ambient());
/// let resolved = Processor::new("a.comp.md", "# A\n")
///     .parse(&ParseOptions::default())
///     .resolve_with((), packs, &ambient, &policy);
/// # drop(resolved);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md
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

    /// The processor's policy the bundle's configuration made (its `deny`, `imports`, `refused` and the packs' trust):
    /// the default policy when the manifest has none.
    ///
    /// # Errors
    ///
    /// [`ManifestError::Policy`] when the manifest's `policy` isn't the processor's.
    pub fn policy(&self) -> Result<Policy, ManifestError> {
        self.bundle.manifest().policy.as_ref().map_or_else(
            || Ok(Policy::default()),
            |policy| {
                Policy::deserialize(policy)
                    .map_err(|error| ManifestError::Policy(error.to_string()))
            },
        )
    }

    /// The packs every document gets without importing them, as the bundle's configuration names them; their uses
    /// aren't reported, as on the page's path.
    #[must_use]
    pub fn ambient(&self) -> Ambient {
        Ambient {
            packs: self.bundle.manifest().ambient.clone(),
            report: false,
        }
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

    /// A bundle holding `@kit/notes` (a `note` component) and `@kit/old`, whose configuration makes `@kit/notes` ambient
    /// and denies `@kit/old`, as `@compmd/vite` writes it.
    fn configured() -> HostPacks {
        let manifest = br#"{"format": 1, "frames": {"hash": "1df336e75685de8c"},
            "packs": [{"pack": "@kit/notes", "version": "1.0.0"}, {"pack": "@kit/old", "version": "1.0.0"}],
            "policy": {"deny": [{"pack": "@kit/old", "reason": "a flaw found in 1.x"}]},
            "ambient": ["@kit/notes"]}"#;
        let pack = |name: &str| {
            [
                (
                    format!("packs/{name}@1.0.0/package.json"),
                    format!(r#"{{"name": "{name}", "version": "1.0.0"}}"#).into_bytes(),
                ),
                (
                    format!("packs/{name}@1.0.0/index.comp.md"),
                    b"---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=note\n---\nA note.\n```\n".to_vec(),
                ),
            ]
        };
        let files = [("manifest.json".to_owned(), manifest.to_vec())]
            .into_iter()
            .chain(pack("@kit/notes"))
            .chain(pack("@kit/old"));
        HostPacks::new(
            Arc::new(Bundle::from_files(files).expect("a bundle")),
            Arc::new(NoStore),
        )
    }

    /// The diagnostics of `text` processed in Rust with the bundle's packs, policy and ambient packs.
    fn diagnostics(text: &str) -> Vec<String> {
        let packs = configured();
        let (policy, ambient) = (packs.policy().expect("a policy"), packs.ambient());
        compmd::Processor::new("a.comp.md", text)
            .parse(&compmd::ParseOptions::default())
            .resolve_with((), packs, &ambient, &policy)
            .diagnostics()
            .iter()
            .map(|diagnostic| format!("{} {}", diagnostic.code, diagnostic.message))
            .collect()
    }

    #[test]
    fn processing_in_rust_holds_a_document_to_the_bundles_policy() {
        let denied = diagnostics(
            "---\nmd:\n  import:\n    - from: \"@kit/old@1\"\n      names: \"*\"\n---\n",
        );
        assert_eq!(
            denied,
            ["F026 `@kit/old@1.0.0` is denied here: a flaw found in 1.x; the import is dropped"]
        );
    }

    #[test]
    fn processing_in_rust_gives_a_document_the_bundles_ambient_packs() {
        assert_eq!(diagnostics(":: note[A note]\n"), Vec::<String>::new());
        assert_eq!(
            configured().ambient().packs,
            ["@kit/notes".parse::<PackName>().expect("a name")]
        );
    }

    #[test]
    fn a_bundle_without_a_policy_gets_the_default_and_one_the_processor_cant_read_is_an_error() {
        let packs = HostPacks::new(Arc::new(bundle()), Arc::new(NoStore));
        assert_eq!(packs.policy(), Ok(Policy::default()));
        assert_eq!(packs.ambient(), Ambient::default());
        let manifest =
            br#"{"format": 1, "frames": {"hash": "1df336e75685de8c"}, "policy": {"imports": "some"}}"#;
        let unreadable = HostPacks::new(
            Arc::new(
                Bundle::from_files([("manifest.json".to_owned(), manifest.to_vec())])
                    .expect("a bundle"),
            ),
            Arc::new(NoStore),
        );
        assert!(
            matches!(unreadable.policy(), Err(ManifestError::Policy(message)) if message.contains("some"))
        );
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
