//! A release: a pack at one version, `@kit/mermaid@1.2.0`.

use std::fmt;
use std::str::FromStr;

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::invalid::Invalid;
use crate::name::PackName;

/// A pack at one version. In JSON it is `{"pack": "@kit/mermaid", "version": "1.2.0"}`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ReleaseId {
    /// The pack.
    pub pack: PackName,
    /// Its version.
    pub version: Version,
}

impl ReleaseId {
    /// `pack` at `version`.
    #[must_use]
    pub const fn new(pack: PackName, version: Version) -> Self {
        Self { pack, version }
    }
}

/// `@kit/mermaid@1.2.0`, as diagnostics and a host's folders name it.
impl fmt::Display for ReleaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.pack, self.version)
    }
}

impl FromStr for ReleaseId {
    type Err = Invalid;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || Invalid::Release(text.to_owned());
        let (pack, version) = text.rsplit_once('@').ok_or_else(invalid)?;
        Ok(Self {
            pack: PackName::new(pack).map_err(|_| invalid())?,
            version: version.parse().map_err(|_| invalid())?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_release_is_a_pack_and_a_version() {
        let release: ReleaseId = "@kit/mermaid@1.2.0".parse().expect("a release");
        assert_eq!(
            release,
            ReleaseId::new(
                "@kit/mermaid".parse().expect("a name"),
                Version::new(1, 2, 0)
            )
        );
        assert_eq!(release.to_string(), "@kit/mermaid@1.2.0");
        assert_eq!(
            serde_json::to_value(&release).expect("JSON"),
            serde_json::json!({"pack": "@kit/mermaid", "version": "1.2.0"})
        );
    }

    #[test]
    fn what_isnt_a_release_is_refused() {
        for text in [
            "@kit/mermaid",
            "kit/mermaid@1.2.0",
            "@kit/mermaid@one",
            "mermaid",
        ] {
            assert_eq!(
                text.parse::<ReleaseId>(),
                Err(Invalid::Release(text.to_owned())),
                "{text}"
            );
        }
        assert!(
            Invalid::Release("x".to_owned())
                .to_string()
                .contains("isn't a release")
        );
    }
}
