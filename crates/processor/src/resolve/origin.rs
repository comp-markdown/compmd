//! Where a document comes from: a file of the root folder, or a file of a pack. The origin names the document in
//! diagnostics (spec §10.4) and decides where its relative paths lead (spec §8.6).

use super::path_refusal::PathRefusal;
use compmd_pack::{FilePath, PackName, Place, ReleaseId};
use semver::Version;
use std::fmt;

/// The pack the processor builds in (spec §9).
pub(crate) const CORE: &str = "@compmd/core";

/// Where a document comes from.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Origin {
    /// A file of the document's root folder, by its path from it.
    File(FilePath),
    /// A file of a pack, by its place: one the host's `Packs` serve, imported or ambient, or `@compmd/core`, which
    /// the processor builds in.
    Pack(Place),
    /// The processed document, when the path the host gave it isn't a file's path in its root (`./a.comp.md`,
    /// `C:\docs\a.comp.md`, an empty path): it is processed, but its relative paths name nothing.
    Unplaced(String),
}

impl Origin {
    /// The processed document, at the `path` the host gave it under its root folder.
    pub(crate) fn main(path: &str) -> Self {
        FilePath::new(path).map_or_else(|_| Self::Unplaced(path.to_string()), Self::File)
    }

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

    /// The file `relative` (`./card.comp.md`, `../ui/x.jsx`) names, seen from this document, in its root: the
    /// document's root folder, or the pack's.
    ///
    /// # Errors
    ///
    /// Why it names no file: it leaves the root or names a folder, a name isn't a file's name, or this document has no
    /// place in its root.
    pub(crate) fn join(&self, relative: &str) -> Result<Self, PathRefusal> {
        let joined = match self {
            Self::File(path) => path.join(relative).map(Self::File),
            Self::Pack(place) => place.join(relative).map(Self::Pack),
            Self::Unplaced(document) => {
                return Err(PathRefusal::Unplaced {
                    relative: relative.to_string(),
                    document: document.clone(),
                });
            }
        };
        joined.ok_or_else(|| {
            let relative = relative.to_string();
            if relative.contains(['\\', ':', '\0']) {
                PathRefusal::NotAPath { relative }
            } else {
                PathRefusal::Leaves { relative }
            }
        })
    }

    /// The file's path in its root folder or its pack.
    pub(crate) fn path(&self) -> &str {
        match self {
            Self::File(path) => path.as_str(),
            Self::Pack(place) => place.path.as_str(),
            Self::Unplaced(path) => path,
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
            Self::File(path) => path.fmt(f),
            Self::Pack(place) => place.fmt(f),
            Self::Unplaced(path) => f.write_str(path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str) -> Origin {
        Origin::File(FilePath::new(path).expect("a path"))
    }

    fn pack(place: &str) -> Origin {
        Origin::Pack(place.parse().expect("a place"))
    }

    fn leaves(relative: &str) -> Result<Origin, PathRefusal> {
        Err(PathRefusal::Leaves {
            relative: relative.into(),
        })
    }

    #[test]
    fn relative_paths_resolve_against_the_documents_folder() {
        assert_eq!(
            file("input.comp.md").join("./ui.comp.md"),
            Ok(file("ui.comp.md"))
        );
        assert_eq!(
            file("docs/a.comp.md").join("../ui/b.jsx"),
            Ok(file("ui/b.jsx"))
        );
        assert_eq!(
            file("docs/a.comp.md").join("./x/../y.comp.md"),
            Ok(file("docs/y.comp.md"))
        );
        assert_eq!(
            file("a.comp.md").join("../b.comp.md"),
            leaves("../b.comp.md")
        );
        assert_eq!(file("a.comp.md").join("./"), leaves("./"), "a folder");
        assert_eq!(file("docs/a.csv").path(), "docs/a.csv");
    }

    #[test]
    fn a_name_that_isn_t_a_file_s_name_is_refused() {
        for relative in ["./..\\..\\secrets.csv", "./c:/x.csv", "./a\0.csv"] {
            assert_eq!(
                file("docs/a.comp.md").join(relative),
                Err(PathRefusal::NotAPath {
                    relative: relative.into()
                }),
                "{relative:?}"
            );
        }
    }

    #[test]
    fn a_document_the_host_gave_no_path_in_its_root_resolves_nothing() {
        for path in ["./a.comp.md", "/docs/a.comp.md", "C:\\docs\\a.comp.md", ""] {
            let main = Origin::main(path);
            assert_eq!(main, Origin::Unplaced(path.into()));
            assert_eq!((main.path(), main.to_string().as_str()), (path, path));
            assert_eq!(
                main.join("./b.csv"),
                Err(PathRefusal::Unplaced {
                    relative: "./b.csv".into(),
                    document: path.into()
                })
            );
        }
        assert_eq!(Origin::main("docs/a.comp.md"), file("docs/a.comp.md"));
    }

    #[test]
    fn a_pack_keeps_its_paths_inside_the_package() {
        let kit = pack("@acme/kit@1.4.2/kit.comp.md");
        assert_eq!(kit.to_string(), "@acme/kit@1.4.2/kit.comp.md");
        assert_eq!(
            kit.join("./parts/chip.comp.md")
                .map(|origin| origin.to_string()),
            Ok("@acme/kit@1.4.2/parts/chip.comp.md".into())
        );
        assert_eq!(kit.join("../x.comp.md"), leaves("../x.comp.md"));
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
