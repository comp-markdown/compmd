//! Where a document comes from: a file of the root folder, or a file of a pack. The origin names the document in
//! diagnostics (spec §10.4) and decides where its relative paths lead (spec §8.6).

use compmd_pack::{FilePath, PackName, Place, ReleaseId};
use semver::Version;
use std::fmt;

/// The pack the processor builds in (spec §9).
pub(crate) const CORE: &str = "@compmd/core";

/// Where a document comes from.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Origin {
    /// A file of the document's root folder, by its `/`-separated path from it.
    File(String),
    /// A file of a pack, by its place: one the host's `Packs` serve, imported or ambient, or `@compmd/core`, which
    /// the processor builds in.
    Pack(Place),
}

impl Origin {
    /// `@compmd/core`'s document, at the version it exports.
    pub(crate) fn core() -> Self {
        Self::Pack(Place::new(
            ReleaseId::new(core_name(), Version::new(1, 0, 0)),
            FilePath::new("index.comp.md").unwrap_or_else(|_| unreachable!("a plain path")),
        ))
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
            Self::Pack(place) => place.join(relative).map(Self::Pack),
        }
    }

    /// The file's path in its root folder or its pack.
    pub(crate) fn path(&self) -> &str {
        match self {
            Self::File(path) => path,
            Self::Pack(place) => place.path.as_str(),
        }
    }
}

/// `@compmd/core`, the name.
pub(crate) fn core_name() -> PackName {
    PackName::new(CORE).unwrap_or_else(|_| unreachable!("a pack's name"))
}

/// How diagnostics name the document: `ui.comp.md`, `@myorg/ui@1.2.0/card.comp.md` (spec §10.4).
impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(path) => f.write_str(path),
            Self::Pack(place) => place.fmt(f),
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

    fn pack(place: &str) -> Origin {
        Origin::Pack(place.parse().expect("a place"))
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
        let kit = pack("@acme/kit@1.4.2/kit.comp.md");
        assert_eq!(kit.to_string(), "@acme/kit@1.4.2/kit.comp.md");
        assert_eq!(
            kit.join("./parts/chip.comp.md")
                .map(|origin| origin.to_string()),
            Some("@acme/kit@1.4.2/parts/chip.comp.md".into())
        );
        assert_eq!(kit.join("../x.comp.md"), None);
        assert_eq!(kit.path(), "kit.comp.md");
        assert_eq!(file("a.comp.md").to_string(), "a.comp.md");
    }

    #[test]
    fn core_is_a_pack() {
        let core = Origin::core();
        assert!(core.is_core() && !pack("@acme/kit@1.0.0/index.comp.md").is_core());
        assert_eq!(core.to_string(), "@compmd/core@1.0.0/index.comp.md");
        assert_eq!(core.path(), "index.comp.md");
        assert!(core_name().is_reserved());
    }
}
