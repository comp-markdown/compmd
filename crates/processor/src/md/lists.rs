//! The list functions of spec §8.2. They name a field instead of taking an expression; sorting is stable, and strings
//! sort by code point.

#![allow(
    clippy::unnecessary_wraps,
    reason = "every function has an overload's signature, which can fail"
)]

use super::{Md, number_of, whole};
use crate::cel::overload::{list, text};
use crate::cel::{Key, Value};
use indexmap::IndexMap;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::rc::Rc;

pub(super) fn sort(_: &Md, args: &[Value]) -> Result<Value, String> {
    sorted(list(args, 0), |item| Ok(item.clone()))
}

pub(super) fn sort_by(_: &Md, args: &[Value]) -> Result<Value, String> {
    let field = text(args, 1);
    sorted(list(args, 0), |item| field_of(item, field).cloned())
}

pub(super) fn reverse(_: &Md, args: &[Value]) -> Result<Value, String> {
    Ok(Value::list(list(args, 0).iter().rev().cloned().collect()))
}

/// The elements from the start up to the end, both clamped to the list.
pub(super) fn slice(_: &Md, args: &[Value]) -> Result<Value, String> {
    let items = list(args, 0);
    let bound = |n: &Value| whole(n).map(|n| usize::try_from(n).unwrap_or(0).min(items.len()));
    let (start, end) = (bound(&args[1])?, bound(&args[2])?);
    Ok(Value::list(items[start..end.max(start)].to_vec()))
}

/// The sum, `0` for an empty list.
pub(super) fn sum(_: &Md, args: &[Value]) -> Result<Value, String> {
    Ok(Value::Double(numbers("sum", args)?.into_iter().sum()))
}

/// The smallest, `null` for an empty list.
pub(super) fn min(_: &Md, args: &[Value]) -> Result<Value, String> {
    Ok(numbers("min", args)?
        .into_iter()
        .reduce(f64::min)
        .map_or(Value::Null, Value::Double))
}

/// The largest, `null` for an empty list.
pub(super) fn max(_: &Md, args: &[Value]) -> Result<Value, String> {
    Ok(numbers("max", args)?
        .into_iter()
        .reduce(f64::max)
        .map_or(Value::Null, Value::Double))
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

/// The numbers of the list argument.
fn numbers(name: &str, args: &[Value]) -> Result<Vec<f64>, String> {
    list(args, 0)
        .iter()
        .map(|item| {
            number_of(item).ok_or_else(|| format!("md.{name}() needs numbers, not {}", item.kind()))
        })
        .collect()
}

/// First occurrences, in order. Values that can be map keys are found by hashing, the rest one by one.
pub(super) fn unique(_: &Md, args: &[Value]) -> Result<Value, String> {
    let mut keys = HashSet::new();
    let mut others: Vec<&Value> = Vec::new();
    let mut out = Vec::new();
    for item in list(args, 0) {
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
    Ok(Value::list(out))
}

/// A map from each value of the field to the elements that have it, in order of first occurrence. A whole number
/// from data groups under an `int` key.
pub(super) fn group_by(_: &Md, args: &[Value]) -> Result<Value, String> {
    let field = text(args, 1);
    let mut groups: IndexMap<Key, Vec<Value>> = IndexMap::new();
    for item in list(args, 0) {
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
