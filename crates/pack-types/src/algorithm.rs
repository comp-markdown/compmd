//! A hash's algorithm, as an integrity names it.

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
    /// Every algorithm an integrity may name.
    pub const ALL: [Self; 3] = [Self::Sha256, Self::Sha384, Self::Sha512];

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
