//! The host's configuration of packs (spec §6.4, §15): which packs a document may import, how far each one's code is
//! trusted, the packs the host denies or left out, and the runtimes and libraries it provides component code (spec
//! §7.6).
//!
//! ```
//! use compmd::policy::{Imports, Policy, Trust};
//!
//! let policy: Policy = serde_json::from_str(r#"{
//!     "packs": { "@acme/maps": { "level": "permissive", "network": ["https://tiles.example.com"] } },
//!     "imports": "listed",
//!     "deny": [{ "pack": "@bad/pack", "version": "<2", "reason": "a flaw found in 1.x" }],
//!     "refused": { "@huge/pack": "larger than this host serves" },
//!     "runtimes": [{ "name": "react", "version": "19.3.0" }, { "name": "svelte", "version": "5.57.2" }],
//!     "libraries": [{ "name": "d3", "version": "7.9.0" }]
//! }"#).unwrap();
//! assert_eq!(policy.imports, Imports::Listed);
//! assert_eq!(policy.trust(&"@acme/charts".parse()?), Trust::Strict);
//! # Ok::<(), compmd::pack::Invalid>(())
//! ```

use compmd_pack::PackName;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The npm names of the spec's runtimes (spec §7.6): a pack's `peerDependencies` on one of them are checked against
/// the host's runtimes, any other against its libraries.
const RUNTIME_PACKAGES: [&str; 4] = ["react", "react-dom", "preact", "svelte"];

/// How far a pack's code is trusted (spec §15).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "level", rename_all = "lowercase")]
pub enum Trust {
    /// A sandbox that sees only the host contract, with no network: the default.
    #[default]
    Strict,
    /// A sandbox of its own, which also reaches `network`, and `host` if given it (spec §7.4).
    Permissive {
        /// The origins its code may reach, `https://tiles.example.com`.
        #[serde(default)]
        network: Vec<String>,
        /// Whether its components are given `host`.
        #[serde(default)]
        host: bool,
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
    /// The pack.
    pub pack: PackName,
    /// The versions denied; all when absent.
    #[serde(default)]
    pub version: Option<VersionReq>,
    /// Why, which the diagnostic says.
    pub reason: String,
}

/// A runtime or library the host provides component code (spec §7.6): its npm name and the version it has.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Library {
    /// The npm name, `svelte` or `d3`.
    pub name: String,
    /// The version the host has, `5.57.2`.
    pub version: Version,
}

/// The host's configuration of packs. The default allows any pack, in the strict sandbox.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct Policy {
    /// The packs the configuration lists, each with its trust level; any other is strict.
    #[serde(default)]
    pub packs: BTreeMap<PackName, Trust>,
    /// Which packs a document may import.
    #[serde(default)]
    pub imports: Imports,
    /// The packs the host denies.
    #[serde(default)]
    pub deny: Vec<Deny>,
    /// The packs the host left out, each with why, such as being larger than it serves.
    #[serde(default)]
    pub refused: BTreeMap<PackName, String>,
    /// The runtimes the host runs, by the framework's npm name (`react`, `react-dom`, `svelte`) and version. When
    /// absent, every runtime of the spec's bindings is assumed, at any version.
    #[serde(default)]
    pub runtimes: Option<Vec<Library>>,
    /// The libraries the host offers component code (spec §6.3). When absent, a pack's `peerDependencies` on
    /// libraries aren't checked.
    #[serde(default)]
    pub libraries: Option<Vec<Library>>,
}

impl Policy {
    /// How far the code of `pack` is trusted.
    #[must_use]
    pub fn trust(&self, pack: &PackName) -> Trust {
        self.packs.get(pack).cloned().unwrap_or_default()
    }

    /// Why a document may not import `pack`, if it may not, before a release is chosen: the host left it out, or
    /// imports are listed and it isn't, nor among `ambient`.
    pub(crate) fn refusal(&self, pack: &PackName, ambient: &[PackName]) -> Option<String> {
        if let Some(reason) = self.refused.get(pack) {
            return Some(format!("`{pack}` isn't available here: {reason}"));
        }
        let listed = self.packs.contains_key(pack) || ambient.contains(pack);
        (self.imports == Imports::Listed && !listed).then(|| {
            format!(
                "this host allows only the packs its configuration lists, and `{pack}` isn't one"
            )
        })
    }

    /// Whether the host runs `runtime`, a binding's runtime such as `svelte@5`: one of its runtimes has that name
    /// and major version.
    pub(crate) fn runs(&self, runtime: &str) -> bool {
        let Some(runtimes) = &self.runtimes else {
            return true;
        };
        runtime.rsplit_once('@').is_some_and(|(name, major)| {
            runtimes
                .iter()
                .any(|known| known.name == name && known.version.major.to_string() == major)
        })
    }

    /// Whether a pack's `peerDependencies` entry `name` is checked against the host's runtimes and libraries
    /// ([`Policy::unmet`]) rather than got as a library pack: a runtime, a library the host offers, or a name that
    /// isn't scoped, which no pack can have.
    pub(crate) fn provides(&self, name: &str) -> bool {
        RUNTIME_PACKAGES.contains(&name)
            || !name.starts_with('@')
            || self
                .libraries
                .iter()
                .flatten()
                .any(|library| library.name == name)
    }

    /// What is missing when a pack needs `name` in `range` (its `peerDependencies`, spec §6.4), if anything: a
    /// runtime the host doesn't run, a library it doesn't offer, or one it has at a version outside the range.
    /// Nothing when the host didn't say what it has.
    pub(crate) fn unmet(&self, name: &str, range: &VersionReq) -> Option<String> {
        let runtime = RUNTIME_PACKAGES.contains(&name);
        let provided = if runtime {
            self.runtimes.as_ref()
        } else {
            self.libraries.as_ref()
        }?;
        match provided.iter().find(|known| known.name == name) {
            None if runtime => Some(format!("`{name}`, a runtime this host doesn't run")),
            None => Some(format!("`{name}`, a library this host doesn't offer")),
            Some(known) if !range.matches(&known.version) => Some(format!(
                "`{name}` `{range}`, and this host has {}",
                known.version
            )),
            Some(_) => None,
        }
    }

