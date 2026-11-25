//! A document's project, read as its hosts read it (`compmd-project`; [the language server], decision 4): its folder
//! packs, its installed packs, and what its configuration gives the processor.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::BTreeMap;
use std::path::Path;

use compmd_project::{DiskFiles, PackFolders, Translated, config_of, integrities};

/// A project as its documents are processed in it.
pub(crate) struct Project {
    /// The folder packs its configuration lists.
    pub(crate) listed: PackFolders<DiskFiles>,
    /// The packs installed in its `node_modules`.
    pub(crate) installed: PackFolders<DiskFiles>,
    /// What its configuration gives the processor: its policy, its ambient packs, its allowed origins.
    pub(crate) translated: Translated,
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
        })
    }
}
