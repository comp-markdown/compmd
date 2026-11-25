//! The catalog against what it describes: each example evaluates to its result, every function the spec's tables
//! (§8.2) list is in it with the spec's names and examples, and every `md.` function in it is in the tables.

use super::{EXAMPLE_RENDER, Function, Kind, all};
use crate::cel::{Program, Scope, Type, Value, Values};
use crate::md::{Md, Signatures};
use serde_json::Value as Json;

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
    let render = Md::new(
        EXAMPLE_RENDER.now,
        EXAMPLE_RENDER.lang,
        EXAMPLE_RENDER.tz,
        &(),
    );
    program
        .evaluate(&values, &render)
        .map(|value| value.to_json())
        .map_err(|e| e.message().to_string())
}

/// Whether two results are the same: numbers by value, so `60` is `60.0`.
fn same(a: &Json, b: &Json) -> bool {
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
fn every_example_gives_its_result() {
    let mut wrong = Vec::new();
    for function in all() {
        if function.examples.is_empty() {
            wrong.push(format!("{} has no example", function.name));
        }
        for example in &function.examples {
            let given = evaluate(&example.expression, &example.given);
            if !given
                .as_ref()
                .is_ok_and(|given| same(given, &example.result))
            {
                wrong.push(format!(
                    "{}: {} gives {given:?}, not {}",
                    function.name, example.expression, example.result
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn every_param_is_named_and_every_function_described() {
    for function in all() {
        assert!(!function.description.is_empty(), "{}", function.name);
        assert_eq!(function.section, "8.2");
        for signature in &function.signatures {
            for param in &signature.params {
                assert!(
                    !param.name.starts_with("arg"),
                    "{}: {}",
                    function.name,
                    signature.written
                );
            }
        }
    }
}

/// A row of one of spec §8.2's tables of `md.` functions: the signatures it lists (`md.min(list)`), its example, and
/// its result when it is a value (`"$120.00"`).
struct Row {
    signatures: Vec<String>,
    example: String,
    result: Option<Json>,
}

/// The code spans of a table cell.
fn spans(cell: &str) -> Vec<String> {
    cell.split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// The rows of spec §8.2's two tables of `md.` functions.
fn spec_rows() -> Vec<Row> {
    let spec = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../spec/compmd-spec-v1.md"
    ))
    .expect("the spec, beside the crate");
    let section = spec
        .split("### 8.2 Expressions")
        .nth(1)
        .and_then(|rest| rest.split("### 8.3").next())
        .expect("section 8.2");
    section
        .lines()
        .filter(|line| line.starts_with("| `md."))
        .map(|line| {
            let cells: Vec<&str> = line.split(" | ").collect();
            let result = spans(cells[2])
                .first()
                .and_then(|value| serde_json::from_str(value).ok());
            Row {
                signatures: spans(cells[0]),
                example: spans(cells[1]).remove(0),
                result: result.filter(|_| {
                    cells[2]
                        .trim()
                        .trim_end_matches('|')
                        .trim()
                        .starts_with('`')
                }),
            }
        })
        .collect()
}

/// A signature as the spec writes it, `md.currency(n, code)`: its function's name and its params' names, those that
/// are names (`md.list(list, "or")` gives `list`).
fn spec_signature(written: &str) -> (String, Vec<String>) {
    let (name, params) = written
        .trim_end_matches(')')
        .split_once('(')
        .expect("a call");
    let names = params
        .split(", ")
        .filter(|param| !param.is_empty() && !param.starts_with('"'))
        .map(str::to_string)
        .collect();
    (name.to_string(), names)
}

#[test]
fn the_catalog_has_every_function_of_the_specs_tables_with_its_names_and_examples() {
    let catalog = all();
    let find = |name: &str| catalog.iter().find(|function| function.name == name);
    let rows = spec_rows();
    assert!(rows.len() > 15, "the spec's tables were read");
    for row in &rows {
        let functions: Vec<&Function> = row
            .signatures
            .iter()
            .map(|written| {
                let (name, names) = spec_signature(written);
                let function = find(&name).unwrap_or_else(|| panic!("{name} is in the catalog"));
                assert!(
                    function.signatures.iter().any(|signature| {
                        let ours: Vec<&str> = signature
                            .params
                            .iter()
                            .map(|param| param.name.as_str())
                            .collect();
                        ours.starts_with(&names.iter().map(String::as_str).collect::<Vec<_>>())
                    }),
                    "{written}: the catalog names its params as the spec does"
                );
                function
            })
            .collect();
        let example = functions
            .iter()
            .flat_map(|function| &function.examples)
            .find(|example| example.expression == row.example)
            .unwrap_or_else(|| panic!("{} is an example of {:?}", row.example, row.signatures));
        if let Some(result) = &row.result {
            assert!(
                same(&example.result, result),
                "{}: {} as the spec says",
                row.example,
                result
            );
        }
    }
    let listed: Vec<String> = rows
        .iter()
        .flat_map(|row| {
            row.signatures
                .iter()
                .map(|written| spec_signature(written).0)
        })
        .collect();
    for function in catalog.iter().filter(|function| function.kind == Kind::Md) {
        assert!(
            listed.contains(&function.name),
            "{} is in the spec's tables",
            function.name
        );
    }
}

#[test]
fn the_catalog_is_json() {
    let json = serde_json::to_value(all()).expect("JSON");
    let size = json
        .as_array()
        .and_then(|functions| functions.iter().find(|function| function["name"] == "size"))
        .expect("size");
    assert_eq!(size["kind"], "cel");
    assert_eq!(
        size["signatures"][0]["params"][0],
        serde_json::json!({ "name": "value", "type": "string | list | map" })
    );
    assert_eq!(size["signatures"][1]["receiver"], "string | list | map");
    assert_eq!(size["section"], "8.2");
}
