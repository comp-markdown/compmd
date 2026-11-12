//! The pack store's storage ([hosting], decisions 4 and 5): the releases a host fetched or keeps, by their place, each
//! checked against its integrity before it was kept. A host gives the storage: a folder ([`crate::FolderStore`]),
//! its own store of published versions, or none ([`NoStore`]).
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use compmd_pack::{Integrity, PackName, Place, ReleaseId};

/// A release a store holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredRelease {
    /// The release.
    pub release: ReleaseId,
    /// The integrity the registry publishes for it (`sha512-…`), which its files were checked against.
    pub integrity: Integrity,
}

/// Where a host keeps packs beyond the bundle.
pub trait Store: Send + Sync {
    /// The releases of `pack` it holds, in any order.
    fn releases(&self, pack: &PackName) -> Vec<StoredRelease>;

    /// The bytes of the file at `place`, when it holds them.
    fn file(&self, place: &Place) -> Option<Vec<u8>>;
}

/// No store: the host serves the bundle's packs alone.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoStore;

impl Store for NoStore {
    fn releases(&self, _pack: &PackName) -> Vec<StoredRelease> {
        Vec::new()
    }

    fn file(&self, _place: &Place) -> Option<Vec<u8>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_store_holds_nothing() {
        assert_eq!(
            NoStore.releases(&"@kit/mermaid".parse().expect("a name")),
            []
        );
        assert_eq!(
            NoStore.file(&"@kit/mermaid@1.0.0/a.js".parse().expect("a place")),
            None
        );
    }
}
