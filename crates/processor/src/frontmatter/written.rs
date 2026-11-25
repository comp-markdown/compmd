//! A document's frontmatter as written (spec §2.1), for a tool that rewrites it: its syntax, its value with each
//! mapping's keys in the order they are written, and the line each key is on.

use std::collections::HashMap;

use serde_json::Value;

use crate::parse::frontmatter::{find, parse};

pub use crate::parse::frontmatter::Format as Syntax;

/// A frontmatter as written.
///
/// ```
/// use compmd::frontmatter::{Syntax, Written};
///
/// let written = Written::of("---\ntitle: Notes\nmd:\n  lang: de\n---\n\n# Notes\n").expect("a frontmatter");
/// assert_eq!(written.syntax, Syntax::Yaml);
/// assert_eq!(written.value["md"]["lang"], "de");
/// assert_eq!(written.key_lines["/md/lang"], 4);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Written {
    /// What it is written in.
    pub syntax: Syntax,
    /// Its value, each mapping's keys in written order; data imports' `from` as written.
    pub value: Value,
    /// The line each key is on, from 1 in the document, by JSON pointer (`/md/import/0/from`), whose steps are the
    /// keys and indexes as they are.
    pub key_lines: HashMap<String, usize>,
}

impl Written {
    /// The frontmatter at the start of the document `text`; `None` when it has none, or one that doesn't read (F033).
    #[must_use]
    pub fn of(text: &str) -> Option<Self> {
        let found = find(text)?;
        let document = parse(found).ok()?;
        // The frontmatter's text starts on the document's second line.
        let key_lines = document
            .keys
            .into_iter()
            .map(|(pointer, mark)| (pointer, mark.line + 1))
            .collect();
        Some(Self {
            syntax: found.format,
            value: document.value,
            key_lines,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn toml_keys_have_their_lines_and_their_order() {
        let written =
            Written::of("+++\nzebra = 1\napple = 2\n[md]\nlang = \"de\"\n+++\n").expect("TOML");
        assert_eq!(written.syntax, Syntax::Toml);
        assert_eq!(
            written.value,
            json!({ "zebra": 1, "apple": 2, "md": { "lang": "de" } })
        );
        let keys: Vec<&String> = written
            .value
            .as_object()
            .expect("a mapping")
            .keys()
            .collect();
        assert_eq!(keys, ["zebra", "apple", "md"]);
        assert_eq!(written.key_lines["/apple"], 3);
        assert_eq!(written.key_lines["/md/lang"], 5);
    }

    #[test]
    fn none_without_one_or_with_one_that_doesnt_read() {
        assert_eq!(Written::of("# Notes\n"), None);
        assert_eq!(Written::of("---\na: [\n---\n"), None);
    }
}
