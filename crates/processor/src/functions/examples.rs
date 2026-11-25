//! The catalog's examples, evaluated: `#[library]` writes a test for each function's (`compmd_macros::library`), which
//! calls [`assert_gives`]; the macros' are checked here.

use super::{Example, macros};
use crate::cel::overload;
use crate::cel::{Program, Scope, Type, Value, Values};
use crate::md::{Md, Signatures};
use serde_json::Value as Json;

/// The render the examples are evaluated at, so those that read its time, time zone or language give what they say:
/// 2026-10-21T10:00:00Z (in nanoseconds since 1970), in UTC, in English. The catalog's docs say it in words.
const NOW: i128 = 1_792_576_800 * 1_000_000_000;
const TZ: &str = "UTC";
const LANG: &str = "en";

/// Asserts that `example` evaluates to its result, at the examples' render.
///
/// # Panics
///
/// When it doesn't, with what it gave.
pub(crate) fn assert_gives(example: &overload::Example) {
    assert_example(&super::example(example));
}

/// Asserts that `example` evaluates to its result.
fn assert_example(example: &Example) {
    let given = evaluate(&example.expression, &example.given);
    assert!(
        given
            .as_ref()
            .is_ok_and(|given| same(given, &example.result)),
        "{} gives {given:?}, not {}",
        example.expression,
        example.result
    );
}

/// The variables `given` names, as an expression sees them: `md.doc` and `md.data` are fields of one variable, `md`.
fn variables(given: &serde_json::Map<String, Json>) -> serde_json::Map<String, Json> {
    let mut variables = serde_json::Map::new();
    let mut md = serde_json::Map::new();
    for (name, value) in given {
        match name.strip_prefix("md.") {
            Some(field) => md.insert(field.to_string(), value.clone()),
            None => variables.insert(name.clone(), value.clone()),
        };
    }
    if !md.is_empty() {
        variables.insert("md".to_string(), Json::Object(md));
    }
    variables
}

/// What `expression` gives with `given`, at the catalog's render, after checking it as a document's is checked.
fn evaluate(expression: &str, given: &serde_json::Map<String, Json>) -> Result<Json, String> {
    let program = Program::parse(expression).map_err(|e| e.message().to_string())?;
    let given = &variables(given);
    let scope = given.iter().fold(Scope::default(), |scope, (name, value)| {
        scope.with(name.clone(), Type::of_json(value))
    });
    program
        .check(&scope, &Signatures)
        .map_err(|e| format!("refused: {}", e.message()))?;
    let values = given
        .iter()
        .fold(Values::default(), |values, (name, value)| {
            values.with(name.clone(), Value::from_json(value))
        });
    let render = Md::new(NOW, LANG, TZ, &());
    program
        .evaluate(&values, &render)
        .map(|value| value.to_json())
        .map_err(|e| e.message().to_string())
}

/// Whether two results are the same: numbers by value, so `60` is `60.0`.
pub(super) fn same(a: &Json, b: &Json) -> bool {
    match (a, b) {
        (Json::Number(x), Json::Number(y)) => x.as_f64() == y.as_f64(),
        (Json::Array(x), Json::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| same(x, y))
        }
        (Json::Object(x), Json::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .zip(y)
                    .all(|((kx, x), (ky, y))| kx == ky && same(x, y))
        }
        _ => a == b,
    }
}

#[test]
fn every_macro_example_gives_its_result() {
    for function in macros::all() {
        for example in &function.examples {
            assert_example(example);
        }
    }
}

#[test]
#[should_panic(expected = "size(\"ab\") gives Ok(Number(2)), not 3")]
fn a_wrong_example_fails_its_test() {
    assert_gives(&overload::Example {
        expression: "size(\"ab\")",
        given: &[],
        result: "3",
    });
}
