//! The frames' build's hash, which names its folder, `frames/<hash>/`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::manifest_error::ManifestError;

/// A frames' build's hash: the first 16 hex digits, lowercase, of a SHA-256 over each file's path and bytes, so it
/// can't name another folder.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FramesHash(String);

impl FramesHash {
    /// The hash `text`, when it is one.
    ///
    /// # Errors
    ///
    /// [`ManifestError::Hash`] when it isn't 16 lowercase hex digits.
    pub fn new(text: impl Into<String>) -> Result<Self, ManifestError> {
        let text = text.into();
        let hex = text.len() == 16
            && text
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
        if hex {
            Ok(Self(text))
        } else {
            Err(ManifestError::Hash(text))
        }
    }

    /// The hash as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for FramesHash {
    type Err = ManifestError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::new(text)
    }
}

impl TryFrom<String> for FramesHash {
    type Error = ManifestError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::new(text)
    }
}

impl From<FramesHash> for String {
    fn from(hash: FramesHash) -> Self {
        hash.0
    }
}

impl fmt::Display for FramesHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hash_is_16_lowercase_hex_digits() {
        let hash: FramesHash = "1df336e75685de8c".parse().expect("a hash");
        assert_eq!(hash.as_str(), "1df336e75685de8c");
        assert_eq!(hash.to_string(), "1df336e75685de8c");
        assert_eq!(String::from(hash.clone()), "1df336e75685de8c");
        assert_eq!(
            FramesHash::try_from("1df336e75685de8c".to_owned()),
            Ok(hash)
        );
        for text in [
            "../secret",
            "1DF336E75685DE8C",
            "1df336e7",
            "1df336e75685de8g",
        ] {
            assert_eq!(
                FramesHash::new(text),
                Err(ManifestError::Hash(text.to_owned())),
                "{text}"
            );
        }
    }
}