    /// Why `pack` at `version` is denied, if it is.
    pub(crate) fn denial(&self, pack: &PackName, version: &Version) -> Option<String> {
        self.deny
            .iter()
            .find(|deny| {
                deny.pack == *pack
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

    fn name(text: &str) -> PackName {
        text.parse().expect("a pack's name")
    }

    #[test]
    fn listed_packs_have_their_trust_and_others_are_strict() {
        let policy = policy(json!({ "packs": {
            "@acme/app": { "level": "trusted" },
            "@acme/maps": { "level": "permissive", "network": ["https://tiles.example.com"] },
            "@acme/charts": { "level": "permissive", "host": true }
        } }));
        assert_eq!(policy.trust(&name("@acme/app")), Trust::Trusted);
        assert_eq!(
            policy.trust(&name("@acme/maps")),
            Trust::Permissive {
                network: vec!["https://tiles.example.com".into()],
                host: false,
            }
        );
        assert_eq!(
            policy.trust(&name("@acme/charts")),
            Trust::Permissive {
                network: Vec::new(),
                host: true,
            }
        );
        assert_eq!(policy.trust(&name("@other/kit")), Trust::Strict);
        assert_eq!(
            serde_json::to_value(Trust::Strict).expect("JSON"),
            json!({ "level": "strict" })
        );
    }

    #[test]
    fn a_pack_is_refused_when_left_out_or_unlisted() {
        let any = policy(json!({ "refused": { "@huge/pack": "larger than 5 MB" } }));
        assert_eq!(
            any.refusal(&name("@huge/pack"), &[]).as_deref(),
            Some("`@huge/pack` isn't available here: larger than 5 MB")
        );
        assert_eq!(any.refusal(&name("@acme/kit"), &[]), None);
        let listed = policy(
            json!({ "imports": "listed", "packs": { "@acme/maps": { "level": "strict" } } }),
        );
        assert_eq!(listed.refusal(&name("@acme/maps"), &[]), None);
        assert_eq!(
            listed.refusal(&name("@host/c"), &[name("@host/c")]),
            None,
            "ambient"
        );
        assert!(listed.refusal(&name("@acme/kit"), &[]).is_some());
    }

    #[test]
    fn the_host_runs_the_runtimes_it_lists_and_every_one_when_it_lists_none() {
        assert!(Policy::default().runs("preact@10"));
        let policy = policy(json!({ "runtimes": [
            { "name": "react", "version": "19.3.0" },
            { "name": "svelte", "version": "5.57.2" }
        ] }));
        assert!(policy.runs("react@19"));
        assert!(policy.runs("svelte@5"));
        assert!(!policy.runs("svelte@4"));
        assert!(!policy.runs("preact@10"));
        assert!(!policy.runs("react"));
    }

    #[test]
    fn a_packs_needs_are_met_by_the_hosts_runtimes_and_libraries() {
        let range = |text: &str| VersionReq::parse(text).expect("a range");
        let policy = policy(json!({
            "runtimes": [{ "name": "svelte", "version": "5.40.0" }],
            "libraries": [{ "name": "d3", "version": "7.9.0" }]
        }));
        assert_eq!(policy.unmet("svelte", &range("^5.20")), None);
        assert_eq!(
            policy.unmet("svelte", &range("^5.57")).as_deref(),
            Some("`svelte` `^5.57`, and this host has 5.40.0")
        );
        assert_eq!(
            policy.unmet("react", &range("^19")).as_deref(),
            Some("`react`, a runtime this host doesn't run")
        );
        assert_eq!(policy.unmet("d3", &range("^7")), None);
        assert_eq!(
            policy.unmet("three", &range("^0.170")).as_deref(),
            Some("`three`, a library this host doesn't offer")
        );
        assert_eq!(
            Policy::default().unmet("d3", &range("^7")),
            None,
            "not said"
        );
        assert_eq!(Policy::default().unmet("react", &range("^19")), None);
    }

    #[test]
    fn runtimes_offered_libraries_and_unscoped_names_are_provided_and_scoped_ones_are_library_packs()
     {
        let policy = policy(json!({ "libraries": [{ "name": "@acme/d3", "version": "7.9.0" }] }));
        assert!(policy.provides("svelte"));
        assert!(policy.provides("d3"), "not scoped");
        assert!(policy.provides("@acme/d3"), "offered");
        assert!(!policy.provides("@kit/three"));
        assert!(!Policy::default().provides("@kit/three"));
    }

    #[test]
    fn a_denial_covers_its_versions() {
        let policy = policy(json!({ "deny": [
            { "pack": "@bad/pack", "reason": "misuse" },
            { "pack": "@acme/kit", "version": "<2", "reason": "a flaw found in 1.x" }
        ] }));
        let version = |text: &str| Version::parse(text).expect("a version");
        assert_eq!(
            policy
                .denial(&name("@bad/pack"), &version("3.0.0"))
                .as_deref(),
            Some("`@bad/pack@3.0.0` is denied here: misuse")
        );
        assert!(
            policy
                .denial(&name("@acme/kit"), &version("1.4.2"))
                .is_some()
        );
        assert_eq!(policy.denial(&name("@acme/kit"), &version("2.0.0")), None);
        assert_eq!(
            policy.denial(&name("@acme/charts"), &version("1.0.0")),
            None
        );
    }
}
