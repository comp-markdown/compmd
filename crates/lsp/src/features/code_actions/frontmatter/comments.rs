//! A frontmatter's comments on lines of their own, each kept with the key that follows it, so that a rewrite writes
//! it above that key again, in YAML or TOML (both comment with `#`). A comment after a key on its line isn't kept.

use std::collections::HashMap;

use compmd::frontmatter::Written;

/// The comments of a frontmatter.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Comments {
    /// The comments above each key, by its pointer, each without its `#` and the space after it.
    pub(crate) above: HashMap<String, Vec<String>>,
    /// Those after the last key.
    pub(crate) end: Vec<String>,
}

impl Comments {
    /// The comments of the frontmatter `written`, whose text is the document's lines `first..=last` (from 1) of `text`.
    pub(crate) fn of(text: &str, written: &Written, first: usize, last: usize) -> Self {
        let mut keys: Vec<(usize, &String)> = written
            .key_lines
            .iter()
            .map(|(pointer, line)| (*line, pointer))
            .collect();
        // On a line with several keys (a flow mapping), the comment is the outermost's.
        keys.sort_by_key(|(line, pointer)| (*line, pointer.len()));
        let mut comments = Self::default();
        for (index, line) in text.split('\n').enumerate().take(last).skip(first - 1) {
            let Some(comment) = line.trim().strip_prefix('#') else {
                continue;
            };
            let number = index + 1;
            if in_text_above(written, &keys, number, line.trim()) {
                continue;
            }
            let comment = comment.strip_prefix(' ').unwrap_or(comment).to_owned();
            match keys.iter().find(|(line, _)| *line > number) {
                Some((_, pointer)) => comments
                    .above
                    .entry((*pointer).clone())
                    .or_default()
                    .push(comment),
                None => comments.end.push(comment),
            }
        }
        comments
    }

    /// The comment lines above the key at `pointer`, written with `indent`.
    pub(crate) fn write_above(&self, pointer: &str, indent: &str, out: &mut String) {
        for comment in self.above.get(pointer).into_iter().flatten() {
            write_comment(comment, indent, out);
        }
    }

    /// The comment lines after the last key.
    pub(crate) fn write_end(&self, out: &mut String) {
        for comment in &self.end {
            write_comment(comment, "", out);
        }
    }
}

/// Whether `line`, the document's line `number`, is a line of the text of several lines that the key above it holds
/// (a YAML block scalar, a TOML multi-line string), not a comment.
fn in_text_above(written: &Written, keys: &[(usize, &String)], number: usize, line: &str) -> bool {
    keys.iter()
        .rev()
        .find(|(key_line, _)| *key_line < number)
        .and_then(|(_, pointer)| written.value.pointer(pointer))
        .and_then(serde_json::Value::as_str)
        .is_some_and(|text| text.contains('\n') && text.lines().any(|held| held.trim() == line))
}

/// `comment` on a line of its own, with `indent`.
fn write_comment(comment: &str, indent: &str, out: &mut String) {
    out.push_str(indent);
    out.push('#');
    if !comment.is_empty() {
        out.push(' ');
        out.push_str(comment);
    }
    out.push('\n');
}
