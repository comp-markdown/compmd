//! What a host sets for its resolutions: its registries, its size cap, and the packs its policy denies.

use std::collections::BTreeMap;

use compmd_pack::PackName;
use serde::{Deserialize, Serialize};

/// The size cap when the host sets none: Mermaid 12's module as one file is 7 MB, with its layout engines.
pub const DEFAULT_MAX_PACK_SIZE: u64 = 10_000_000;

/// A pack the host's policy denies, all versions or those in `version`, as its configuration's `deny` writes it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Denied {
    /// The pack.
    pub pack: PackName,
    /// The range denied, read as an import's is; all versions when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// A host's settings for resolving.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    /// The registries by scope, `@kit`, with `*` for every other scope, each a URL ending in `/`.
    pub registries: BTreeMap<String, String>,
    /// The largest unpacked size of a pack, in bytes.
    #[serde(default = "default_max_pack_size")]
    pub max_pack_size: u64,
    /// The packs the host's policy denies: a denied version is downloaded for nothing, so it never is.
    #[serde(default)]
    pub deny: Vec<Denied>,
}

/// The size cap when the host sets none.
const fn default_max_pack_size() -> u64 {
    DEFAULT_MAX_PACK_SIZE
}

impl Options {
    /// The options with `registries`, the default size cap and no pack denied.
    #[must_use]
    pub fn new(registries: BTreeMap<String, String>) -> Self {
        Self {
            registries,
            max_pack_size: DEFAULT_MAX_PACK_SIZE,
            deny: Vec::new(),
        }
    }

    /// The registry of `pack`: its scope's, else `*`'s.
    pub(crate) fn registry_of(&self, pack: &PackName) -> Option<&str> {
        self.registries
            .get(&format!("@{}", pack.scope()))
            .or_else(|| self.registries.get("*"))
            .map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pack_gets_its_scopes_registry_else_every_other_scopes() {
        let mut options = Options::new(BTreeMap::from([
            ("@kit".to_owned(), "https://npm.kit.example/".to_owned()),
            ("*".to_owned(), "https://registry.npmjs.org/".to_owned()),
        ]));
        let pack = |name: &str| name.parse::<PackName>().expect("a name");
        assert_eq!(
            options.registry_of(&pack("@kit/mermaid")),
            Some("https://npm.kit.example/")
        );
        assert_eq!(
            options.registry_of(&pack("@acme/maps")),
            Some("https://registry.npmjs.org/")
        );
        options.registries.remove("*");
        assert_eq!(options.registry_of(&pack("@acme/maps")), None);
        assert_eq!(options.max_pack_size, DEFAULT_MAX_PACK_SIZE);
        let read: Options = serde_json::from_str(r#"{"registries": {}}"#).expect("options");
        assert_eq!(
            (read.max_pack_size, read.deny),
            (DEFAULT_MAX_PACK_SIZE, Vec::new())
        );
    }
}
