//! No pack store: a host that serves its bundle's packs alone.

use compmd_pack::{PackName, Place};

use crate::store::Store;
use crate::stored_release::StoredRelease;

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
