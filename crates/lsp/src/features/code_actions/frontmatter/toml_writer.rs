//! A frontmatter's value written as TOML (spec §2.1): each table's plain values first, as TOML needs, then its tables
//! (`[md]`) and its lists of tables (`[[md.import]]`); a list that holds anything else, and a mapping inside one, on
//! one line (`{ name = "badge", as = "tag" }`).

use std::fmt::Write as _;

use serde_json::{Map, Value};

use super::comments::Comments;

/// A value TOML can't hold: an empty one (YAML's `null`), or an integer past 64 bits; at its pointer.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Unwritable(pub(crate) String);

/// `mapping` as the text of a TOML frontmatter, with `comments` above their keys.
pub(crate) fn write(
    mapping: &Map<String, Value>,
    comments: &Comments,
) -> Result<String, Unwritable> {
    let mut out = String::new();
    write_table(mapping, &[], "", comments, &mut out)?;
    comments.write_end(&mut out);
    Ok(out)
}

/// Whether `value` is written as a table of its own, under a header.
fn is_table(value: &Value) -> bool {
    matches!(value, Value::Object(entries) if !entries.is_empty())
}

/// Whether `value` is written as a list of tables, each under a `[[…]]` header.
fn is_table_list(value: &Value) -> bool {
    matches!(value, Value::Array(items) if !items.is_empty() && items.iter().all(is_table))
}

/// The table `mapping` at `path` (its keys from the root) and `pointer`: its values, then its tables.
fn write_table(
    mapping: &Map<String, Value>,
    path: &[&str],
    pointer: &str,
    comments: &Comments,
    out: &mut String,
) -> Result<(), Unwritable> {
    let (nested, values): (Vec<_>, Vec<_>) = mapping
        .iter()
        .partition(|(_, value)| is_table(value) || is_table_list(value));
    for (key, value) in values {
        let at = format!("{pointer}/{key}");
        comments.write_above(&at, "", out);
        out.push_str(&key_text(key));
        out.push_str(" = ");
        out.push_str(&inline(value, &at)?);
        out.push('\n');
    }
    for (key, value) in nested {
        let at = format!("{pointer}/{key}");
        let mut inner = path.to_vec();
        inner.push(key);
        let header = inner
            .iter()
            .map(|key| key_text(key))
            .collect::<Vec<_>>()
            .join(".");
        match value {
            Value::Object(entries) => {
                separate(out);
                comments.write_above(&at, "", out);
                // A table of tables only needs no header of its own.
                if entries
                    .values()
                    .any(|value| !is_table(value) && !is_table_list(value))
                {
                    let _ = writeln!(out, "[{header}]");
                }
                write_table(entries, &inner, &at, comments, out)?;
            }
            Value::Array(items) => {
                for (index, item) in items.iter().enumerate() {
                    let Value::Object(entries) = item else {
                        continue;
                    };
                    separate(out);
                    if index == 0 {
                        comments.write_above(&at, "", out);
                    }
                    let _ = writeln!(out, "[[{header}]]");
                    write_table(entries, &inner, &format!("{at}/{index}"), comments, out)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// A blank line before a header, unless it is the first line.
fn separate(out: &mut String) {
    if !out.is_empty() && !out.ends_with("\n\n") {
        out.push('\n');
    }
}

/// `value`, at `pointer`, on one line.
fn inline(value: &Value, pointer: &str) -> Result<String, Unwritable> {
    let unwritable = || Unwritable(pointer.to_owned());
    Ok(match value {
        Value::Null => return Err(unwritable()),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => {
            if number.is_u64() && number.as_i64().is_none() {
                return Err(unwritable());
            }
            number.to_string()
        }
        Value::String(text) => string_text(text),
        Value::Array(items) => {
            let items = items
                .iter()
                .enumerate()
                .map(|(index, item)| inline(item, &format!("{pointer}/{index}")))
                .collect::<Result<Vec<_>, _>>()?;
            format!("[{}]", items.join(", "))
        }
        Value::Object(entries) if entries.is_empty() => "{}".into(),
        Value::Object(entries) => {
            let entries = entries
                .iter()
                .map(|(key, value)| {
                    let value = inline(value, &format!("{pointer}/{key}"))?;
                    Ok(format!("{} = {value}", key_text(key)))
                })
                .collect::<Result<Vec<_>, _>>()?;
            format!("{{ {} }}", entries.join(", "))
        }
    })
}

/// `key`, bare when TOML allows it, else quoted.
fn key_text(key: &str) -> String {
    let bare = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if bare {
        key.to_owned()
    } else {
        string_text(key)
    }
}

/// `text` as a TOML basic string.
fn string_text(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04X}", u32::from(c));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests;
