//! The trust a configuration gives a pack.

use serde::{Deserialize, Serialize};

/// The trust a configuration gives a pack (spec §15).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Trust {
    /// In a strict sandbox: the default.
    #[default]
    Strict,
    /// In a permissive sandbox, with the network to its listed origins.
    Permissive,
    /// In the host's page: a folder pack built with the app only.
    Trusted,
}
