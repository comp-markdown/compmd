//! Reading a project (`compmd-project`) for `@compmd/host`, as JSON: the configuration, the packs found, the lock
//! file's integrity ([the language server], Q3). Each function takes the files as `ProjectFiles`, which the
//! WebAssembly exports give from the host's own file system (`js_project`).
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::path::{Path, PathBuf};

use compmd_project::{
    Config, ConfigError, Fetch, Origins, PackFolder, PackFolders, ProjectError, ProjectFiles,
};
use serde_json::{Value, json};

/// `config` as `@compmd/vite`'s `ReadConfig` is written, its packs as `[key, pack]` pairs in order.
fn config_json(config: &Config) -> Value {
    let packs: Vec<Value> = config
        .packs
        .iter()
        .map(|(key, pack)| {
            let mut read = json!({ "trust": pack.trust, "ambient": pack.ambient });
            if let Some(version) = &pack.version {
                read["version"] = json!(version.to_string());
            }
            json!([key, read])
        })
        .collect();
    let deny: Vec<Value> = config
        .deny
        .iter()
        .map(|deny| {
            let mut entry = json!({ "pack": deny.pack.as_str(), "reason": deny.reason });
            if let Some(version) = &deny.version {
                entry["version"] = json!(version.to_string());
            }
            entry
        })
        .collect();
    let libraries: Vec<Value> = config
        .libraries
        .iter()
        .map(|library| json!({ "name": library.name, "imports": library.imports }))
        .collect();
    let mut read = json!({
        "packs": packs,
        "imports": match config.imports {
            compmd::policy::Imports::Any => "any",
            compmd::policy::Imports::Listed => "listed",
        },
        "deny": deny,
        "libraries": libraries,
        "registries": config.registries,
        "fetch": match config.fetch {
            Fetch::Host => "host",
            Fetch::Direct => "direct",
        },
        "origins": match &config.origins {
            Origins::Any => json!("any"),
            Origins::Listed(origins) => json!(origins),
        },
        "editor": { "suggest": config.editor.suggest },
    });
    if let Some(bytes) = config.max_pack_size {
        read["maxPackSize"] = json!(bytes);
    }
    read
}

/// What reading a configuration gave: `{config}`, or `{problems, source, message}` for one that is wrong.
fn config_result(result: Result<Config, ConfigError>) -> Value {
    match result {
        Ok(config) => json!({ "config": config_json(&config) }),
        Err(error) => {
            json!({ "problems": error.problems, "source": error.source, "message": error.to_string() })
        }
    }
}

/// The configuration `text` holds (its JSON), read as coming from `source`: `{config}`, or `{problems, message}`;
/// `text` is always JSON, since the host gives what it parsed.
#[must_use]
pub(crate) fn read_config(text: &str, source: &str) -> String {
    let value: Value = serde_json::from_str(text).unwrap_or(Value::Null);
    config_result(compmd_project::read_config(&value, source)).to_string()
}

/// `{error}`, or for a wrong configuration `{problems, message}`, for what stops reading a project.
fn project_error(error: ProjectError) -> Value {
    match error {
        ProjectError::Config(config) => config_result(Err(config)),
        other => json!({ "error": other.to_string() }),
    }
}

/// The configuration of `folder`: `{config}`, `{problems, message}`, or `{error}` (a file that isn't JSON, or a
/// configuration in two places).
#[must_use]
pub(crate) fn config_of(folder: &str, files: &impl ProjectFiles) -> String {
    match compmd_project::config_of(Path::new(folder), files) {
        Ok(config) => json!({ "config": config_json(&config) }),
        Err(error) => project_error(error),
    }
    .to_string()
}

/// A pack found, as JSON: `{name, version, dir, key?, integrity?}`.
fn pack_json(pack: &PackFolder) -> Value {
    let mut found = json!({
        "name": pack.release.pack.as_str(),
        "version": pack.release.version.to_string(),
        "dir": pack.dir.to_string_lossy(),
    });
    if let Some(key) = &pack.key {
        found["key"] = json!(key);
    }
    if let Some(integrity) = &pack.integrity {
        found["integrity"] = json!(integrity.to_string());
    }
    found
}

/// Packs found, `{packs}`, or `{error}`.
fn packs_result<F: ProjectFiles>(found: Result<PackFolders<F>, ProjectError>) -> String {
    match found {
        Ok(found) => json!({ "packs": found.packs().iter().map(pack_json).collect::<Vec<_>>() }),
        Err(error) => project_error(error),
    }
    .to_string()
}

/// The installed packs of the project at `root`, with their integrity from its lock file: `{packs}` or `{error}`.
#[must_use]
pub(crate) fn installed_packs(root: &str, files: &impl ProjectFiles) -> String {
    let integrities = compmd_project::integrities(Path::new(root), files);
    packs_result(PackFolders::installed(Path::new(root), files, &integrities))
}

/// The folder packs among the pack `keys` of a configuration in `folder`: `{packs}` or `{error}`.
#[must_use]
pub(crate) fn folder_packs(keys: &[String], folder: &str, files: &impl ProjectFiles) -> String {
    let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
    packs_result(PackFolders::folders(&keys, Path::new(folder), files))
}

/// Each release the lock file nearest above `root` holds, by `name@version`, with its integrity, as JSON.
#[must_use]
pub(crate) fn integrities(root: &str, files: &impl ProjectFiles) -> String {
    let found = compmd_project::integrities(Path::new(root), files);
    let by_release: serde_json::Map<String, Value> = found
        .iter()
        .map(|(release, integrity)| (release.to_string(), json!(integrity.to_string())))
        .collect();
    Value::Object(by_release).to_string()
}

/// The lock file nearest above `root`, if any.
#[must_use]
pub(crate) fn lock_file_of(root: &str, files: &impl ProjectFiles) -> Option<String> {
    compmd_project::lock_file_of(Path::new(root), files)
        .map(|(path, _)| path.to_string_lossy().into_owned())
}

/// The root folder of the document at `document`, among the editor's `workspace_folders`.
#[must_use]
pub(crate) fn root_of(
    document: &str,
    workspace_folders: &[String],
    files: &impl ProjectFiles,
) -> String {
    let workspaces: Vec<PathBuf> = workspace_folders.iter().map(PathBuf::from).collect();
    compmd_project::root_of(Path::new(document), &workspaces, files)
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests;
