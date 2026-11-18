//! An integrity (W3C Subresource Integrity): a hash's algorithm and its base64 digest, `sha512-…`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::invalid::Invalid;

/// A hash's algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Algorithm {
    /// SHA-256.
    Sha256,
    /// SHA-384, which a frame checks a built module against.
    Sha384,
    /// SHA-512, which npm publishes for a release's tarball (spec §6.1).
    Sha512,
}

impl Algorithm {
    /// Its name in an integrity: `sha256`, `sha384`, `sha512`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Sha384 => "sha384",
            Self::Sha512 => "sha512",
        }
    }
}

/// An integrity, `sha512-9c1fQe…`. In JSON it is that string.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Integrity {
    algorithm: Algorithm,
    digest: String,
}

impl Integrity {
    /// The integrity `text`, when it is one: `sha256-`, `sha384-` or `sha512-` and a digest. The digest is the hash in
    /// base64, but is taken as written, any visible characters: it is compared, never decoded, and the spec's own
    /// examples shorten it (`sha512-9c1fQe…`).
    ///
    /// # Errors
    ///
    /// [`Invalid::Integrity`] when it isn't one.
    pub fn new(text: &str) -> Result<Self, Invalid> {
        let invalid = || Invalid::Integrity(text.to_owned());
        let (name, digest) = text.split_once('-').ok_or_else(invalid)?;
        let algorithm = [Algorithm::Sha256, Algorithm::Sha384, Algorithm::Sha512]
            .into_iter()
            .find(|algorithm| algorithm.name() == name)
            .ok_or_else(invalid)?;
        let visible = !digest.is_empty() && !digest.chars().any(char::is_whitespace);
        if visible {
            Ok(Self {
                algorithm,
                digest: digest.to_owned(),
            })
        } else {
            Err(invalid())
        }
    }

    /// Its algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    /// Its digest, in base64.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
}

impl fmt::Display for Integrity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.algorithm.name(), self.digest)
    }
}

impl FromStr for Integrity {
    type Err = Invalid;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::new(text)
    }
}

impl TryFrom<String> for Integrity {
    type Error = Invalid;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::new(&text)
    }
}

impl From<Integrity> for String {
    fn from(integrity: Integrity) -> Self {
        integrity.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_integrity_is_an_algorithm_and_a_digest() {
        for (text, algorithm) in [
            ("sha256-abc=", Algorithm::Sha256),
            ("sha384-a+b/c", Algorithm::Sha384),
            ("sha512-9c1fQe0=", Algorithm::Sha512),
        ] {
            let integrity: Integrity = text.parse().expect(text);
            assert_eq!(integrity.algorithm(), algorithm);
            assert_eq!(integrity.to_string(), text);
        }
        assert_eq!(Integrity::new("sha512-xyz").expect("one").digest(), "xyz");
        assert_eq!(
            Integrity::new("sha512-9c1fQe…")
                .expect("as the spec shortens it")
                .digest(),
            "9c1fQe…"
        );
    }

    #[test]
    fn what_isnt_an_integrity_is_refused() {
        for text in [
            "",
            "sha512",
            "sha512-",
            "md5-abc",
            "sha512-a b",
            "sha512-a\tb",
        ] {
            assert_eq!(
                Integrity::new(text),
                Err(Invalid::Integrity(text.to_owned())),
                "{text}"
            );
        }
        assert!(
            Invalid::Integrity("x".to_owned())
                .to_string()
                .contains("base64")
        );
    }

    #[test]
    fn an_integrity_is_a_string_in_json() {
        let integrity: Integrity = serde_json::from_str(r#""sha512-abc""#).expect("one");
        assert_eq!(
            serde_json::to_string(&integrity).expect("JSON"),
            r#""sha512-abc""#
        );
        assert!(serde_json::from_str::<Integrity>(r#""md5-abc""#).is_err());
        assert_eq!(String::from(integrity), "sha512-abc");
    }
}
