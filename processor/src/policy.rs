//! The host's configuration of packs (spec §6.4, §15): which packs a document may import, how far each one's code is
//! trusted, and the packs the host denies or left out.
//!
//! ```
//! use compmd::policy::{Imports, Policy, Trust};
//!
//! let policy: Policy = serde_json::from_str(r#"{
//!     "packs": { "@acme/maps": { "level": "permissive", "network": ["https://tiles.example.com"] } },
//!     "imports": "listed",
//!     "deny": [{ "pack": "@bad/pack", "version": "<2", "reason": "a flaw found in 1.x" }],
//!     "refused": { "@huge/pack": "larger than this host serves" }
//! }"#).unwrap();
//! assert_eq!(policy.imports, Imports::Listed);
//! assert_eq!(policy.trust("@acme/charts"), Trust::Strict);
//! ```

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// How far a pack's code is trusted (spec §15).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "level", rename_all = "lowercase")]
pub enum Trust {
    /// A sandbox that sees only the host contract, with no network: the default.
    #[default]
    Strict,
    /// A sandbox of its own, which also reaches `network`, and is given `host` (spec §7.4).
    Permissive {
        /// The origins its code may reach, `https://tiles.example.com`.
        #[serde(default)]
        network: Vec<String>,
    },
    /// The host's own code, built with its application: it runs where the processor's code does.
    Trusted,
}

/// Which packs a document may import (spec §6.4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Imports {
    /// Any pack in the registry, in the strict sandbox unless listed: the default.
    #[default]
    Any,
    /// Only the packs the configuration lists, and the ambient ones.
    Listed,
}

/// A pack the host denies, as its own policy: all its versions, or those in `version`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Deny {
    /// The pack's name.
    pub pack: String,
    /// The versions denied; all when absent.
    #[serde(default)]
    pub version: Option<VersionReq>,
    /// Why, which the diagnostic says.
    pub reason: String,
}

/// The host's configuration of packs. The default allows any pack, in the strict sandbox.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Policy {
    /// The packs the configuration lists, each with its trust level; any other is strict.
    #[serde(default)]
    pub packs: BTreeMap<String, Trust>,
    /// Which packs a document may import.
    #[serde(default)]
    pub imports: Imports,
    /// The packs the host denies.
    #[serde(default)]
    pub deny: Vec<Deny>,
    /// The packs the host left out, each with why, such as being larger than it serves.
    #[serde(default)]
    pub refused: BTreeMap<String, String>,
}

impl Policy {
    /// How far the code of `pack` is trusted.
    #[must_use]
    pub fn trust(&self, pack: &str) -> Trust {
        self.packs.get(pack).cloned().unwrap_or_default()
    }

    /// Why a document may not import `pack`, if it may not, before a release is chosen: the host left it out, or
    /// imports are listed and it isn't, nor among `ambient`.
    pub(crate) fn refusal(&self, pack: &str, ambient: &[String]) -> Option<String> {
        if let Some(reason) = self.refused.get(pack) {
            return Some(format!("`{pack}` isn't available here: {reason}"));
        }
        let listed = self.packs.contains_key(pack) || ambient.iter().any(|known| known == pack);
        (self.imports == Imports::Listed && !listed).then(|| {
            format!(
                "this host allows only the packs its configuration lists, and `{pack}` isn't one"
            )
        })
    }

    /// Why `pack` at `version` is denied, if it is.
    pub(crate) fn denial(&self, pack: &str, version: &Version) -> Option<String> {
        self.deny
            .iter()
            .find(|deny| {
                deny.pack == pack
                    && deny
                        .version
                        .as_ref()
                        .is_none_or(|range| range.matches(version))
            })
            .map(|deny| format!("`{pack}@{version}` is denied here: {}", deny.reason))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn policy(value: serde_json::Value) -> Policy {
        serde_json::from_value(value).expect("a policy")
    }

    #[test]
    fn listed_packs_have_their_trust_and_others_are_strict() {
        let policy = policy(json!({ "packs": {
            "@acme/app": { "level": "trusted" },
            "@acme/maps": { "level": "permissive", "network": ["https://tiles.example.com"] },
            "@acme/charts": { "level": "permissive" }
        } }));
        assert_eq!(policy.trust("@acme/app"), Trust::Trusted);
        assert_eq!(
            policy.trust("@acme/maps"),
            Trust::Permissive {
                network: vec!["https://tiles.example.com".into()]
            }
        );
        assert_eq!(
            policy.trust("@acme/charts"),
            Trust::Permissive {
                network: Vec::new()
            }
        );
        assert_eq!(policy.trust("@other/kit"), Trust::Strict);
        assert_eq!(
            serde_json::to_value(Trust::Strict).expect("JSON"),
            json!({ "level": "strict" })
        );
    }

    #[test]
    fn a_pack_is_refused_when_left_out_or_unlisted() {
        let any = policy(json!({ "refused": { "@huge/pack": "larger than 5 MB" } }));
        assert_eq!(
            any.refusal("@huge/pack", &[]).as_deref(),
            Some("`@huge/pack` isn't available here: larger than 5 MB")
        );
        assert_eq!(any.refusal("@acme/kit", &[]), None);
        let listed = policy(
            json!({ "imports": "listed", "packs": { "@acme/maps": { "level": "strict" } } }),
        );
        assert_eq!(listed.refusal("@acme/maps", &[]), None);
        assert_eq!(
            listed.refusal("@host/c", &["@host/c".into()]),
            None,
            "ambient"
        );
        assert!(listed.refusal("@acme/kit", &[]).is_some());
    }

    #[test]
    fn a_denial_covers_its_versions() {
        let policy = policy(json!({ "deny": [
            { "pack": "@bad/pack", "reason": "misuse" },
            { "pack": "@acme/kit", "version": "<2", "reason": "a flaw found in 1.x" }
        ] }));
        let version = |text: &str| Version::parse(text).expect("a version");
        assert_eq!(
            policy.denial("@bad/pack", &version("3.0.0")).as_deref(),
            Some("`@bad/pack@3.0.0` is denied here: misuse")
        );
        assert!(policy.denial("@acme/kit", &version("1.4.2")).is_some());
        assert_eq!(policy.denial("@acme/kit", &version("2.0.0")), None);
        assert_eq!(policy.denial("@acme/charts", &version("1.0.0")), None);
    }
}
