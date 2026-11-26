//! A document's project, read as its hosts read it (`compmd-project`; [the language server], decision 4): its folder
//! packs, its installed packs, what its configuration gives the processor, and the registries it fetches packs from.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::BTreeMap;
use std::path::Path;

use compmd_host::registry::{DEFAULT_MAX_PACK_SIZE, Denied, Options};
use compmd_project::{Config, DiskFiles, PackFolders, Translated, config_of, integrities};

/// A project as its documents are processed in it.
pub(crate) struct Project {
    /// The folder packs its configuration lists.
    pub(crate) listed: PackFolders<DiskFiles>,
    /// The packs installed in its `node_modules`.
    pub(crate) installed: PackFolders<DiskFiles>,
    /// What its configuration gives the processor: its policy, its ambient packs, its allowed origins.
    pub(crate) translated: Translated,
    /// How packs are resolved from its registries; `None` when its configuration names none.
    pub(crate) registries: Option<Options>,
}

/// The registry rules' options for `config`: its registries, its size cap and the packs it denies.
fn registry_options(config: &Config) -> Option<Options> {
    if config.registries.is_empty() {
        return None;
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a size in bytes, which the configuration checks is a whole, positive number"
    )]
    let max_pack_size = config
        .max_pack_size
        .map_or(DEFAULT_MAX_PACK_SIZE, |size| size as u64);
    let deny = config
        .deny
        .iter()
        .map(|deny| Denied {
            pack: deny.pack.clone(),
            version: deny.version.as_ref().map(ToString::to_string),
        })
        .collect();
    Some(Options {
        registries: config.registries.clone(),
        max_pack_size,
        deny,
    })
}

impl Project {
    /// The project at `root`, or why it can't be read.
    pub(crate) fn read(root: &Path) -> Result<Self, String> {
        let config = config_of(root, &DiskFiles).map_err(|error| error.to_string())?;
        let installed = PackFolders::installed(root, DiskFiles, &integrities(root, &DiskFiles))
            .map_err(|error| error.to_string())?;
        let listed =
            PackFolders::listed(&config, root, DiskFiles).map_err(|error| error.to_string())?;
        let translated = config.translated(
            |key| listed.name_of(key).or_else(|| key.parse().ok()),
            &BTreeMap::new(),
        );
        Ok(Self {
            listed,
            installed,
            translated,
            registries: registry_options(&config),
        })
    }
}

#[cfg(test)]
mod tests {
    use compmd_project::read_config;
    use serde_json::json;

    use super::*;

    #[test]
    fn the_registries_options_come_from_the_configuration() {
        let config = |value| read_config(&value, "compmd.config.json").expect("a configuration");
        assert_eq!(registry_options(&config(json!({}))), None, "no registry");
        let written = json!({
            "registries": { "@kit": "https://npm.kit.example/" },
            "maxPackSize": "2 MB",
            "deny": [{ "pack": "@kit/old", "version": "<2", "reason": "unsafe" }, { "pack": "@kit/gone", "reason": "gone" }]
        });
        let options = registry_options(&config(written)).expect("options");
        assert_eq!(options.registries["@kit"], "https://npm.kit.example/");
        assert_eq!(options.max_pack_size, 2_000_000);
        let denied: Vec<(&str, Option<&str>)> = options
            .deny
            .iter()
            .map(|denied| (denied.pack.as_str(), denied.version.as_deref()))
            .collect();
        assert_eq!(denied, [("@kit/old", Some("<2")), ("@kit/gone", None)]);
        let unset = json!({ "registries": { "*": "https://registry.npmjs.org/" } });
        assert_eq!(
            registry_options(&config(unset)).map(|options| options.max_pack_size),
            Some(DEFAULT_MAX_PACK_SIZE)
        );
    }
}
