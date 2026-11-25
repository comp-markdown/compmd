//! How the crate reads a project's files: through its host, so the same code reads a disk, an editor's open files or
//! what a WebAssembly host passes in.

use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;

/// A project's files, as its host reads them.
pub trait ProjectFiles {
    /// The text of the file at `path`, or `None` when there is none, it is a folder, or it isn't UTF-8.
    fn text(&self, path: &Path) -> Option<String>;

    /// Whether `path` is a folder.
    fn is_dir(&self, path: &Path) -> bool;
}

impl<T: ProjectFiles + ?Sized> ProjectFiles for &T {
    fn text(&self, path: &Path) -> Option<String> {
        (**self).text(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        (**self).is_dir(path)
    }
}

impl<T: ProjectFiles + ?Sized> ProjectFiles for Rc<T> {
    fn text(&self, path: &Path) -> Option<String> {
        (**self).text(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        (**self).is_dir(path)
    }
}

impl<T: ProjectFiles + ?Sized> ProjectFiles for Arc<T> {
    fn text(&self, path: &Path) -> Option<String> {
        (**self).text(path)
    }

    fn is_dir(&self, path: &Path) -> bool {
        (**self).is_dir(path)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use std::rc::Rc;
    use std::sync::Arc;

    use super::ProjectFiles;

    /// A project in memory: its files by path; every folder above a file is a folder.
    #[derive(Default)]
    pub(crate) struct Memory(BTreeMap<PathBuf, String>);

    impl Memory {
        /// The project of `files`, by path and text.
        pub(crate) fn of<'a>(files: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
            Self(
                files
                    .into_iter()
                    .map(|(path, text)| (PathBuf::from(path), text.to_owned()))
                    .collect(),
            )
        }
    }

    impl ProjectFiles for Memory {
        fn text(&self, path: &Path) -> Option<String> {
            self.0.get(path).cloned()
        }

        fn is_dir(&self, path: &Path) -> bool {
            self.0
                .keys()
                .any(|file| file != path && file.starts_with(path))
        }
    }

    /// What `files` holds at `path`, and whether `folder` is a folder, read through `files` as given.
    fn read(files: impl ProjectFiles, path: &str, folder: &str) -> (Option<String>, bool) {
        (files.text(Path::new(path)), files.is_dir(Path::new(folder)))
    }

    #[test]
    fn a_reference_and_a_shared_pointer_read_what_they_point_to() {
        let files = Memory::of([("/a/b.txt", "B")]);
        assert_eq!(read(&files, "/a/b.txt", "/a"), (Some("B".into()), true));
        let shared = Rc::new(files);
        assert_eq!(
            read(Rc::clone(&shared), "/a/b.txt", "/a/b.txt"),
            (Some("B".into()), false)
        );
        let shared = Arc::new(Memory::of([("/a/b.txt", "B")]));
        assert_eq!(read(shared, "/a", "/"), (None, true));
    }
}
