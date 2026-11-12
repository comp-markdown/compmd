//! A file's place (spec §6.4): a release and the file's path in it, `@kit/mermaid@1.2.0/dist/mermaid.js`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::file_path::FilePath;
use crate::invalid::Invalid;
use crate::release::ReleaseId;

/// A file of a pack, by its place. In JSON it is `{"pack": …, "version": …, "path": …}`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Place {
    /// The release.
    #[serde(flatten)]
    pub release: ReleaseId,
    /// The file's path in it.
    pub path: FilePath,
}

impl Place {
    /// The file `path` of `release`.
    #[must_use]
    pub const fn new(release: ReleaseId, path: FilePath) -> Self {
        Self { release, path }
    }

    /// The place `parts` name: `@scope`, `name@version`, then the file's path, as a URL's path after `packs/` writes
    /// it. None when they don't name one.
    #[must_use]
    pub fn from_parts(parts: &[&str]) -> Option<Self> {
        let [scope, named, path @ ..] = parts else {
            return None;
        };
        Some(Self {
            release: format!("{scope}/{named}").parse().ok()?,
            path: FilePath::new(path.join("/")).ok()?,
        })
    }

    /// The file `relative` names, seen from this one, in the same release; `None` when it leaves the pack.
    #[must_use]
    pub fn join(&self, relative: &str) -> Option<Self> {
        Some(Self {
            release: self.release.clone(),
            path: self.path.join(relative)?,
        })
    }
}

/// `@kit/mermaid@1.2.0/dist/mermaid.js`, as diagnostics name a document of a pack (spec §10.4).
impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.release, self.path)
    }
}

impl FromStr for Place {
    type Err = Invalid;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = text.split('/').collect();
        Self::from_parts(&parts).ok_or_else(|| Invalid::Path(text.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(text: &str) -> Place {
        text.parse().expect(text)
    }

    #[test]
    fn a_place_is_a_release_and_a_path() {
        let module = place("@kit/mermaid@1.2.0/dist/mermaid.js");
        assert_eq!(module.release.to_string(), "@kit/mermaid@1.2.0");
        assert_eq!(module.path.as_str(), "dist/mermaid.js");
        assert_eq!(module.to_string(), "@kit/mermaid@1.2.0/dist/mermaid.js");
        assert_eq!(
            Place::new(module.release.clone(), module.path.clone()),
            module
        );
        assert_eq!(
            Place::from_parts(&["@kit", "mermaid@1.2.0", "dist", "mermaid.js"]),
            Some(module)
        );
    }

    #[test]
    fn a_place_is_flat_in_json() {
        let module = place("@kit/mermaid@1.2.0/dist/mermaid.js");
        let json = serde_json::json!({"pack": "@kit/mermaid", "version": "1.2.0", "path": "dist/mermaid.js"});
        assert_eq!(serde_json::to_value(&module).expect("JSON"), json);
        assert_eq!(
            serde_json::from_value::<Place>(json).expect("a place"),
            module
        );
    }

    #[test]
    fn what_names_no_place_is_refused() {
        for text in [
            "@kit/mermaid@1.2.0",
            "@kit/mermaid@1.2.0/",
            "@kit/mermaid/a.js",
            "kit/mermaid@1.2.0/a.js",
            "@kit/mermaid@1.2.0/../a.js",
            "@kit",
        ] {
            assert_eq!(
                text.parse::<Place>(),
                Err(Invalid::Path(text.to_owned())),
                "{text}"
            );
        }
        assert_eq!(Place::from_parts(&[]), None);
    }

    #[test]
    fn a_relative_file_stays_in_its_release() {
        let kit = place("@acme/kit@1.4.2/kit.comp.md");
        assert_eq!(
            kit.join("./parts/chip.comp.md"),
            Some(place("@acme/kit@1.4.2/parts/chip.comp.md"))
        );
        assert_eq!(kit.join("../x.comp.md"), None);
    }
}
