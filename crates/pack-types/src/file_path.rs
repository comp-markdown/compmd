//! A file's path in a pack: `/`-separated names that can't leave the pack's folder.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::invalid::Invalid;

/// A file's path in a pack, `dist/mermaid.js`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FilePath(String);

impl FilePath {
    /// The path `text`, when it is one: `/`-separated names, none of them empty, `.` or `..`, with no `\` or `:` (a
    /// Windows path of its own), so it can't name a file outside the pack.
    ///
    /// # Errors
    ///
    /// [`Invalid::Path`] when it isn't one.
    pub fn new(text: impl Into<String>) -> Result<Self, Invalid> {
        let text = text.into();
        if text.split('/').all(is_name) {
            Ok(Self(text))
        } else {
            Err(Invalid::Path(text))
        }
    }

    /// The path as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Its names, in order.
    pub fn parts(&self) -> impl Iterator<Item = &str> {
        self.0.split('/')
    }

    /// The file `relative` (`./card.comp.md`, `../ui/x.jsx`) names, seen from this file's folder; `None` when it
    /// leaves the pack, or names its folder.
    #[must_use]
    pub fn join(&self, relative: &str) -> Option<Self> {
        Self::new(self.joined_names(relative)?.join("/")).ok()
    }

    /// The folder `relative` (`./`, `../guide/`) names, seen from this file's folder: its `/`-separated path, empty
    /// for the root folder; `None` when it leaves the root, or one of its names isn't a file's name.
    ///
    /// ```
    /// # use compmd_pack::FilePath;
    /// let post = FilePath::new("posts/harbour.comp.md").expect("a path");
    /// assert_eq!(post.join_folder("../guide/").as_deref(), Some("guide"));
    /// assert_eq!(post.join_folder("../").as_deref(), Some(""));
    /// assert_eq!(post.join_folder("../../"), None);
    /// ```
    #[must_use]
    pub fn join_folder(&self, relative: &str) -> Option<String> {
        let names = self.joined_names(relative)?;
        names
            .iter()
            .all(|name| is_name(name))
            .then(|| names.join("/"))
    }

    /// The names of what `relative` names, seen from this file's folder; `None` when `..` leaves the root.
    fn joined_names<'a>(&'a self, relative: &'a str) -> Option<Vec<&'a str>> {
        let mut names: Vec<&str> = self.parts().collect();
        names.pop();
        for part in relative.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    names.pop()?;
                }
                name => names.push(name),
            }
        }
        Some(names)
    }
}

/// Whether `part` is a file's or a folder's name: not empty, `.` or `..`, with no `\` or `:` (a Windows path of its
/// own) and no NUL.
fn is_name(part: &str) -> bool {
    !part.is_empty() && part != "." && part != ".." && !part.contains(['\\', ':', '\0'])
}

impl FromStr for FilePath {
    type Err = Invalid;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::new(text)
    }
}

impl TryFrom<String> for FilePath {
    type Error = Invalid;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::new(text)
    }
}

impl From<FilePath> for String {
    fn from(path: FilePath) -> Self {
        path.0
    }
}

impl fmt::Display for FilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(text: &str) -> FilePath {
        FilePath::new(text).expect(text)
    }

    #[test]
    fn a_path_is_plain_names() {
        let file = path("dist/mermaid.js");
        assert_eq!(file.as_str(), "dist/mermaid.js");
        assert_eq!(file.parts().collect::<Vec<_>>(), ["dist", "mermaid.js"]);
        assert_eq!(file.to_string(), "dist/mermaid.js");
        assert_eq!("a.js".parse::<FilePath>(), Ok(path("a.js")));
    }

    #[test]
    fn a_path_that_could_leave_the_pack_is_refused() {
        for text in [
            "",
            "/a.js",
            "a//b.js",
            "./a.js",
            "a/../b.js",
            "..",
            "a\\b.js",
            "C:/a.js",
            "a\0b",
        ] {
            assert_eq!(
                FilePath::new(text),
                Err(Invalid::Path(text.to_owned())),
                "{text:?}"
            );
        }
        assert!(
            Invalid::Path("x".to_owned())
                .to_string()
                .contains("isn't a file's path")
        );
    }

    #[test]
    fn a_relative_path_resolves_against_the_files_folder() {
        let kit = path("kit.comp.md");
        assert_eq!(
            kit.join("./parts/chip.comp.md"),
            Some(path("parts/chip.comp.md"))
        );
        assert_eq!(
            path("docs/a.comp.md").join("../ui/b.jsx"),
            Some(path("ui/b.jsx"))
        );
        assert_eq!(
            path("docs/a.comp.md").join("./x/../y.comp.md"),
            Some(path("docs/y.comp.md"))
        );
        assert_eq!(kit.join("../x.comp.md"), None, "it leaves the pack");
        assert_eq!(kit.join("./"), None, "it names a folder");
        assert_eq!(kit.join("./a\\b.js"), None, "a name with `\\`");
    }

    #[test]
    fn a_relative_folder_resolves_against_the_files_folder() {
        let post = path("posts/harbour.comp.md");
        assert_eq!(post.join_folder("./").as_deref(), Some("posts"));
        assert_eq!(post.join_folder(".").as_deref(), Some("posts"));
        assert_eq!(post.join_folder("../guide/").as_deref(), Some("guide"));
        assert_eq!(post.join_folder("..").as_deref(), Some(""));
        assert_eq!(post.join_folder("../../"), None, "it leaves the root");
        assert_eq!(post.join_folder("./a:b/"), None, "a name with `:`");
    }

    #[test]
    fn a_path_is_a_string_in_json() {
        let file: FilePath = serde_json::from_str(r#""dist/a.js""#).expect("a path");
        assert_eq!(
            serde_json::to_string(&file).expect("JSON"),
            r#""dist/a.js""#
        );
        assert!(serde_json::from_str::<FilePath>(r#""../a.js""#).is_err());
        assert_eq!(String::from(file), "dist/a.js");
        assert_eq!(FilePath::try_from("a.js".to_owned()), Ok(path("a.js")));
    }
}
