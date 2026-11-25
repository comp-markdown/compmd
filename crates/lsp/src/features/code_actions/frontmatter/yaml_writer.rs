//! A frontmatter's value written as YAML 1.2 core (spec §2.1): mappings in block style, a list of plain values in
//! flow style (`[card, callout]`) and other lists in block style, a string plain where it reads back as the same string
//! and double-quoted otherwise.

use std::fmt::Write as _;

use serde_json::{Map, Value};

use super::comments::Comments;

/// `mapping` as the text of a YAML frontmatter, with `comments` above their keys.
pub(crate) fn write(mapping: &Map<String, Value>, comments: &Comments) -> String {
    let mut out = String::new();
    write_mapping(mapping, "", 0, comments, &mut out);
    comments.write_end(&mut out);
    out
}

/// The entries of `mapping` at `pointer`, each on lines of its own with `indent` spaces.
fn write_mapping(
    mapping: &Map<String, Value>,
    pointer: &str,
    indent: usize,
    comments: &Comments,
    out: &mut String,
) {
    let pad = " ".repeat(indent);
    for (key, value) in mapping {
        let at = format!("{pointer}/{key}");
        comments.write_above(&at, &pad, out);
        out.push_str(&pad);
        write_entry(key, value, &at, indent, comments, out);
    }
}

/// `key: value`, whose line's indentation is written, the value's lines indented under it.
fn write_entry(
    key: &str,
    value: &Value,
    pointer: &str,
    indent: usize,
    comments: &Comments,
    out: &mut String,
) {
    out.push_str(&scalar_text(key));
    out.push(':');
    match value {
        Value::Object(entries) if !entries.is_empty() => {
            out.push('\n');
            write_mapping(entries, pointer, indent + 2, comments, out);
        }
        Value::Array(items) if !flows(items) => {
            out.push('\n');
            write_items(items, pointer, indent + 2, comments, out);
        }
        _ => {
            out.push(' ');
            out.push_str(&flow(value));
            out.push('\n');
        }
    }
}

/// The items of a list in block style, each a `- ` line with `indent` spaces, a mapping's first entry on it.
fn write_items(
    items: &[Value],
    pointer: &str,
    indent: usize,
    comments: &Comments,
    out: &mut String,
) {
    let pad = " ".repeat(indent);
    for (index, item) in items.iter().enumerate() {
        let at = format!("{pointer}/{index}");
        match item {
            Value::Object(entries) if !entries.is_empty() => {
                let mut entries = entries.iter();
                if let Some((key, value)) = entries.next() {
                    let first = format!("{at}/{key}");
                    comments.write_above(&first, &pad, out);
                    out.push_str(&pad);
                    out.push_str("- ");
                    write_entry(key, value, &first, indent + 2, comments, out);
                }
                let rest: Map<String, Value> = entries
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect();
                write_mapping(&rest, &at, indent + 2, comments, out);
            }
            Value::Array(inner) if !flows(inner) => {
                out.push_str(&pad);
                out.push_str("-\n");
                write_items(inner, &at, indent + 2, comments, out);
            }
            _ => {
                out.push_str(&pad);
                out.push_str("- ");
                out.push_str(&flow(item));
                out.push('\n');
            }
        }
    }
}

/// Whether a list is written in flow style: when none of its items is a mapping or a list with something in it.
fn flows(items: &[Value]) -> bool {
    items.iter().all(|item| match item {
        Value::Object(entries) => entries.is_empty(),
        Value::Array(inner) => inner.is_empty(),
        _ => true,
    })
}

/// `value` in flow style, on one line.
fn flow(value: &Value) -> String {
    match value {
        Value::Null => "null".into(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => scalar_text(text),
        Value::Array(items) => {
            let items: Vec<String> = items.iter().map(flow).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(entries) => {
            let entries: Vec<String> = entries
                .iter()
                .map(|(key, value)| format!("{}: {}", scalar_text(key), flow(value)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        }
    }
}

/// `text` as a YAML string: plain when it reads back as that string, in a block or in a flow, else double-quoted.
fn scalar_text(text: &str) -> String {
    if plain(text) {
        text.to_owned()
    } else {
        quoted(text)
    }
}

/// Whether `text` can be written plain: it starts with a letter, a digit, `.`, `/` or `_`, holds nothing YAML reads as
/// syntax nor anything YAML can't hold as it is, and doesn't read as a null, a boolean or a number in the core schema.
fn plain(text: &str) -> bool {
    let Some(first) = text.chars().next() else {
        return false;
    };
    let starts_well =
        (first.is_alphanumeric() || matches!(first, '.' | '/' | '_')) && !text.starts_with("..");
    let syntax_free = !text.contains([
        ':', '#', ',', '[', ']', '{', '}', '"', '\'', '\\', '\n', '\r', '\t',
    ]) && text.trim() == text
        && text.chars().all(printable);
    starts_well && syntax_free && !other_than_a_string(text)
}

/// `text` double-quoted, with what YAML can't hold as it is escaped.
fn quoted(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if !printable(c) => {
                let _ = write!(out, "\\u{:04X}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Whether YAML 1.2 holds `c` as it is (its printable characters, §5.1).
fn printable(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r' | ' '..='~' | '\u{85}' | '\u{A0}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}')
        || u32::from(c) >= 0x1_0000
}

/// Whether the core schema reads `text`, written plain, as something other than a string.
fn other_than_a_string(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    matches!(lower.as_str(), "null" | "true" | "false" | "~")
        || lower.starts_with("0x")
        || lower.starts_with("0o")
        || lower.starts_with(".inf")
        || lower.starts_with(".nan")
        || text.parse::<f64>().is_ok()
}

#[cfg(test)]
mod tests;
