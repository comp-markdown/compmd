//! CEL's `contains`, `startsWith` and `endsWith`, and the strings extension's functions of the profile. Each takes
//! its receiver first; positions and counts are in code points.

#![allow(
    clippy::unnecessary_wraps,
    reason = "every function has an overload's signature, which can fail"
)]

use super::Functions;
use super::overload::{int, list, text};
use super::value::Value;

pub(crate) fn contains(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    Ok(Value::Bool(text(args, 0).contains(text(args, 1))))
}

pub(crate) fn starts_with(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    Ok(Value::Bool(text(args, 0).starts_with(text(args, 1))))
}

pub(crate) fn ends_with(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    Ok(Value::Bool(text(args, 0).ends_with(text(args, 1))))
}

pub(crate) fn char_at(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    let chars = chars(args);
    let i = position(int(args, 1), chars.len())?;
    Ok(Value::string(
        &chars.get(i).map(ToString::to_string).unwrap_or_default(),
    ))
}

/// The first start of the needle at or after the position given, or -1.
pub(crate) fn index_of(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    let (chars, needle) = (chars(args), needle(args));
    let from = int(args, 2).map_or(Ok(0), |from| position(Some(from), chars.len()))?;
    let found = (from..=chars.len().saturating_sub(needle.len()))
        .find(|&i| chars[i..].starts_with(&needle));
    Ok(found.map_or(Value::Int(-1), count))
}

/// The last start of the needle at or before the position given, or -1.
pub(crate) fn last_index_of(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    let (chars, needle) = (chars(args), needle(args));
    let to = int(args, 2).map_or(Ok(chars.len()), |to| position(Some(to), chars.len()))?;
    let Some(last) = chars.len().checked_sub(needle.len()) else {
        return Ok(Value::Int(-1));
    };
    let found = (0..=to.min(last))
        .rev()
        .find(|&i| chars[i..].starts_with(&needle));
    Ok(found.map_or(Value::Int(-1), count))
}

pub(crate) fn lower_ascii(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    Ok(Value::string(&text(args, 0).to_ascii_lowercase()))
}

pub(crate) fn upper_ascii(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    Ok(Value::string(&text(args, 0).to_ascii_uppercase()))
}

pub(crate) fn trim(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    Ok(Value::string(text(args, 0).trim()))
}

/// Replaces the first `n` occurrences, or all of them when `n` is negative or not given.
pub(crate) fn replace(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    let (s, old, new) = (text(args, 0), text(args, 1), text(args, 2));
    Ok(Value::string(
        &match int(args, 3).and_then(|n| usize::try_from(n).ok()) {
            Some(n) => s.replacen(old, new, n),
            None => s.replace(old, new),
        },
    ))
}

/// Splits as Go's `strings.SplitN` does: into at most `n` parts when `n` is positive, every part when it is
/// negative or not given, none when it is zero. An empty separator splits into code points.
pub(crate) fn split(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    let (s, separator) = (text(args, 0), text(args, 1));
    let limit = int(args, 2).map_or(usize::MAX, |n| usize::try_from(n).unwrap_or(usize::MAX));
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
    Ok(Value::list(parts.into_iter().map(Value::string).collect()))
}

/// The code points from the start up to the end, or to the end of the string when it isn't given.
pub(crate) fn substring(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    let chars = chars(args);
    let start = position(int(args, 1), chars.len())?;
    let end = int(args, 2).map_or(Ok(chars.len()), |end| position(Some(end), chars.len()))?;
    if start > end {
        return Err(format!(
            "substring() from {start} to {end}: the start is after the end"
        ));
    }
    Ok(Value::string(&chars[start..end].iter().collect::<String>()))
}

/// The strings of the list joined by the separator, or by nothing.
pub(crate) fn join(_: &dyn Functions, args: &[Value]) -> Result<Value, String> {
    let parts = list(args, 0)
        .iter()
        .map(|item| match item {
            Value::String(s) => Ok(&**s),
            other => Err(format!("join() needs strings, not {}", other.kind())),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::string(&parts.join(text(args, 1))))
}

fn chars(args: &[Value]) -> Vec<char> {
    text(args, 0).chars().collect()
}

fn needle(args: &[Value]) -> Vec<char> {
    text(args, 1).chars().collect()
}

/// A code point position from 0 to `len`.
fn position(i: Option<i64>, len: usize) -> Result<usize, String> {
    let i = i.unwrap_or_default();
    usize::try_from(i)
        .ok()
        .filter(|i| *i <= len)
        .ok_or_else(|| format!("index {i} is out of range for a string of {len}"))
}

fn count(n: usize) -> Value {
    Value::Int(i64::try_from(n).unwrap_or(i64::MAX))
}
