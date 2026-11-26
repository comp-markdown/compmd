//! A document's project, read as its hosts read it (`compmd-project`; [the language server], decision 4): its folder
//! packs, its installed packs, what its configuration gives the processor, and the registries it fetches packs from.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::BTreeMap;
use std::path::Path;

use compmd_host::registry::{DEFAULT_MAX_PACK_SIZE, Denied, Options};
use compmd_project::{Config, DiskFiles, PackFolders, Translated, config_of, integrities};

use crate::own_packs::{SCOPE, site_fetches};

/// A project as its documents are processed in it.
pub(crate) struct Project {
    /// The folder packs its configuration lists.
    pub(crate) listed: PackFolders<DiskFiles>,
    /// The packs installed in its `node_modules`.
    pub(crate) installed: PackFolders<DiskFiles>,
    /// What its configuration gives the processor: its policy, its ambient packs, its allowed origins.
    pub(crate) translated: Translated,
    /// How the editor resolves packs from registries: those its configuration names, and `own` for CompMD's own packs
    /// when it names none for them.
    pub(crate) registries: Options,
    /// Whether its configuration names a registry for CompMD's own packs, so the site can fetch them.
    pub(crate) site_fetches_own: bool,
}

/// The registry rules' options for `config` in the editor: its registries, with `own` for CompMD's own packs when it
/// names none for them, its size cap and the packs it denies.
fn registry_options(config: &Config, own: &str) -> Options {
    let mut registries = config.registries.clone();
    if !site_fetches(&registries) {
        registries.insert(SCOPE.to_owned(), own.to_owned());
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
    Options {
        registries,
        max_pack_size,
        deny,
    }
}

impl Project {
    /// The project at `root`, CompMD's own packs resolved from `own` when its configuration names no registry for
    /// them; or why it can't be read.
    pub(crate) fn read(root: &Path, own: &str) -> Result<Self, String> {
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
            registries: registry_options(&config, own),
            site_fetches_own: site_fetches(&config.registries),
        })
    }
}

#[cfg(test)]
mod tests {
    use compmd_project::read_config;
    use serde_json::json;

    use super::*;

    #[test]
    fn the_registries_options_come_from_the_configuration_with_compmds_own() {
        let config = |value| read_config(&value, "compmd.config.json").expect("a configuration");
        let own = "https://npm.own.example/";
        assert_eq!(
            registry_options(&config(json!({})), own).registries,
            BTreeMap::from([(SCOPE.to_owned(), own.to_owned())]),
            "CompMD's own, with no registry"
        );
        let written = json!({
            "registries": { "@kit": "https://npm.kit.example/" },
            "maxPackSize": "2 MB",
            "deny": [{ "pack": "@kit/old", "version": "<2", "reason": "unsafe" }, { "pack": "@kit/gone", "reason": "gone" }]
        });
        let options = registry_options(&config(written), own);
        assert_eq!(options.registries["@kit"], "https://npm.kit.example/");
        assert_eq!(options.registries[SCOPE], own);
        assert_eq!(options.max_pack_size, 2_000_000);
        let denied: Vec<(&str, Option<&str>)> = options
            .deny
            .iter()
            .map(|denied| (denied.pack.as_str(), denied.version.as_deref()))
            .collect();
        assert_eq!(denied, [("@kit/old", Some("<2")), ("@kit/gone", None)]);
        let every = json!({ "registries": { "*": "https://registry.npmjs.org/" } });
        let options = registry_options(&config(every), own);
        assert_eq!(options.max_pack_size, DEFAULT_MAX_PACK_SIZE);
        assert!(
            !options.registries.contains_key(SCOPE),
            "the site's own registry for every scope"
        );
    }
}
