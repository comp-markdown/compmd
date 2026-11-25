//! A document's root folder ([the language server], decision 3): the folder its relative paths resolve in, whose
//! configuration is the document's.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::path::{Path, PathBuf};

use crate::config_file::holds_config;
use crate::project_files::ProjectFiles;

/// The root folder of the document at `document`, the first that applies:
///
/// 1. the nearest folder above it that holds a configuration ([`holds_config`]);
/// 2. the deepest of `workspace_folders` (the editor's) that holds it;
/// 3. its own folder.
///
/// # Examples
///
/// ```
/// use std::path::{Path, PathBuf};
/// use compmd_project::{root_of, DiskFiles};
///
/// let root = root_of(Path::new("/nowhere/notes/a.comp.md"), &[PathBuf::from("/nowhere")], &DiskFiles);
/// assert_eq!(root, Path::new("/nowhere"));
/// ```
#[must_use]
pub fn root_of(
    document: &Path,
    workspace_folders: &[PathBuf],
    files: &impl ProjectFiles,
) -> PathBuf {
    let folder = document.parent().unwrap_or(document);
    if let Some(configured) = folder.ancestors().find(|dir| holds_config(dir, files)) {
        return configured.to_owned();
    }
    workspace_folders
        .iter()
        .filter(|workspace| document.starts_with(workspace))
        .max_by_key(|workspace| workspace.components().count())
        .cloned()
        .unwrap_or_else(|| folder.to_owned())
}

#[cfg(test)]
mod tests {
    use crate::project_files::tests::Memory;

    use super::*;

    /// A site whose `blog/` has a configuration of its own, in a workspace with a folder of notes.
    fn files() -> Memory {
        Memory::of([
            ("/work/site/compmd.config.json", "{}"),
            ("/work/site/blog/package.json", r#"{"compmd": {}}"#),
            (
                "/work/site/packs/kit/package.json",
                r#"{"name": "@acme/kit", "compmd": "./index.comp.md"}"#,
            ),
        ])
    }

    #[test]
    fn the_nearest_configuration_above_comes_first() {
        let workspace = [PathBuf::from("/work")];
        assert_eq!(
            root_of(
                Path::new("/work/site/posts/a.comp.md"),
                &workspace,
                &files()
            ),
            Path::new("/work/site")
        );
        assert_eq!(
            root_of(
                Path::new("/work/site/blog/2026/a.comp.md"),
                &workspace,
                &files()
            ),
            Path::new("/work/site/blog")
        );
        assert_eq!(
            root_of(Path::new("/work/site/a.comp.md"), &workspace, &files()),
            Path::new("/work/site")
        );
        assert_eq!(
            root_of(
                Path::new("/work/site/packs/kit/index.comp.md"),
                &workspace,
                &files()
            ),
            Path::new("/work/site"),
            "a pack's manifest holds no configuration"
        );
    }

    #[test]
    fn then_the_deepest_workspace_folder_that_holds_it() {
        let workspaces = [
            PathBuf::from("/work"),
            PathBuf::from("/work/notes"),
            PathBuf::from("/elsewhere"),
        ];
        assert_eq!(
            root_of(
                Path::new("/work/notes/2026/a.comp.md"),
                &workspaces,
                &files()
            ),
            Path::new("/work/notes")
        );
        assert_eq!(
            root_of(Path::new("/work/other/a.comp.md"), &workspaces, &files()),
            Path::new("/work")
        );
    }

    #[test]
    fn then_its_own_folder() {
        assert_eq!(
            root_of(
                Path::new("/tmp/drafts/a.comp.md"),
                &[PathBuf::from("/work")],
                &files()
            ),
            Path::new("/tmp/drafts")
        );
        assert_eq!(root_of(Path::new("/"), &[], &files()), Path::new("/"));
    }
}
