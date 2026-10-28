//! An import's `from` (spec §6.1, §6.4): a document or a component file by a relative path, or a pack from the
//! registry with a version or range.

use super::language::Language;
use semver::{Version, VersionReq};
use thiserror::Error;

/// What an import's `from` names.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Source<'a> {
    /// A CompMD document, `./ui.comp.md`.
    Document(&'a str),
    /// A component file, `./HabitHeatmap.jsx`, which exports one component named after its stem.
    File {
        path: &'a str,
        language: Language,
        stem: &'a str,
    },
    /// A pack from the registry, `@acme/kit@^1.4.2`.
    Pack(PackSpec),
}

/// A pack and the releases an import takes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PackSpec {
    pub(crate) name: String,
    /// The releases it takes: every release without a version.
    pub(crate) range: VersionReq,
    /// Whether it names one exact version, as `integrity` requires.
    pub(crate) exact: bool,
}

/// Why a `from` names nothing that can be loaded (F026).
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub(crate) enum Unloadable {
    #[error(
        "`{0}` isn't a relative path or a scoped pack (`@scope/name`); pack names are always scoped"
    )]
    Unscoped(String),
    #[error("`@compmd/` is reserved: processors build `@compmd/core` in, and it is never imported")]
    Reserved,
    #[error("`{0}` isn't a version or a version range")]
    Version(String),
    #[error(
        "`{0}` is neither a `.comp.md` document nor a component file (`.js`, `.ts`, `.jsx`, `.tsx`, `.svelte`)"
    )]
    Extension(String),
}

impl<'a> Source<'a> {
    /// Reads `from`.
    ///
    /// # Errors
    ///
    /// Why it names nothing that can be loaded.
    pub(crate) fn parse(from: &'a str) -> Result<Self, Unloadable> {
        if from.starts_with("./") || from.starts_with("../") {
            return relative(from);
        }
        let Some(scoped) = from.strip_prefix('@') else {
            return Err(Unloadable::Unscoped(from.to_string()));
        };
        let (name, version) = match scoped.split_once('@') {
            Some((name, version)) => (name, Some(version)),
            None => (scoped, None),
        };
        let parts: Vec<&str> = name.split('/').collect();
        if parts.len() != 2 || parts.iter().any(|part| part.is_empty()) {
            return Err(Unloadable::Unscoped(from.to_string()));
        }
        if parts[0] == "compmd" {
            return Err(Unloadable::Reserved);
        }
        let (range, exact) = match version {
            None => (VersionReq::STAR, false),
            Some(text) => range(text).ok_or_else(|| Unloadable::Version(text.to_string()))?,
        };
        Ok(Self::Pack(PackSpec {
            name: format!("@{name}"),
            range,
            exact,
        }))
    }
}

fn relative(path: &str) -> Result<Source<'_>, Unloadable> {
    if path.ends_with(".comp.md") {
        return Ok(Source::Document(path));
    }
    let file = path.rsplit('/').next().unwrap_or(path);
    let (stem, extension) = file
        .rsplit_once('.')
        .ok_or_else(|| Unloadable::Extension(path.to_string()))?;
    let language =
        Language::from_name(extension).ok_or_else(|| Unloadable::Extension(path.to_string()))?;
    Ok(Source::File {
        path,
        language,
        stem,
    })
}

/// A version range, and whether it is one exact version: `1.4.2` is exactly that release, unlike Cargo's
/// requirements, where it would be `^1.4.2`.
fn range(text: &str) -> Option<(VersionReq, bool)> {
    if let Ok(version) = Version::parse(text) {
        let exact = VersionReq::parse(&format!("={version}")).ok()?;
        return Some((exact, true));
    }
    VersionReq::parse(text).ok().map(|range| (range, false))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(from: &str) -> Option<PackSpec> {
        match Source::parse(from) {
            Ok(Source::Pack(spec)) => Some(spec),
            _ => None,
        }
    }

    #[test]
    fn relative_paths_name_documents_and_component_files() {
        assert_eq!(
            Source::parse("./ui.comp.md"),
            Ok(Source::Document("./ui.comp.md"))
        );
        assert_eq!(
            Source::parse("../charts/HabitHeatmap.jsx"),
            Ok(Source::File {
                path: "../charts/HabitHeatmap.jsx",
                language: Language::Jsx,
                stem: "HabitHeatmap",
            })
        );
        assert!(matches!(
            Source::parse("./notes.md"),
            Err(Unloadable::Extension(_))
        ));
        assert!(matches!(
            Source::parse("./README"),
            Err(Unloadable::Extension(_))
        ));
    }

    #[test]
    fn packs_take_a_version_or_a_range() {
        let any = pack("@acme/kit").expect("a pack");
        assert_eq!((any.name.as_str(), any.exact), ("@acme/kit", false));
        assert!(any.range.matches(&Version::new(3, 0, 0)));
        let exact = pack("@acme/kit@1.4.2").expect("a pack");
        assert!(exact.exact);
        assert!(!exact.range.matches(&Version::new(1, 4, 3)));
        let major = pack("@acme/kit@1").expect("a pack");
        assert!(major.range.matches(&Version::new(1, 9, 0)) && !major.exact);
        let caret = pack("@acme/kit@^1.4.2").expect("a pack");
        assert!(!caret.range.matches(&Version::new(1, 4, 1)));
        assert_eq!(pack("./ui.comp.md"), None);
    }

    #[test]
    fn unscoped_reserved_and_broken_sources_are_unloadable() {
        assert!(matches!(
            Source::parse("chartkit"),
            Err(Unloadable::Unscoped(_))
        ));
        assert!(matches!(
            Source::parse("@kit"),
            Err(Unloadable::Unscoped(_))
        ));
        assert!(matches!(
            Source::parse("@a/b/c"),
            Err(Unloadable::Unscoped(_))
        ));
        assert_eq!(Source::parse("@compmd/extras"), Err(Unloadable::Reserved));
        assert!(matches!(
            Source::parse("@a/b@soon"),
            Err(Unloadable::Version(_))
        ));
        assert!(Unloadable::Reserved.to_string().contains("reserved"));
    }
}
