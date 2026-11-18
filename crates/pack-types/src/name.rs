//! A pack's name (spec §6.4): always scoped, `@scope/name`, an npm package's.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::invalid::Invalid;

/// The scope the spec reserves: `@compmd/core` is built into processors, and `@compmd/*` is never fetched.
const RESERVED: &str = "compmd";

/// The longest name npm takes.
const LONGEST: usize = 214;

/// A pack's name, `@scope/name`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PackName(String);

impl PackName {
    /// The name `text`, when it is one: `@`, a scope, `/` and a name, each part lowercase letters, digits, `-`, `.`,
    /// `_` and `~`, not starting with `.` or `_`, as npm names scoped packages; at most 214 characters.
    ///
    /// # Errors
    ///
    /// [`Invalid::Name`] when it isn't one.
    pub fn new(text: impl Into<String>) -> Result<Self, Invalid> {
        let text = text.into();
        let plain = |part: &str| {
            !part.is_empty()
                && !part.starts_with(['.', '_'])
                && part.bytes().all(|byte| {
                    byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'-' | b'.' | b'_' | b'~')
                })
        };
        let scoped = text.len() <= LONGEST
            && text
                .strip_prefix('@')
                .and_then(|rest| rest.split_once('/'))
                .is_some_and(|(scope, name)| plain(scope) && plain(name));
        if scoped {
            Ok(Self(text))
        } else {
            Err(Invalid::Name(text))
        }
    }

    /// The name as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Its scope, without `@`: `kit` for `@kit/mermaid`.
    #[must_use]
    pub fn scope(&self) -> &str {
        self.parts().0
    }

    /// Its name in the scope: `mermaid` for `@kit/mermaid`.
    #[must_use]
    pub fn name(&self) -> &str {
        self.parts().1
    }

    /// Whether it is in the scope the spec reserves, `@compmd/`.
    #[must_use]
    pub fn is_reserved(&self) -> bool {
        self.scope() == RESERVED
    }

    fn parts(&self) -> (&str, &str) {
        self.0[1..].split_once('/').unwrap_or_default()
    }
}

impl FromStr for PackName {
    type Err = Invalid;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::new(text)
    }
}

impl TryFrom<String> for PackName {
    type Error = Invalid;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::new(text)
    }
}

impl From<PackName> for String {
    fn from(name: PackName) -> Self {
        name.0
    }
}

impl fmt::Display for PackName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl PartialEq<str> for PackName {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for PackName {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_scoped_and_plain() {
        for text in ["@kit/mermaid", "@comp-markdown/mermaid", "@a1/b.c_d~e"] {
            let name = PackName::new(text).expect(text);
            assert_eq!(name.as_str(), text);
            assert_eq!(name.to_string(), text);
            assert_eq!(name, text);
            assert!(name == *text, "against a `str` too");
        }
        let name: PackName = "@kit/mermaid".parse().expect("a name");
        assert_eq!((name.scope(), name.name()), ("kit", "mermaid"));
        assert!(!name.is_reserved());
        assert!(PackName::new("@compmd/core").expect("a name").is_reserved());
    }

    #[test]
    fn what_isnt_a_name_is_refused_saying_why() {
        for text in [
            "mermaid",
            "@kit",
            "@kit/",
            "@/mermaid",
            "@kit/../x",
            "@kit/.x",
            "@kit/_x",
            "@KIT/x",
            "@kit/a/b",
            "@kit/a b",
        ] {
            assert_eq!(
                PackName::new(text),
                Err(Invalid::Name(text.to_owned())),
                "{text}"
            );
        }
        let long = format!("@kit/{}", "a".repeat(LONGEST));
        assert!(PackName::new(long).is_err());
        assert!(
            Invalid::Name("x".to_owned())
                .to_string()
                .contains("`@scope/name`")
        );
    }

    #[test]
    fn a_name_is_a_string_in_json() {
        let name: PackName = serde_json::from_str(r#""@kit/mermaid""#).expect("a name");
        assert_eq!(
            serde_json::to_string(&name).expect("JSON"),
            r#""@kit/mermaid""#
        );
        assert!(serde_json::from_str::<PackName>(r#""mermaid""#).is_err());
        assert_eq!(String::from(name), "@kit/mermaid");
        assert_eq!(
            PackName::try_from("@kit/x".to_owned()),
            PackName::new("@kit/x")
        );
    }
}
