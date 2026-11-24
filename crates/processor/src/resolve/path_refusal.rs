//! Why a relative path names no file (spec §8.6): it leaves its root, isn't a path, or is written in a document whose
//! own path the host gave isn't one. Each is F026 where the path is written.

use std::fmt;

/// Why the relative path `relative` names no file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PathRefusal {
    /// `..` leads out of the root folder or the pack's, or the path names a folder.
    Leaves { relative: String },
    /// A name has `\`, `:` or NUL: a Windows path of its own, which could lead anywhere.
    NotAPath { relative: String },
    /// The document it is written in has no place in its root: the host gave it the path `document`, which isn't one.
    Unplaced { relative: String, document: String },
}

impl fmt::Display for PathRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Leaves { relative } => write!(f, "`{relative}` leaves its root"),
            Self::NotAPath { relative } => {
                write!(
                    f,
                    "`{relative}` isn't a file's path: a name has `\\`, `:` or NUL"
                )
            }
            Self::Unplaced { relative, document } => write!(
                f,
                "`{relative}` can't be resolved: the document's path, `{document}`, isn't a `/`-separated path in its root"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_refusal_says_why() {
        let relative = "../x.csv".to_string();
        assert_eq!(
            PathRefusal::Leaves {
                relative: relative.clone()
            }
            .to_string(),
            "`../x.csv` leaves its root"
        );
        assert!(
            PathRefusal::NotAPath {
                relative: "a\\b".into()
            }
            .to_string()
            .contains("isn't a file's path")
        );
        assert!(
            PathRefusal::Unplaced {
                relative,
                document: "C:\\a.comp.md".into()
            }
            .to_string()
            .contains("the document's path, `C:\\a.comp.md`,")
        );
    }
}
