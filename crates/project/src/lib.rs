//! `compmd-project`: reading a [CompMD](https://compmd.dev/) project as its hosts do ([the language server], decisions 3
//! and 5): a document's root folder, the configuration there (`compmd.config.json`, or `package.json`'s `compmd`
//! object, [the Vite plugin], decisions 1 and 2), its folder packs and installed packs, and their integrity from the
//! lock file.
//!
//! The crate does no I/O of its own: it reads through [`ProjectFiles`], which [`DiskFiles`] implements with `std::fs`,
//! and a WebAssembly host with the functions it is given. The pack sources are the processor's own `Packs`, so a
//! document processed with them resolves its imports as the site will.
//!
//! ```no_run
//! use std::path::Path;
//! use compmd_project::{config_of, integrities, root_of, DiskFiles, PackFolders};
//!
//! let document = Path::new("/site/posts/hello.comp.md");
//! let root = root_of(document, &[], &DiskFiles);
//! let config = config_of(&root, &DiskFiles)?;
//! let installed = PackFolders::installed(&root, DiskFiles, &integrities(&root, &DiskFiles))?;
//! let listed = PackFolders::listed(&config, &root, DiskFiles)?;
//! let translated = config.translated(|key| listed.name_of(key).or_else(|| key.parse().ok()), &Default::default());
//! # Ok::<(), compmd_project::ProjectError>(())
//! ```
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md
//! [the Vite plugin]: https://github.com/comp-markdown/compmd/blob/main/docs/vite.md

mod config;
mod config_error;
mod config_file;
mod disk_files;
mod lock_file;
mod names;
mod origin;
mod pack_folder;
mod pack_folders;
mod project_error;
mod project_files;
mod root;
mod size;
mod translated;

pub use config::{Config, Editor, Fetch, Library, ListedPack, Origins, is_folder, read_config};
pub use config_error::ConfigError;
pub use config_file::{config_of, holds_config};
pub use disk_files::DiskFiles;
pub use lock_file::{Integrities, integrities, lock_file_of};
pub use pack_folder::PackFolder;
pub use pack_folders::PackFolders;
pub use project_error::ProjectError;
pub use project_files::ProjectFiles;
pub use root::root_of;
pub use translated::Translated;
