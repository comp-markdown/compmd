//! What the processor takes from a configuration: its policy, the ambient packs and the origins remote data may come
//! from, as `@compmd/vite`'s `policyOf` gives them.

use std::collections::BTreeMap;

use compmd::policy::Policy;
use compmd_pack::PackName;

use crate::config::{Config, Origins};

/// A configuration as the processor takes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Translated {
    /// The packs' trust, what may be imported, and what is denied or left out.
    pub policy: Policy,
    /// The packs every document gets without importing them.
    pub ambient: Vec<PackName>,
    /// The origins documents' remote data may come from without asking the reader.
    pub allowed_origins: Origins,
}

impl Config {
    /// What the processor takes from the configuration.
    ///
    /// `name_of` gives a pack key's pack name: a folder pack's `package.json` name, a pack name its own; a key it
    /// gives none for is left out. `refused` are the packs the host leaves out, each with why, such as being larger
    /// than [`Config::max_pack_size`].
    ///
    /// # Examples
    ///
    /// ```
    /// use compmd_project::read_config;
    ///
    /// let config = read_config(&serde_json::json!({ "packs": { "./components": { "ambient": true } } }), "compmd.config.json")?;
    /// let translated = config.translated(|key| if key == "./components" { "@acme/app".parse().ok() } else { key.parse().ok() }, &Default::default());
    /// assert_eq!(translated.ambient[0], "@acme/app");
    /// # Ok::<(), compmd_project::ConfigError>(())
    /// ```
    #[must_use]
    pub fn translated(
        &self,
        name_of: impl Fn(&str) -> Option<PackName>,
        refused: &BTreeMap<PackName, String>,
    ) -> Translated {
        let mut packs = BTreeMap::new();
        let mut ambient = Vec::new();
        for (key, pack) in &self.packs {
            let Some(name) = name_of(key) else {
                continue;
            };
            if pack.ambient {
                ambient.push(name.clone());
            }
            packs.insert(name, pack.trust.clone());
        }
        Translated {
            policy: Policy {
                packs,
                imports: self.imports,
                deny: self.deny.clone(),
                refused: refused.clone(),
                ..Policy::default()
            },
            ambient,
            allowed_origins: self.origins.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use compmd::policy::{Imports, Trust};
    use serde_json::json;

    use crate::config::is_folder;
    use crate::read_config;

    use super::*;

    fn name(text: &str) -> PackName {
        text.parse().expect("a pack's name")
    }

    #[test]
    fn gives_the_processors_policy_and_ambient_packs_folder_packs_by_their_names() {
        let config = read_config(
            &json!({
                "packs": {
                    "@kit/mermaid": {},
                    "@acme/maps": { "trust": "permissive", "network": ["https://tiles.example.com"] },
                    "./components": { "trust": "trusted", "ambient": true }
                },
                "deny": [{ "pack": "@bad/pack", "reason": "misuse" }]
            }),
            "compmd.config.json",
        )
        .expect("a configuration");
        let refused = BTreeMap::from([(name("@huge/pack"), "larger than 5 MB".to_owned())]);
        let translated = config.translated(
            |key| {
                if is_folder(key) {
                    Some(name("@acme/app"))
                } else {
                    key.parse().ok()
                }
            },
            &refused,
        );
        assert_eq!(
            translated.policy.packs,
            BTreeMap::from([
                (name("@kit/mermaid"), Trust::Strict),
                (
                    name("@acme/maps"),
                    Trust::Permissive {
                        network: vec!["https://tiles.example.com".into()],
                        host: false
                    }
                ),
                (name("@acme/app"), Trust::Trusted),
            ])
        );
        assert_eq!(translated.policy.imports, Imports::Any);
        assert_eq!(translated.policy.deny, config.deny);
        assert_eq!(translated.policy.refused, refused);
        assert_eq!(translated.ambient, [name("@acme/app")]);
        assert_eq!(
            Config::default()
                .translated(|key| key.parse().ok(), &BTreeMap::new())
                .policy,
            Policy::default()
        );
    }

    #[test]
    fn leaves_out_a_key_with_no_name() {
        let config = read_config(
            &json!({ "packs": { "./unnamed": { "ambient": true } } }),
            "compmd.config.json",
        )
        .expect("a configuration");
        let translated = config.translated(|_| None, &BTreeMap::new());
        assert_eq!(translated.policy.packs, BTreeMap::new());
        assert_eq!(translated.ambient, Vec::<PackName>::new());
    }

    #[test]
    fn gives_the_processor_the_origins_remote_data_may_come_from() {
        let origins = |value| {
            read_config(&value, "compmd.config.json")
                .expect("a configuration")
                .translated(|key| key.parse().ok(), &BTreeMap::new())
                .allowed_origins
        };
        assert_eq!(origins(json!({})), Origins::Listed(Vec::new()));
        assert_eq!(
            origins(json!({ "origins": ["https://cdn.example.com"] })),
            Origins::Listed(vec!["https://cdn.example.com".into()])
        );
        assert_eq!(origins(json!({ "origins": "any" })), Origins::Any);
    }
}
