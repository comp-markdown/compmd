//! A pack a bundle holds, as its manifest lists it.

use std::collections::BTreeMap;

use compmd_pack::{FilePath, Integrity, ReleaseId};
use serde::{Deserialize, Serialize};

use crate::trust::Trust;

/// A pack the bundle holds. In JSON its release is flat: `{"pack", "version", "integrity", "trust", "modules"}`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct BundledPack {
    /// The release.
    #[serde(flatten)]
    pub release: ReleaseId,
    /// The integrity the registry publishes for it (`sha512-…`), from the app's lock file; none when the lock file
    /// gave none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrity: Option<Integrity>,
    /// The trust the configuration gives it.
    #[serde(default)]
    pub trust: Trust,
    /// Its built modules, by their path in the pack, with their integrity (`sha384-…`), which a frame checks.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub modules: BTreeMap<FilePath, Integrity>,
}
