//! What stops a project from being read: its configuration, or its packs.

use std::path::PathBuf;

use crate::config_error::ConfigError;

/// Why a project can't be read, saying what to fix, in the words `@compmd/vite` uses.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ProjectError {
    /// A file that holds JSON doesn't.
    #[error("{} isn't JSON: {message}", path.display())]
    NotJson {
        /// The file.
        path: PathBuf,
        /// What the JSON reader found.
        message: String,
    },
    /// The folder has both `compmd.config.json` and `package.json`'s `compmd` object.
    #[error(
        "the configuration is in both compmd.config.json and package.json's `compmd` key: keep one"
    )]
    TwoConfigurations,
    /// The configuration is wrong.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// A pack's folder has no `package.json` with a name.
    #[error("{} isn't a pack: it has no package.json with a name", dir.display())]
    NotAPack {
        /// The folder.
        dir: PathBuf,
    },
    /// A pack's `package.json` names it with a name that isn't a pack's.
    #[error("{}: `{name}` isn't a pack's name: a pack is a scoped npm name, `@scope/pack`", dir.display())]
    PackName {
        /// The pack's folder.
        dir: PathBuf,
        /// The name it has.
        name: String,
    },
    /// A pack's version isn't semver.
    #[error("{name}: its version, `{version}`, isn't a semver version such as `1.2.0`")]
    PackVersion {
        /// The pack.
        name: String,
        /// The version it has, as written.
        version: String,
    },
    /// An installed pack's `package.json` names it otherwise than the dependency that installed it.
    #[error(
        "{dependency} is installed, but its package.json names it `{name}`: a pack is served by its own name"
    )]
    Renamed {
        /// The name the project depends on it by.
        dependency: String,
        /// The name its `package.json` gives.
        name: String,
    },
    /// A folder pack the configuration lists isn't a folder.
    #[error("{folder} is listed in the configuration, but it isn't a folder of the app")]
    NoFolder {
        /// The configuration's key, `./components`.
        folder: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_says_what_to_fix() {
        let not_json = ProjectError::NotJson {
            path: "/site/compmd.config.json".into(),
            message: "EOF while parsing".into(),
        };
        assert_eq!(
            not_json.to_string(),
            "/site/compmd.config.json isn't JSON: EOF while parsing"
        );
        assert_eq!(
            ProjectError::TwoConfigurations.to_string(),
            "the configuration is in both compmd.config.json and package.json's `compmd` key: keep one"
        );
        let config = ConfigError {
            source: "compmd.config.json".into(),
            problems: vec!["fetch: `host` or `direct`".into()],
        };
        assert_eq!(
            ProjectError::from(config).to_string(),
            "compmd.config.json:\n- fetch: `host` or `direct`"
        );
        assert_eq!(
            ProjectError::NotAPack {
                dir: "/site/kit".into()
            }
            .to_string(),
            "/site/kit isn't a pack: it has no package.json with a name"
        );
        assert_eq!(
            ProjectError::PackName {
                dir: "/site/kit".into(),
                name: "kit".into()
            }
            .to_string(),
            "/site/kit: `kit` isn't a pack's name: a pack is a scoped npm name, `@scope/pack`"
        );
        assert_eq!(
            ProjectError::PackVersion {
                name: "@acme/kit".into(),
                version: "1.0".into()
            }
            .to_string(),
            "@acme/kit: its version, `1.0`, isn't a semver version such as `1.2.0`"
        );
        assert_eq!(
            ProjectError::Renamed {
                dependency: "@acme/kit".into(),
                name: "@acme/other".into()
            }
            .to_string(),
            "@acme/kit is installed, but its package.json names it `@acme/other`: a pack is served by its own name"
        );
        assert_eq!(
            ProjectError::NoFolder {
                folder: "./components".into()
            }
            .to_string(),
            "./components is listed in the configuration, but it isn't a folder of the app"
        );
    }
}
