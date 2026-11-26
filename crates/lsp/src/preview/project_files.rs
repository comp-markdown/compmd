//! The project's files as the preview's page loads them by URL (docs/editor-support.md, decision 8): what a site serves
//! beside its documents, images, fonts and the rest, from the project's folder, under `files/`. A path never leads out
//! of the folder, by `..` or by a link.

use std::fs;
use std::path::{Path, PathBuf};

use percent_encoding::percent_decode_str;

/// The files under a project's root.
pub(crate) struct ProjectFiles {
    root: PathBuf,
}

/// The path `url_path` names under a folder, its parts decoded; `None` when a part is empty, `.`, `..`, or holds a
/// separator, which a file's path in a URL never needs.
fn relative_path(url_path: &str) -> Option<PathBuf> {
    url_path
        .split('/')
        .map(|part| {
            let part = percent_decode_str(part).decode_utf8().ok()?;
            let plain =
                !part.is_empty() && part != "." && part != ".." && !part.contains(['/', '\\']);
            plain.then(|| part.into_owned())
        })
        .collect()
}

impl ProjectFiles {
    /// The files under `root`.
    pub(crate) fn new(root: &Path) -> Self {
        Self {
            root: root.to_owned(),
        }
    }

    /// The file at `url_path` (its URL's path under `files/`), when it is a file inside the root.
    pub(crate) fn file(&self, url_path: &str) -> Option<Vec<u8>> {
        let root = fs::canonicalize(&self.root).ok()?;
        let path = fs::canonicalize(root.join(relative_path(url_path)?)).ok()?;
        if !path.starts_with(&root) || !path.is_file() {
            return None;
        }
        fs::read(path).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_in_a_url_is_a_path_under_the_folder_or_none() {
        assert_eq!(
            relative_path("posts/the%20frost.png"),
            Some(PathBuf::from("posts").join("the frost.png"))
        );
        for outside in [
            "../x.png",
            "a/../../x.png",
            "a//b",
            "./a",
            "a%2F..%2Fb",
            "a%5Cb",
            "",
            "%FF",
        ] {
            assert_eq!(relative_path(outside), None, "{outside}");
        }
    }

    #[test]
    fn a_file_is_read_only_inside_the_root() {
        let dir =
            std::env::temp_dir().join(format!("compmd-lsp-project-files-{}", std::process::id()));
        let root = dir.join("site");
        fs::create_dir_all(root.join("images")).expect("a folder");
        fs::write(root.join("images/frost.png"), b"png").expect("written");
        fs::write(dir.join("secret.txt"), b"no").expect("written");
        let files = ProjectFiles::new(&root);
        assert_eq!(files.file("images/frost.png").as_deref(), Some(&b"png"[..]));
        assert_eq!(files.file("images"), None, "a folder");
        assert_eq!(files.file("images/none.png"), None);
        assert_eq!(files.file("../secret.txt"), None);
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(dir.join("secret.txt"), root.join("link.txt"))
                .expect("a link");
            assert_eq!(files.file("link.txt"), None, "a link leading out");
        }
        fs::remove_dir_all(dir).expect("removed");
    }
}
