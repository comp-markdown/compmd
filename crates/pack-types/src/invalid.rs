//! Why a text isn't one of the names this crate knows.

/// What is wrong with a text read as a pack's name, a release, a path, a place or an integrity.
#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq)]
pub enum Invalid {
    /// It isn't a pack's name.
    #[error(
        "`{0}` isn't a pack's name: `@scope/name`, each part lowercase letters, digits, `-`, `.`, `_` and `~`, not starting with `.` or `_`"
    )]
    Name(String),
    /// It isn't a release: a pack's name, `@` and a version.
    #[error("`{0}` isn't a release: a pack's name, `@` and a version (`@kit/mermaid@1.2.0`)")]
    Release(String),
    /// It isn't a file's path in a pack.
    #[error(
        "`{0}` isn't a file's path in a pack: `/`-separated names, none of them empty, `.` or `..`, with no `\\` or `:`"
    )]
    Path(String),
    /// It isn't an integrity.
    #[error("`{0}` isn't an integrity: `sha256-`, `sha384-` or `sha512-` and a digest, in base64")]
    Integrity(String),
}
