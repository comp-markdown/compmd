//! A release a pack store holds.

use compmd_pack::{Integrity, ReleaseId};

/// A release a store holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredRelease {
    /// The release.
    pub release: ReleaseId,
    /// The integrity the registry publishes for it (`sha512-…`), which its files were checked against.
    pub integrity: Integrity,
}
