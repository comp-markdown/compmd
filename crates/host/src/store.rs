//! The pack store's storage ([hosting], decisions 4 and 5): the releases a host fetched or keeps, by their place, each
//! checked against its integrity before it was kept. A host gives the storage: a folder ([`crate::FolderStore`]),
//! its own store of published versions, or none ([`crate::NoStore`]).
//!
//! [hosting]: https://github.com/comp-markdown/compmd/blob/main/docs/hosting.md

use compmd_pack::{PackName, Place};

use crate::stored_release::StoredRelease;

/// Where a host keeps packs beyond the bundle.
pub trait Store: Send + Sync {
    /// The releases of `pack` it holds, in any order.
    fn releases(&self, pack: &PackName) -> Vec<StoredRelease>;

    /// The bytes of the file at `place`, when it holds them.
    fn file(&self, place: &Place) -> Option<Vec<u8>>;
}
