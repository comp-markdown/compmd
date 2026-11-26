//! Who fetches a bundle's registry packs, as its configuration's `fetch` says.

use serde::{Deserialize, Serialize};

/// Who fetches from the registries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FetchedBy {
    /// The host, through its `registry/` proxy: the default.
    #[default]
    Host,
    /// The reader's browser, from the registries themselves.
    Direct,
}
