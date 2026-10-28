//! The list functions of spec §8.2. They name a field instead of taking an expression; sorting is stable, and strings
//! sort by code point.

use super::{number_of, whole};
use crate::cel::{Key, Value};
use indexmap::IndexMap;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::rc::Rc;

/// Calls `md.<name>(items, rest…)`.
pub(super) fn call(name: &str, items: &[Value], rest: &[Value]) -> Result<Value, String> {
    match (name, rest) {
        ("sort", []) => sorted(items, |item| Ok(item.clone())),
        ("sortBy", [Value::String(field)]) => sorted(items, |item| field_of(item, field).cloned()),
        ("reverse", []) => Ok(Value::list(items.iter().rev().cloned().collect())),
        ("slice", [start, end]) => {
            let bound =
                |n: &Value| whole(n).map(|n| usize::try_from(n).unwrap_or(0).min(items.len()));
            let (start, end) = (bound(start)?, bound(end)?);
            Ok(Value::list(items[start..end.max(start)].to_vec()))
        }
        ("sum", []) => Ok(Value::Double(numbers(name, items)?.into_iter().sum())),
        ("min" | "max", []) => {
            let pick = if name == "min" { f64::min } else { f64::max };
            Ok(numbers(name, items)?
                .into_iter()
                .reduce(pick)
                .map_or(Value::Null, Value::Double))
        }
        ("unique", []) => Ok(Value::list(unique(items))),
        ("groupBy", [Value::String(field)]) => group_by(items, field),
        _ => {
            let kinds: Vec<&str> = std::iter::once("list")
                .chain(rest.iter().map(Value::kind))
                .collect();
            Err(format!("md.{name}() doesn't take ({})", kinds.join(", ")))
        }
    }
}

/// Each element as interpolation writes it (spec §8.1); `null` as nothing.
pub(super) fn written(items: &[Value]) -> Result<Vec<String>, String> {
    items
        .iter()
        .map(|item| {
            item.written()
                .map(Option::unwrap_or_default)
                .map_err(str::to_string)
        })
        .collect()
}

/// The `field` of a record.
fn field_of<'v>(item: &'v Value, field: &str) -> Result<&'v Value, String> {
    let Value::Map(entries) = item else {
        return Err(format!(
            "expected records with the field '{field}', not a {}",
            item.kind()
        ));
    };
    entries
        .get(&Key::String(Rc::from(field)))
        .ok_or_else(|| format!("an element has no field '{field}'"))
}

/// The elements in ascending order of their keys, stably: numbers by value, `NaN` last, strings by code point.
fn sorted(items: &[Value], key: impl Fn(&Value) -> Result<Value, String>) -> Result<Value, String> {
    let mut keyed = items
        .iter()
        .map(|item| Ok((key(item)?, item)))
        .collect::<Result<Vec<_>, String>>()?;
    let mut error = None;
    keyed.sort_by(|(a, _), (b, _)| {
        a.order(b).map_or_else(
            |()| {
                error.get_or_insert_with(|| {
                    format!("can't order a {} and a {}", a.kind(), b.kind())
                });
                Ordering::Equal
            },
            // Only a `NaN` leaves two numbers unordered.
            |ordering| ordering.unwrap_or_else(|| is_nan(a).cmp(&is_nan(b))),
        )
    });
    match error {
        Some(error) => Err(error),
        None => Ok(Value::list(
            keyed.into_iter().map(|(_, item)| item.clone()).collect(),
        )),
    }
}

fn is_nan(value: &Value) -> bool {
    matches!(value, Value::Double(d) if d.is_nan())
}

fn numbers(name: &str, items: &[Value]) -> Result<Vec<f64>, String> {
    items
        .iter()
        .map(|item| {
            number_of(item).map_err(|_| format!("md.{name}() needs numbers, not {}", item.kind()))
        })
        .collect()
}

/// First occurrences, in order. Values that can be map keys are found by hashing, the rest one by one.
fn unique(items: &[Value]) -> Vec<Value> {
    let mut keys = HashSet::new();
    let mut others: Vec<&Value> = Vec::new();
    let mut out = Vec::new();
    for item in items {
        let first = match Key::to_find(item) {
            Some(key) => keys.insert(key),
            None if others.iter().any(|other| other.equals(item)) => false,
            None => {
                others.push(item);
                true
            }
        };
        if first {
            out.push(item.clone());
        }
    }
    out
}

/// A map from each value of `field` to the elements that have it, in order of first occurrence. A whole number from
/// data groups under an `int` key.
fn group_by(items: &[Value], field: &str) -> Result<Value, String> {
    let mut groups: IndexMap<Key, Vec<Value>> = IndexMap::new();
    for item in items {
        let value = field_of(item, field)?;
        let key = Key::to_find(value)
            .ok_or_else(|| format!("can't group by '{field}': {} isn't a map key", value.text()))?;
        groups.entry(key).or_default().push(item.clone());
    }
    Ok(Value::Map(Rc::new(
        groups
            .into_iter()
            .map(|(key, group)| (key, Value::list(group)))
            .collect(),
    )))
}
