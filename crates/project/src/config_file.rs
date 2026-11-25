//! Where a folder's configuration is: `compmd.config.json`, or `package.json`'s `compmd` object ([the Vite plugin],
//! decision 1). A pack's `package.json` names its document in `compmd` (spec §6.4): a string there isn't one.
//!
//! [the Vite plugin]: https://github.com/comp-markdown/compmd/blob/main/docs/vite.md

use std::path::Path;

use serde_json::Value;

use crate::config::{Config, read_config};
use crate::project_error::ProjectError;
use crate::project_files::ProjectFiles;

/// The configuration's own file.
pub(crate) const CONFIG_FILE: &str = "compmd.config.json";

/// The JSON at `path`, if there is a file there.
fn json_at(path: &Path, files: &impl ProjectFiles) -> Result<Option<Value>, ProjectError> {
    files
        .text(path)
        .map(|text| {
            serde_json::from_str(&text).map_err(|error| ProjectError::NotJson {
                path: path.to_owned(),
                message: error.to_string(),
            })
        })
        .transpose()
}

/// The configuration `package.json`'s `compmd` key holds: an object, not a pack's document.
fn manifest_config(manifest: Option<Value>) -> Option<Value> {
    manifest?
        .get("compmd")
        .filter(|compmd| compmd.is_object())
        .cloned()
}

/// Whether `dir` holds a configuration: `compmd.config.json`, or an object under `package.json`'s `compmd` key. A
/// `package.json` that isn't JSON holds none.
#[must_use]
pub fn holds_config(dir: &Path, files: &impl ProjectFiles) -> bool {
    files.text(&dir.join(CONFIG_FILE)).is_some()
        || manifest_config(json_at(&dir.join("package.json"), files).ok().flatten()).is_some()
}

/// The configuration of `folder`: `compmd.config.json` there, or its `package.json`'s `compmd` object; with neither,
/// the default configuration.
///
/// # Errors
///
/// [`ProjectError::NotJson`] when either file isn't JSON, [`ProjectError::TwoConfigurations`] when both have one,
/// and [`ProjectError::Config`] when the configuration is wrong.
pub fn config_of(folder: &Path, files: &impl ProjectFiles) -> Result<Config, ProjectError> {
    let in_manifest = manifest_config(json_at(&folder.join("package.json"), files)?);
    match (json_at(&folder.join(CONFIG_FILE), files)?, in_manifest) {
        (Some(_), Some(_)) => Err(ProjectError::TwoConfigurations),
        (Some(config), None) => Ok(read_config(&config, CONFIG_FILE)?),
        (None, Some(config)) => Ok(read_config(&config, "package.json (`compmd`)")?),
        (None, None) => Ok(Config::default()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::config::Fetch;
    use crate::project_files::tests::Memory;

    use super::*;

    #[test]
    fn a_folder_holds_a_configuration_in_its_own_file_or_its_manifests_object() {
        let files = Memory::of([
            ("/site/compmd.config.json", "{}"),
            ("/app/package.json", r#"{"compmd": {"fetch": "direct"}}"#),
            (
                "/pack/package.json",
                r#"{"name": "@acme/kit", "compmd": "./index.comp.md"}"#,
            ),
            ("/broken/package.json", "{"),
        ]);
        assert!(holds_config(Path::new("/site"), &files));
        assert!(holds_config(Path::new("/app"), &files));
        assert!(
            !holds_config(Path::new("/pack"), &files),
            "a pack's document"
        );
        assert!(!holds_config(Path::new("/broken"), &files));
        assert!(!holds_config(Path::new("/none"), &files));
    }

    #[test]
    fn the_configuration_is_its_files_its_manifests_or_the_default() {
        let files = Memory::of([
            ("/site/compmd.config.json", r#"{"fetch": "direct"}"#),
            ("/site/package.json", r#"{"name": "site"}"#),
            ("/app/package.json", r#"{"compmd": {"fetch": "direct"}}"#),
        ]);
        assert_eq!(
            config_of(Path::new("/site"), &files).map(|config| config.fetch),
            Ok(Fetch::Direct)
        );
        assert_eq!(
            config_of(Path::new("/app"), &files).map(|config| config.fetch),
            Ok(Fetch::Direct)
        );
        assert_eq!(config_of(Path::new("/none"), &files), Ok(Config::default()));
    }

    #[test]
    fn a_configuration_in_two_places_or_not_json_or_wrong_is_an_error() {
        let files = Memory::of([
            ("/two/compmd.config.json", "{}"),
            ("/two/package.json", r#"{"compmd": {}}"#),
            ("/broken/compmd.config.json", "{"),
            ("/wrong/package.json", r#"{"compmd": {"fetch": "proxy"}}"#),
        ]);
        assert_eq!(
            config_of(Path::new("/two"), &files),
            Err(ProjectError::TwoConfigurations)
        );
        assert!(matches!(
            config_of(Path::new("/broken"), &files),
            Err(ProjectError::NotJson { path, .. }) if path == Path::new("/broken/compmd.config.json")
        ));
        assert_eq!(
            config_of(Path::new("/wrong"), &files).map_err(|error| error.to_string()),
            Err("package.json (`compmd`):\n- fetch: `host` or `direct`".to_owned())
        );
    }
}
