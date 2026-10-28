//! CEL's `contains`, `startsWith` and `endsWith`, and the strings extension's functions of the profile. Positions
//! and counts are in code points.

use super::Error;
use super::value::Value;

/// Calls `s.name(args)`.
pub(crate) fn call(name: &str, s: &str, args: &[Value], at: usize) -> Result<Value, Error> {
    use Value::{Bool, Int, String as Text};
    let chars = || s.chars().collect::<Vec<_>>();
    let text = |t: &str| Ok(Value::string(t));
    match (name, args) {
        ("contains", [Text(t)]) => Ok(Bool(s.contains(&**t))),
        ("startsWith", [Text(t)]) => Ok(Bool(s.starts_with(&**t))),
        ("endsWith", [Text(t)]) => Ok(Bool(s.ends_with(&**t))),
        ("charAt", [Int(i)]) => {
            let chars = chars();
            let i = position(*i, chars.len(), at)?;
            Ok(Value::string(
                &chars.get(i).map(ToString::to_string).unwrap_or_default(),
            ))
        }
        ("indexOf", [Text(t)]) => Ok(index_of(&chars(), t, 0)),
        ("indexOf", [Text(t), Int(from)]) => {
            let chars = chars();
            let from = position(*from, chars.len(), at)?;
            Ok(index_of(&chars, t, from))
        }
        ("lastIndexOf", [Text(t)]) => {
            let chars = chars();
            Ok(last_index_of(&chars, t, chars.len()))
        }
        ("lastIndexOf", [Text(t), Int(to)]) => {
            let chars = chars();
            let to = position(*to, chars.len(), at)?;
            Ok(last_index_of(&chars, t, to))
        }
        ("lowerAscii", []) => text(&s.to_ascii_lowercase()),
        ("upperAscii", []) => text(&s.to_ascii_uppercase()),
        ("trim", []) => text(s.trim()),
        ("replace", [Text(old), Text(new)]) => text(&s.replace(&**old, new)),
        ("replace", [Text(old), Text(new), Int(n)]) => text(&match usize::try_from(*n) {
            Ok(n) => s.replacen(&**old, new, n),
            Err(_) => s.replace(&**old, new),
        }),
        ("split", [Text(separator)]) => Ok(split(s, separator, -1)),
        ("split", [Text(separator), Int(n)]) => Ok(split(s, separator, *n)),
        ("substring", [Int(start)]) => {
            let chars = chars();
            let start = position(*start, chars.len(), at)?;
            text(&chars[start..].iter().collect::<String>())
        }
        ("substring", [Int(start), Int(end)]) => {
            let chars = chars();
            let (start, end) = (
                position(*start, chars.len(), at)?,
                position(*end, chars.len(), at)?,
            );
            if start > end {
                return Err(Error::new(
                    at,
                    format!("substring() from {start} to {end}: the start is after the end"),
                ));
            }
            text(&chars[start..end].iter().collect::<String>())
        }
        _ => {
            let kinds: Vec<&str> = args.iter().map(Value::kind).collect();
            Err(Error::new(
                at,
                format!("string.{name}() doesn't take ({})", kinds.join(", ")),
            ))
        }
    }
}

/// A code point position from 0 to `len`.
fn position(i: i64, len: usize, at: usize) -> Result<usize, Error> {
    usize::try_from(i)
        .ok()
        .filter(|i| *i <= len)
        .ok_or_else(|| {
            Error::new(
                at,
                format!("index {i} is out of range for a string of {len}"),
            )
        })
}

fn count(n: usize) -> Value {
    Value::Int(i64::try_from(n).unwrap_or(i64::MAX))
}

/// The first start of `needle` at or after `from`, or -1.
fn index_of(chars: &[char], needle: &str, from: usize) -> Value {
    let needle: Vec<char> = needle.chars().collect();
    let found = (from..=chars.len().saturating_sub(needle.len()))
        .find(|&i| chars[i..].starts_with(&needle));
    found.map_or(Value::Int(-1), count)
}

/// The last start of `needle` at or before `to`, or -1.
fn last_index_of(chars: &[char], needle: &str, to: usize) -> Value {
    let needle: Vec<char> = needle.chars().collect();
    let Some(last) = chars.len().checked_sub(needle.len()) else {
        return Value::Int(-1);
    };
    let found = (0..=to.min(last))
        .rev()
        .find(|&i| chars[i..].starts_with(&needle));
    found.map_or(Value::Int(-1), count)
}

/// Splits on `separator` as Go's `strings.SplitN` does: into at most `n` parts when `n` is positive, every part when
/// it is negative, none when it is zero. An empty separator splits into code points.
fn split(s: &str, separator: &str, n: i64) -> Value {
    let limit = usize::try_from(n).unwrap_or(usize::MAX);
    let parts: Vec<&str> = if limit == 0 {
        Vec::new()
    } else if separator.is_empty() {
        let mut parts = Vec::new();
        let mut rest = s;
        while let Some(c) = rest.chars().next() {
            if parts.len() + 1 == limit {
                parts.push(rest);
                break;
            }
            parts.push(&rest[..c.len_utf8()]);
            rest = &rest[c.len_utf8()..];
        }
        parts
    } else {
        s.splitn(limit, separator).collect()
    };
    Value::list(parts.into_iter().map(Value::string).collect())
}
