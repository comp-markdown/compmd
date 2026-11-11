//! A file's place in a pack (spec §6.4): the pack, its version and the file's path, as the store and the URL
//! `packs/<name>@<version>/<path>` name it.

use semver::Version;

/// A file of a pack, by its place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    /// The pack's name, `@scope/name`.
    pub pack: String,
    /// The pack's version.
    pub version: Version,
    /// The file's path in the pack, its parts joined by `/`.
    pub path: String,
}

impl Place {
    /// The place `parts` name, the parts of a path after `packs/`: `@scope`, `name@version`, then the file's path. None
    /// when they don't name one: no version, a version that isn't one, or no file.
    #[must_use]
    pub fn from_parts(parts: &[&str]) -> Option<Self> {
        let (scope, rest) = parts.split_first()?;
        let (named, path) = rest.split_first()?;
        let (name, version) = named.rsplit_once('@')?;
        let pack = format!("{scope}/{name}");
        if !is_pack_name(&pack) || path.is_empty() {
            return None;
        }
        Some(Self {
            pack,
            version: version.parse().ok()?,
            path: path.join("/"),
        })
    }

    /// The folder of its release, `<name>@<version>`, as the bundle and a store lay packs out.
    #[must_use]
    pub fn release(&self) -> String {
        release_folder(&self.pack, &self.version)
    }
}

/// The folder of a pack's release, `<name>@<version>`.
pub(crate) fn release_folder(pack: &str, version: &Version) -> String {
    format!("{pack}@{version}")
}

/// Whether `name` is a pack's name: `@scope/name` (spec §6.4: names are always scoped), each part lowercase letters,
/// digits, `-`, `.` and `_`, not starting with `.` or `_`, so it can't name another folder.
pub(crate) fn is_pack_name(name: &str) -> bool {
    let part = |part: &str| {
        !part.is_empty()
            && !part.starts_with(['.', '_'])
            && part.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'-' | b'.' | b'_')
            })
    };
    name.strip_prefix('@')
        .and_then(|rest| rest.split_once('/'))
        .is_some_and(|(scope, name)| part(scope) && part(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_place_is_its_pack_version_and_path() {
        let place =
            Place::from_parts(&["@kit", "mermaid@1.2.0", "dist", "mermaid.js"]).expect("a place");
        assert_eq!(place.pack, "@kit/mermaid");
        assert_eq!(place.version, Version::new(1, 2, 0));
        assert_eq!(place.path, "dist/mermaid.js");
        assert_eq!(place.release(), "@kit/mermaid@1.2.0");
    }

    #[test]
    fn what_names_no_place_is_none() {
        for parts in [
            &[][..],
            &["@kit"][..],
            &["@kit", "mermaid@1.2.0"][..],
            &["@kit", "mermaid", "a.js"][..],
            &["@kit", "mermaid@one", "a.js"][..],
            &["kit", "mermaid@1.2.0", "a.js"][..],
            &["@Kit", "mermaid@1.2.0", "a.js"][..],
        ] {
            assert_eq!(Place::from_parts(parts), None, "{parts:?}");
        }
    }

    #[test]
    fn a_packs_name_is_scoped_and_plain() {
        for name in ["@kit/mermaid", "@comp-markdown/mermaid", "@a1/b.c_d"] {
            assert!(is_pack_name(name), "{name}");
        }
        for name in [
            "mermaid",
            "@kit",
            "@kit/",
            "@/mermaid",
            "@kit/../x",
            "@kit/.x",
            "@kit/_x",
            "@KIT/x",
            "@kit/a/b",
        ] {
            assert!(!is_pack_name(name), "{name}");
        }
    }
}
