//! The profile's macros (spec §8.2): `has`, and the comprehensions on a list or a map, whose first argument names
//! each element. They are syntax, not functions with overloads, so their signatures are written here.

use super::{Example, Function, Kind, Param, SECTION, signature};
use crate::cel::ast::Macro;
use serde_json::{Map, Value as Json};

/// Every macro: `has`, then the comprehensions.
pub(super) fn all() -> Vec<Function> {
    std::iter::once(has()).chain(comprehensions()).collect()
}

/// The comprehensions, which a list or a map is called with: `list.map(x, …)`.
pub(super) fn comprehensions() -> Vec<Function> {
    Macro::ALL.iter().copied().map(comprehension).collect()
}

/// The macro written `name`.
pub(super) fn named(name: &str) -> Option<Function> {
    if name == "has" {
        return Some(has());
    }
    Macro::from_name(name).map(comprehension)
}

/// `has(x.field)`.
fn has() -> Function {
    Function {
        name: "has".to_string(),
        kind: Kind::Macro,
        signatures: vec![signature(
            "has",
            None,
            vec![param("x.field", "field")],
            "bool",
        )],
        description: "true when the map or record has the field".to_string(),
        examples: vec![example(
            "has(md.doc.title)",
            Some(("md.doc", serde_json::json!({ "title": "Notes" }))),
            Json::Bool(true),
        )],
        section: SECTION.to_string(),
    }
}

/// A comprehension: `list.all(x, predicate)` and the others.
fn comprehension(found: Macro) -> Function {
    let (body, result, description, expression, gives) = match found {
        Macro::All => (
            "predicate",
            "bool",
            "true when every element satisfies the predicate",
            "[1, 2, 3].all(x, x > 0)",
            Json::Bool(true),
        ),
        Macro::Exists => (
            "predicate",
            "bool",
            "true when any element satisfies the predicate",
            "[1, 2, 3].exists(x, x > 2)",
            Json::Bool(true),
        ),
        Macro::ExistsOne => (
            "predicate",
            "bool",
            "true when exactly one element satisfies the predicate",
            "[1, 2, 3].exists_one(x, x > 2)",
            Json::Bool(true),
        ),
        Macro::Filter => (
            "predicate",
            "list",
            "the elements that satisfy the predicate",
            "[1, 2, 3].filter(x, x > 1)",
            serde_json::json!([2, 3]),
        ),
        Macro::Map => (
            "expression",
            "list",
            "each element made into another; `map(x, filter, expression)` keeps those the filter keeps",
            "[1, 2, 3].map(x, x * 2)",
            serde_json::json!([2, 4, 6]),
        ),
    };
    let ty = if body == "predicate" { "bool" } else { "dyn" };
    Function {
        name: found.name().to_string(),
        kind: Kind::Macro,
        signatures: vec![signature(
            found.name(),
            // A map's comprehension ranges over its keys (spec §8.2).
            Some("list | map".to_string()),
            vec![param("x", "name"), param(body, ty)],
            result,
        )],
        description: description.to_string(),
        examples: vec![example(expression, None, gives)],
        section: SECTION.to_string(),
    }
}

fn param(name: &str, ty: &str) -> Param {
    Param {
        name: name.to_string(),
        ty: ty.to_string(),
    }
}

fn example(expression: &str, given: Option<(&str, Json)>, result: Json) -> Example {
    Example {
        expression: expression.to_string(),
        given: given
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect::<Map<_, _>>(),
        result,
    }
}
