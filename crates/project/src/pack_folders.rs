//! The project's packs on its disk, as the processor's `Packs` ([the language server], decision 5): the installed
//! packs, found as `@compmd/vite` finds them, and the folder packs the configuration lists. A host chains them with
//! its other sources in the configuration's order.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::path::{Component, Path, PathBuf};

use compmd::host::{Packs, Release};
use compmd_pack::{PackName, Place};
use serde_json::Value;

use crate::config::{Config, is_folder};
use crate::lock_file::Integrities;
use crate::pack_folder::{PackFolder, is_pack, manifest_at};
use crate::project_error::ProjectError;
use crate::project_files::ProjectFiles;

/// Packs in folders of the project, read through `F` when the processor asks for their files.
#[derive(Clone, Debug)]
pub struct PackFolders<F> {
    files: F,
    packs: Vec<PackFolder>,
}

/// The project's direct dependencies, `dependencies` and `devDependencies`, by name, sorted.
fn dependencies_of(root: &Path, files: &impl ProjectFiles) -> Vec<String> {
    let manifest = manifest_at(root, files).unwrap_or(Value::Null);
    let mut names: Vec<String> = ["dependencies", "devDependencies"]
        .iter()
        .filter_map(|field| manifest[field].as_object())
        .flat_map(|dependencies| dependencies.keys().cloned())
        .collect();
    names.sort();
    names.dedup();
    names
}

/// `base` joined with `relative`, its `.` and `..` resolved without reading the disk.
fn joined(base: &Path, relative: &str) -> PathBuf {
    let mut path = base.to_owned();
    for component in Path::new(relative).components() {
        match component {
            Component::ParentDir => {
                path.pop();
            }
            Component::CurDir => {}
            other => path.push(other),
        }
    }
    path
}

impl<F: ProjectFiles> PackFolders<F> {
    /// The installed packs of the project at `root`: every scoped dependency in its `node_modules` that is a pack
    /// ([the Vite plugin], decision 3), each with its integrity from `integrities`.
    ///
    /// # Errors
    ///
    /// A [`ProjectError`] when an installed pack's `package.json` isn't a pack's ([`PackFolder::read`]).
    ///
    /// [the Vite plugin]: https://github.com/comp-markdown/compmd/blob/main/docs/vite.md
    pub fn installed(
        root: &Path,
        files: F,
        integrities: &Integrities,
    ) -> Result<Self, ProjectError> {
        let mut packs = Vec::new();
        for name in dependencies_of(root, &files)
            .iter()
            .filter(|name| name.starts_with('@'))
        {
            let dir = name
                .split('/')
                .fold(root.join("node_modules"), |dir, part| dir.join(part));
            if files.is_dir(&dir) && is_pack(&dir, &files) {
                let mut pack = PackFolder::read(&dir, Some(name), &files)?;
                pack.integrity = integrities.get(&pack.release).cloned();
                packs.push(pack);
            }
        }
        Ok(Self { files, packs })
    }

    /// The folder packs `config` lists, relative to `folder`, the configuration's.
    ///
    /// # Errors
    ///
    /// [`ProjectError::NoFolder`] when a listed folder isn't one, and a [`ProjectError`] when it isn't a pack
    /// ([`PackFolder::read`]).
    pub fn listed(config: &Config, folder: &Path, files: F) -> Result<Self, ProjectError> {
        let keys: Vec<&str> = config.packs.iter().map(|(key, _)| key.as_str()).collect();
        Self::folders(&keys, folder, files)
    }

    /// The folder packs among the configuration's pack `keys` (`./components`; pack names are left out), relative to
    /// `folder`, the configuration's.
    ///
    /// # Errors
    ///
    /// As [`PackFolders::listed`].
    pub fn folders(keys: &[&str], folder: &Path, files: F) -> Result<Self, ProjectError> {
        let mut packs = Vec::new();
        for key in keys.iter().copied().filter(|key| is_folder(key)) {
            let dir = joined(folder, key);
            if !files.is_dir(&dir) {
                return Err(ProjectError::NoFolder {
                    folder: key.to_owned(),
                });
            }
            let mut pack = PackFolder::read(&dir, None, &files)?;
            pack.key = Some(key.to_owned());
            packs.push(pack);
        }
        Ok(Self { files, packs })
    }

    /// The packs found.
    #[must_use]
    pub fn packs(&self) -> &[PackFolder] {
        &self.packs
    }

    /// The pack name of the folder pack the configuration lists as `key`, `./components`.
    #[must_use]
    pub fn name_of(&self, key: &str) -> Option<PackName> {
        self.packs
            .iter()
            .find(|pack| pack.key.as_deref() == Some(key))
            .map(|pack| pack.release.pack.clone())
    }
}

impl<F: ProjectFiles> Packs for PackFolders<F> {
    fn releases(&self, pack: &PackName) -> Vec<Release> {
        self.packs
            .iter()
            .filter(|found| found.release.pack == *pack)
            .map(|found| Release {
                version: found.release.version.clone(),
                integrity: found.integrity.clone(),
                broken: None,
            })
            .collect()
    }

    fn pack_file(&self, place: &Place) -> Option<String> {
        let found = self
            .packs
            .iter()
            .find(|found| found.release == place.release)?;
        let path = place
            .path
            .parts()
            .fold(found.dir.clone(), |path, part| path.join(part));
        self.files.text(&path)
    }
}

#[cfg(test)]
mod tests;
