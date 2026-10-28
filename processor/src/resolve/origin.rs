//! Where a document comes from: a file of the root folder, or a file of a pack. The origin names the document in
//! diagnostics (spec §10.4) and decides where its relative paths lead (spec §8.6).

use semver::Version;
use std::fmt;

/// The pack the processor builds in (spec §9).
pub(crate) const CORE: &str = "@compmd/core";
/// The scope of `@compmd/core`, reserved by the spec: never fetched from a registry (spec §6.4).
pub(crate) const RESERVED_SCOPE: &str = "@compmd/";

/// Where a document comes from.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Origin {
    /// A file of the document's root folder, by its `/`-separated path from it.
    File(String),
    /// A file of a pack, by its path in the package: one the host's `Packs` serve, imported or ambient, or
    /// `@compmd/core`, which the processor builds in.
    Pack {
        /// The pack's name, `@scope/name`.
        name: String,
        /// The release.
        version: Version,
        /// The file's path in the package.
        path: String,
    },
}

impl Origin {
    /// `@compmd/core`'s document, at the version it exports.
    pub(crate) fn core() -> Self {
        Self::Pack {
            name: CORE.to_string(),
            version: Version::new(1, 0, 0),
            path: "index.comp.md".to_string(),
        }
    }

    /// Whether this is `@compmd/core`'s document.
    pub(crate) fn is_core(&self) -> bool {
        *self == Self::core()
    }

    /// The file `relative` (`./card.comp.md`, `../ui/x.jsx`) names, seen from this document; `None` when it leaves
    /// the root: the document's root folder, or the pack's.
    pub(crate) fn join(&self, relative: &str) -> Option<Self> {
        match self {
            Self::File(path) => joined(path, relative).map(Self::File),
            Self::Pack {
                name,
                version,
                path,
            } => joined(path, relative).map(|path| Self::Pack {
                name: name.clone(),
                version: version.clone(),
                path,
            }),
        }
    }

    /// The file's path in its root folder or its pack.
    pub(crate) fn path(&self) -> &str {
        match self {
            Self::File(path) | Self::Pack { path, .. } => path,
        }
    }
}

/// How diagnostics name the document: `ui.comp.md`, `@myorg/ui@1.2.0/card.comp.md` (spec §10.4).
impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(path) => f.write_str(path),
            Self::Pack {
                name,
                version,
                path,
            } => write!(f, "{name}@{version}/{path}"),
        }
    }
}

/// `relative` resolved against the folder of the file at `base`; `None` when `..` leaves the root.
fn joined(base: &str, relative: &str) -> Option<String> {
    let mut parts: Vec<&str> = base.split('/').collect();
    parts.pop();
    for part in relative.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            name => parts.push(name),
        }
    }
    Some(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str) -> Origin {
        Origin::File(path.to_string())
    }

    #[test]
    fn relative_paths_resolve_against_the_documents_folder() {
        assert_eq!(
            file("input.comp.md").join("./ui.comp.md"),
            Some(file("ui.comp.md"))
        );
        assert_eq!(
            file("docs/a.comp.md").join("../ui/b.jsx"),
            Some(file("ui/b.jsx"))
        );
        assert_eq!(
            file("docs/a.comp.md").join("./x/../y.comp.md"),
            Some(file("docs/y.comp.md"))
        );
        assert_eq!(file("a.comp.md").join("../b.comp.md"), None);
        assert_eq!(file("docs/a.csv").path(), "docs/a.csv");
    }

    #[test]
    fn a_pack_keeps_its_paths_inside_the_package() {
        let kit = Origin::Pack {
            name: "@acme/kit".into(),
            version: Version::new(1, 4, 2),
            path: "kit.comp.md".into(),
        };
        assert_eq!(kit.to_string(), "@acme/kit@1.4.2/kit.comp.md");
        assert_eq!(
            kit.join("./parts/chip.comp.md")
                .map(|origin| origin.to_string()),
            Some("@acme/kit@1.4.2/parts/chip.comp.md".into())
        );
        assert_eq!(kit.join("../x.comp.md"), None);
        assert_eq!(file("a.comp.md").to_string(), "a.comp.md");
    }

    #[test]
    fn core_is_a_pack() {
        let core = Origin::core();
        assert!(core.is_core() && !kit_file().is_core());
        assert_eq!(core.to_string(), "@compmd/core@1.0.0/index.comp.md");
        assert_eq!(core.path(), "index.comp.md");
    }

    fn kit_file() -> Origin {
        Origin::Pack {
            name: "@acme/kit".into(),
            version: Version::new(1, 0, 0),
            path: "index.comp.md".into(),
        }
    }
}
