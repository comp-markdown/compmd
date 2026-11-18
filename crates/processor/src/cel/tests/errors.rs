//! What F024 says, and where: type errors the checker finds, and errors only evaluation finds, as when a value's type
//! is known only then (`dyn`, as in data whose records differ). This file holds the helpers; each module is one group.

mod functions;
mod operators;
mod structures;

use crate::cel::parser::MAX_DEPTH;
use crate::cel::{BUDGET, Error, Program, Scope, Type, Value, Values};
use crate::md::{Md, Signatures};
use crate::tree::DeclaredType;
use serde_json::{Value as Json, json};

/// 2026-10-27T09:00:00Z.
const NOW: i128 = 1_793_091_600 * 1_000_000_000;

/// The scope every check sees: `md.doc`, a list of `double`s, a `length` param, and `x`, `y` of type `dyn`.
fn scope() -> Scope {
    Scope::default()
        .with(
            "md",
            Type::of_json(&json!({ "doc": { "title": "Release", "price": 12.0 } })),
        )
        .with("rows", Type::list(Type::Double))
        .with("gap", Type::declared(&DeclaredType::named("length")))
        .with("x", Type::Dyn)
        .with("y", Type::Dyn)
}

/// The type error in `source`: its column and message.
fn refused(source: &str) -> (usize, String) {
    match Program::parse(source).and_then(|program| program.check(&scope(), &Signatures)) {
        Ok(ty) => panic!("{source} checks, as {ty}"),
        Err(error) => (error.at() + 1, error.message().to_string()),
    }
}

/// The type of `source`.
fn type_of(source: &str) -> String {
    match Program::parse(source).and_then(|program| program.check(&scope(), &Signatures)) {
        Ok(ty) => ty.to_string(),
        Err(error) => panic!("{source}: {error}"),
    }
}

/// Evaluates `source` with `x` and `y` as given, without checking it first.
fn evaluate(source: &str, x: Json, y: Json) -> Result<Json, (usize, String)> {
    let program = Program::parse(source).map_err(|e| (e.at() + 1, e.message().to_string()))?;
    let md = json!({ "doc": { "title": "Release" } });
    let values = [("md", md), ("x", x), ("y", y)]
        .into_iter()
        .fold(Values::default(), |values, (name, json)| {
            values.with(name, Value::from_json(&json))
        });
    program
        .evaluate(&values, &Md::new(NOW, "en", "UTC", &()))
        .map(|value| value.typed_json())
        .map_err(|e| (e.at() + 1, e.message().to_string()))
}

/// The evaluation error of `source` with `x`.
fn fails(source: &str, x: Json) -> (usize, String) {
    match evaluate(source, x, Json::Null) {
        Ok(value) => panic!("{source} gives {value}"),
        Err(error) => error,
    }
}

fn says(column: usize, message: &str) -> (usize, String) {
    (column, message.to_string())
}
