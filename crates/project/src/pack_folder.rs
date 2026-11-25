//! A pack's folder (spec §6.4): a `package.json` with a pack's name and a semver version, and a CompMD document.

use std::path::{Path, PathBuf};

use compmd_pack::{Integrity, ReleaseId};
use semver::Version;
use serde_json::Value;

use crate::project_error::ProjectError;
use crate::project_files::ProjectFiles;

/// A pack found in the project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackFolder {
    /// Its name and version, from its `package.json`.
    pub release: ReleaseId,
    /// Where its files are.
    pub dir: PathBuf,
    /// The configuration's key that lists it, `./components`, for a folder pack.
    pub key: Option<String>,
    /// The integrity the registry published for it, from the lock file, for an installed pack.
    pub integrity: Option<Integrity>,
}

/// The `package.json` at `dir`, when there is one and it is a JSON object.
pub(crate) fn manifest_at(dir: &Path, files: &impl ProjectFiles) -> Option<Value> {
    let manifest: Value = serde_json::from_str(&files.text(&dir.join("package.json"))?).ok()?;
    manifest.is_object().then_some(manifest)
}

/// Whether the package at `dir` is a pack: its manifest names a document, or it has `index.comp.md`.
pub(crate) fn is_pack(dir: &Path, files: &impl ProjectFiles) -> bool {
    manifest_at(dir, files).is_some_and(|manifest| {
        manifest["compmd"].is_string() || files.text(&dir.join("index.comp.md")).is_some()
    })
}

/// A version as JavaScript's `String` writes it: a string as it is, anything else as JSON.
fn written(version: &Value) -> String {
    version
        .as_str()
        .map_or_else(|| version.to_string(), str::to_owned)
}

impl PackFolder {
    /// The pack at `dir`, by its `package.json`. `dependency` is the name the project installed it by, for a package
    /// in `node_modules`; its version is `0.0.0` when it has none.
    ///
    /// # Errors
    ///
    /// [`ProjectError::NotAPack`] when `dir` has no `package.json` with a name, [`ProjectError::PackName`] when its
    /// name isn't a pack's, [`ProjectError::PackVersion`] when its version isn't semver, and
    /// [`ProjectError::Renamed`] when its name isn't `dependency`.
    pub fn read(
        dir: &Path,
        dependency: Option<&str>,
        files: &impl ProjectFiles,
    ) -> Result<Self, ProjectError> {
        let not_a_pack = || ProjectError::NotAPack {
            dir: dir.to_owned(),
        };
        let manifest = manifest_at(dir, files).ok_or_else(not_a_pack)?;
        let name = manifest["name"].as_str().ok_or_else(not_a_pack)?;
        let pack = name.parse().map_err(|_| ProjectError::PackName {
            dir: dir.to_owned(),
            name: name.to_owned(),
        })?;
        let version = manifest.get("version").map_or_else(
            || Some(Version::new(0, 0, 0)),
            |version| {
                let text = version.as_str()?;
                Version::parse(text)
                    .ok()
                    .filter(|parsed| parsed.to_string() == text)
            },
        );
        let version = version.ok_or_else(|| ProjectError::PackVersion {
            name: name.to_owned(),
            version: written(&manifest["version"]),
        })?;
        if let Some(dependency) = dependency.filter(|dependency| *dependency != name) {
            return Err(ProjectError::Renamed {
                dependency: dependency.to_owned(),
                name: name.to_owned(),
            });
        }
        Ok(Self {
            release: ReleaseId::new(pack, version),
            dir: dir.to_owned(),
            key: None,
            integrity: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::project_files::tests::Memory;

    use super::*;

    fn read(manifest: &str, dependency: Option<&str>) -> Result<PackFolder, ProjectError> {
        PackFolder::read(
            Path::new("/kit"),
            dependency,
            &Memory::of([("/kit/package.json", manifest)]),
        )
    }

    #[test]
    fn a_pack_has_a_scoped_name_and_a_semver_version() {
        let pack = read(
            r#"{"name": "@acme/kit", "version": "1.2.0"}"#,
            Some("@acme/kit"),
        )
        .expect("a pack");
        assert_eq!(pack.release.to_string(), "@acme/kit@1.2.0");
        assert_eq!(pack.dir, Path::new("/kit"));
        assert_eq!(
            read(r#"{"name": "@acme/kit"}"#, None).map(|pack| pack.release.to_string()),
            Ok("@acme/kit@0.0.0".into())
        );
    }

    #[test]
    fn anything_else_says_what_is_wrong() {
        let dir = PathBuf::from("/kit");
        assert_eq!(
            read("[]", None),
            Err(ProjectError::NotAPack { dir: dir.clone() })
        );
        assert_eq!(
            read(r#"{"version": "1.0.0"}"#, None),
            Err(ProjectError::NotAPack { dir: dir.clone() })
        );
        assert_eq!(
            PackFolder::read(&dir, None, &Memory::default()),
            Err(ProjectError::NotAPack { dir: dir.clone() })
        );
        assert_eq!(
            read(r#"{"name": "kit"}"#, None),
            Err(ProjectError::PackName {
                dir,
                name: "kit".into()
            })
        );
        for (version, written) in [
            (r#""1.0""#, "1.0"),
            (r#""v1.0.0""#, "v1.0.0"),
            ("1", "1"),
            ("null", "null"),
        ] {
            assert_eq!(
                read(
                    &format!(r#"{{"name": "@acme/kit", "version": {version}}}"#),
                    None
                ),
                Err(ProjectError::PackVersion {
                    name: "@acme/kit".into(),
                    version: written.into()
                })
            );
        }
        assert_eq!(
            read(
                r#"{"name": "@acme/other", "version": "1.0.0"}"#,
                Some("@acme/kit")
            ),
            Err(ProjectError::Renamed {
                dependency: "@acme/kit".into(),
                name: "@acme/other".into()
            })
        );
    }

    #[test]
    fn a_package_is_a_pack_with_a_document() {
        let files = Memory::of([
            (
                "/named/package.json",
                r#"{"name": "@acme/named", "compmd": "./docs/kit.comp.md"}"#,
            ),
            ("/index/package.json", r#"{"name": "@acme/index"}"#),
            ("/index/index.comp.md", "# Kit"),
            ("/library/package.json", r#"{"name": "@acme/library"}"#),
            ("/broken/package.json", "{"),
        ]);
        assert!(is_pack(Path::new("/named"), &files));
        assert!(is_pack(Path::new("/index"), &files));
        assert!(!is_pack(Path::new("/library"), &files));
        assert!(!is_pack(Path::new("/broken"), &files));
        assert!(!is_pack(Path::new("/none"), &files));
    }
}
