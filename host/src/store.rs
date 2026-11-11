//! The pack store's storage ([hosting], decisions 4 and 5): the releases a host fetched or keeps, by their place, each
//! checked against its integrity before it was kept. A host gives the storage: a folder ([`crate::FolderStore`]),
//! its own store of published versions, or none ([`NoStore`]).
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use semver::Version;

/// A release a store holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredRelease {
    /// Its version.
    pub version: Version,
    /// The integrity the registry publishes for it (`sha512-…`), which its files were checked against.
    pub integrity: String,
}

/// Where a host keeps packs beyond the bundle.
pub trait Store: Send + Sync {
    /// The releases of `pack` it holds, in any order.
    fn releases(&self, pack: &str) -> Vec<StoredRelease>;

    /// The bytes of the file at `path` in `pack`'s release `version`, when it holds them.
    fn file(&self, pack: &str, version: &Version, path: &str) -> Option<Vec<u8>>;
}

/// No store: the host serves the bundle's packs alone.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoStore;

impl Store for NoStore {
    fn releases(&self, _pack: &str) -> Vec<StoredRelease> {
        Vec::new()
    }

    fn file(&self, _pack: &str, _version: &Version, _path: &str) -> Option<Vec<u8>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_store_holds_nothing() {
        assert_eq!(NoStore.releases("@kit/mermaid"), []);
        assert_eq!(
            NoStore.file("@kit/mermaid", &Version::new(1, 0, 0), "a.js"),
            None
        );
    }
}
