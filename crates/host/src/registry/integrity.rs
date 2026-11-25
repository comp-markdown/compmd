//! A tarball's integrity, as npm publishes it (`dist.integrity`): `sha512-` and the SHA-512 of its bytes in base64
//! ([resolving registry packs], decision 5).
//!
//! [resolving registry packs]: https://github.com/comp-markdown/compmd/blob/main/docs/resolving-registry-packs.md

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest, Sha512};

/// The `sha512-…` integrity of `bytes`.
pub(crate) fn integrity_of(bytes: &[u8]) -> String {
    format!("sha512-{}", STANDARD.encode(Sha512::digest(bytes)))
}

/// Whether `integrity` is one the rules check: a SHA-512, the only algorithm npm publishes for current packages.
pub(crate) fn is_checked(integrity: Option<&str>) -> bool {
    integrity.is_some_and(|integrity| integrity.starts_with("sha512-"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_integrity_is_the_sha512_in_base64() {
        assert_eq!(
            integrity_of(b"abc"),
            "sha512-3a81oZNherrMQXNJriBBMRLm+k6JqX6iCp7u5ktV05ohkpkqJ0/BqDa6PCOj/uu9RU1EI2Q86A4qmslPpUyknw=="
        );
        assert!(is_checked(Some("sha512-x")));
        assert!(!is_checked(Some("sha1-x")));
        assert!(!is_checked(None));
    }
}
