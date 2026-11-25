//! The names a configuration writes: npm packages, scopes and packs.

use compmd_pack::PackName;

/// Whether `c` may be in an npm name after its first character: `a-z`, `0-9`, `.`, `_`, `~` or `-`.
const fn is_name_char(c: char) -> bool {
    matches!(c, 'a'..='z' | '0'..='9' | '.' | '_' | '~' | '-')
}

/// Whether `part` is an npm name's part: a lowercase letter or digit, then name characters.
fn is_name_part(part: &str) -> bool {
    let mut chars = part.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_lowercase() || first.is_ascii_digit())
        && chars.all(is_name_char)
}

/// Whether `name` is an npm package's name: `d3`, or scoped, `@acme/charts`.
pub(crate) fn is_package_name(name: &str) -> bool {
    match name.strip_prefix('@') {
        Some(scoped) => scoped
            .split_once('/')
            .is_some_and(|(scope, rest)| is_name_part(scope) && is_name_part(rest)),
        None => is_name_part(name),
    }
}

/// Whether `scope` is an npm scope, `@acme`.
pub(crate) fn is_scope(scope: &str) -> bool {
    scope.strip_prefix('@').is_some_and(|rest| {
        let mut chars = rest.chars();
        chars.next().is_some_and(|first| {
            first.is_ascii_lowercase() || first.is_ascii_digit() || matches!(first, '~' | '-')
        }) && chars.all(is_name_char)
    })
}

/// Whether `name` is a pack's name, as the processor reads one: a scoped npm name.
pub(crate) fn is_pack_name(name: &str) -> bool {
    name.parse::<PackName>().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_names_are_npms() {
        for name in ["d3", "react-dom", "@acme/charts", "lodash.merge", "0x"] {
            assert!(is_package_name(name), "{name}");
        }
        for name in [
            "",
            "D3",
            "-a",
            "@acme",
            "@acme/",
            "@/a",
            "a/b",
            "@acme/charts/x",
            "é",
        ] {
            assert!(!is_package_name(name), "{name}");
        }
    }

    #[test]
    fn scopes_start_with_an_at() {
        for scope in ["@acme", "@a.b", "@-x", "@~x"] {
            assert!(is_scope(scope), "{scope}");
        }
        for scope in ["acme", "@", "@.x", "@Acme", "@acme/x"] {
            assert!(!is_scope(scope), "{scope}");
        }
    }

    #[test]
    fn pack_names_are_scoped() {
        assert!(is_pack_name("@acme/charts"));
        assert!(!is_pack_name("charts"));
    }
}
