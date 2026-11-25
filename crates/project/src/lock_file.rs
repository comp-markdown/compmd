//! The integrity the registry published for each installed package, from the project's lock file ([the Vite plugin],
//! V1): `pnpm-lock.yaml` or `package-lock.json`, the nearest above the root, as a workspace keeps one for all its
//! packages.
//!
//! [the Vite plugin]: https://github.com/comp-markdown/compmd/blob/main/docs/vite.md

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use compmd_pack::{Integrity, ReleaseId};
use serde_json::Value;
use yaml_rust2::YamlLoader;

use crate::project_files::ProjectFiles;

/// Each installed release's integrity, by the release.
pub type Integrities = BTreeMap<ReleaseId, Integrity>;

/// The lock files a project may have, in the order they are looked for in a folder.
const LOCK_FILES: [&str; 2] = ["pnpm-lock.yaml", "package-lock.json"];

/// The lock file nearest above `root`, at it or in a folder above, and its text.
#[must_use]
pub fn lock_file_of(root: &Path, files: &impl ProjectFiles) -> Option<(PathBuf, String)> {
    root.ancestors().find_map(|dir| {
        LOCK_FILES.iter().find_map(|name| {
            let path = dir.join(name);
            files.text(&path).map(|text| (path, text))
        })
    })
}

/// `id` and `integrity` as an entry, when both are a scoped package's release and an integrity.
fn entry(id: &str, integrity: Option<&str>) -> Option<(ReleaseId, Integrity)> {
    Some((id.parse().ok()?, integrity?.parse().ok()?))
}

/// The integrities in `pnpm-lock.yaml`. pnpm may write documents of its own before the project's lock (its settings,
/// or an environment lock for a toolchain pinned with `devEngines`); each document's packages count.
fn pnpm_integrities(text: &str) -> Integrities {
    let documents = YamlLoader::load_from_str(text).unwrap_or_default();
    documents
        .iter()
        .filter_map(|document| document["packages"].as_hash())
        .flatten()
        .filter_map(|(key, package)| {
            // `@kit/mermaid@1.0.0`, or `/@kit/mermaid@1.0.0` in older lock files; a peer suffix `(react@19.0.0)` follows.
            let key = key.as_str()?;
            let id = key.strip_prefix('/').unwrap_or(key);
            let id = id.split_once('(').map_or(id, |(id, _)| id);
            entry(id, package["resolution"]["integrity"].as_str())
        })
        .collect()
}

/// The integrities in `package-lock.json`, whose packages are keyed by their folder, `node_modules/@kit/mermaid`.
fn npm_integrities(text: &str) -> Integrities {
    let Ok(lock) = serde_json::from_str::<Value>(text) else {
        return Integrities::new();
    };
    lock["packages"]
        .as_object()
        .into_iter()
        .flatten()
        .filter_map(|(path, package)| {
            let name = package["name"]
                .as_str()
                .or_else(|| path.rsplit("node_modules/").next())?;
            let version = package["version"].as_str()?;
            entry(&format!("{name}@{version}"), package["integrity"].as_str())
        })
        .collect()
}

/// The integrity the registry published for each package the project's lock file holds: empty without one, or with
/// one that can't be read.
#[must_use]
pub fn integrities(root: &Path, files: &impl ProjectFiles) -> Integrities {
    match lock_file_of(root, files) {
        Some((path, text))
            if path
                .extension()
                .is_some_and(|extension| extension == "yaml") =>
        {
            pnpm_integrities(&text)
        }
        Some((_, text)) => npm_integrities(&text),
        None => Integrities::new(),
    }
}

#[cfg(test)]
mod tests {
    use crate::project_files::tests::Memory;

    use super::*;

    fn release(text: &str) -> ReleaseId {
        text.parse().expect("a release")
    }

    const PNPM: &str = "\
---
settings:
  autoInstallPeers: true
---
lockfileVersion: '9.0'
packages:
  '@kit/mermaid@1.0.0':
    resolution: {integrity: sha512-mermaid}
  '/@kit/old@0.1.0':
    resolution: {integrity: sha512-old}
  '@kit/charts@2.0.0(react@19.0.0)':
    resolution: {integrity: sha512-charts}
  'd3@7.9.0':
    resolution: {integrity: sha512-d3}
  '@kit/linked@1.0.0':
    resolution: {directory: ../kit}
";

    #[test]
    fn reads_pnpms_packages_in_every_document_without_peer_suffixes() {
        let files = Memory::of([("/work/pnpm-lock.yaml", PNPM)]);
        let found = integrities(Path::new("/work/site"), &files);
        assert_eq!(
            found.keys().map(ToString::to_string).collect::<Vec<_>>(),
            ["@kit/charts@2.0.0", "@kit/mermaid@1.0.0", "@kit/old@0.1.0"],
            "scoped releases with an integrity"
        );
        assert_eq!(
            found[&release("@kit/mermaid@1.0.0")].to_string(),
            "sha512-mermaid"
        );
    }

    #[test]
    fn reads_npms_packages_by_their_folder_or_their_name() {
        let lock = r#"{"packages": {
            "": {"name": "site"},
            "node_modules/@kit/mermaid": {"version": "1.0.0", "integrity": "sha512-mermaid"},
            "node_modules/@kit/charts/node_modules/@kit/base": {"version": "0.2.0", "integrity": "sha512-base"},
            "packs/notes": {"name": "@site/notes", "version": "1.0.0", "integrity": "sha512-notes"},
            "node_modules/@kit/unlocked": {"version": "1.0.0"}
        }}"#;
        let files = Memory::of([("/site/package-lock.json", lock)]);
        let found = integrities(Path::new("/site"), &files);
        assert_eq!(
            found.keys().map(ToString::to_string).collect::<Vec<_>>(),
            ["@kit/base@0.2.0", "@kit/mermaid@1.0.0", "@site/notes@1.0.0"]
        );
    }

    #[test]
    fn the_nearest_lock_file_counts_pnpms_first() {
        let files = Memory::of([
            ("/work/pnpm-lock.yaml", PNPM),
            ("/work/package-lock.json", "{}"),
            (
                "/work/site/package-lock.json",
                r#"{"packages": {"node_modules/@kit/x": {"version": "1.0.0", "integrity": "sha512-x"}}}"#,
            ),
        ]);
        assert_eq!(
            lock_file_of(Path::new("/work/site/posts"), &files).map(|(path, _)| path),
            Some(PathBuf::from("/work/site/package-lock.json"))
        );
        assert_eq!(
            lock_file_of(Path::new("/work"), &files).map(|(path, _)| path),
            Some(PathBuf::from("/work/pnpm-lock.yaml"))
        );
        assert_eq!(integrities(Path::new("/work/site"), &files).len(), 1);
    }

    #[test]
    fn none_or_one_that_cant_be_read_holds_nothing() {
        assert!(integrities(Path::new("/site"), &Memory::default()).is_empty());
        assert!(
            integrities(
                Path::new("/site"),
                &Memory::of([("/site/package-lock.json", "{")])
            )
            .is_empty()
        );
        assert!(
            integrities(
                Path::new("/site"),
                &Memory::of([("/site/pnpm-lock.yaml", "packages: [")])
            )
            .is_empty()
        );
    }
}
