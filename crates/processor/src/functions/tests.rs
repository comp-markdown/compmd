//! The catalog against the spec: every function the spec's tables (§8.2) list is in it with the spec's names and
//! examples, every `md.` function in it is in the tables, and §8.2 names every function. Each example's result is
//! checked by its own test (`examples`).

use super::examples::same;
use super::{Function, Kind, all};
use serde_json::Value as Json;

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

/// Spec §8.2's text.
fn spec_section() -> String {
    let spec = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../spec/compmd-spec-v1.md"
    ))
    .expect("the spec, beside the crate");
    spec.split("### 8.2 Expressions")
        .nth(1)
        .and_then(|rest| rest.split("### 8.3").next())
        .expect("section 8.2")
        .to_string()
}

/// The rows of spec §8.2's two tables of `md.` functions.
fn spec_rows() -> Vec<Row> {
    spec_section()
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
fn the_spec_names_every_function() {
    let section = spec_section();
    let missing: Vec<String> = all()
        .into_iter()
        .map(|function| function.name)
        .filter(|name| {
            !section.contains(&format!("`{name}`")) && !section.contains(&format!("`{name}("))
        })
        .collect();
    assert!(missing.is_empty(), "spec §8.2 doesn't name {missing:?}");
}

#[test]
fn a_function_or_a_macro_is_found_by_its_name() {
    let written = |name: &str| {
        super::named(name).map(|function| {
            function
                .signatures
                .into_iter()
                .map(|signature| signature.written)
                .collect::<Vec<_>>()
        })
    };
    assert_eq!(
        written("indexOf"),
        Some(vec![
            "string.indexOf(substring: string) → int".to_string(),
            "string.indexOf(substring: string, start: int) → int".to_string()
        ])
    );
    assert_eq!(
        written("md.sum"),
        Some(vec!["md.sum(list: list(number)) → double".to_string()])
    );
    assert!(written("map").is_some() && written("has").is_some());
    assert_eq!(written("md.nothing"), None);
    assert_eq!(written("nothing"), None);
}

/// The names `of_type` gives for `ty`.
fn names_of(ty: &crate::cel::Type) -> Vec<String> {
    super::of_type(ty)
        .into_iter()
        .map(|function| function.name)
        .collect()
}

/// The code spans in the parentheses after `after` in spec §8.2: the getters it lists.
fn spec_list_after(section: &str, after: &str) -> Vec<String> {
    let rest = section.split(after).nth(1).expect(after);
    let listed = rest
        .split_once('(')
        .and_then(|(_, rest)| rest.split_once(')'));
    spans(listed.expect("a list in parentheses").0)
}

#[test]
fn a_timestamp_and_a_duration_take_the_getters_the_spec_lists() {
    use crate::cel::Type;
    let section = spec_section();
    let timestamp = spec_list_after(&section, "A timestamp's getters");
    let duration = spec_list_after(&section, "A duration's");
    assert_eq!(
        timestamp.len(),
        10,
        "the spec's timestamp getters were read"
    );
    assert_eq!(names_of(&Type::Timestamp), timestamp);
    assert_eq!(names_of(&Type::Duration), duration);
}

#[test]
fn each_type_takes_its_methods_and_nothing_outside_the_profile() {
    use crate::cel::Type;
    let comprehensions = ["all", "exists", "exists_one", "filter", "map"];
    let strings = [
        "size",
        "contains",
        "startsWith",
        "endsWith",
        "charAt",
        "indexOf",
        "lastIndexOf",
        "lowerAscii",
        "upperAscii",
        "replace",
        "split",
        "substring",
        "trim",
    ];
    assert_eq!(names_of(&Type::String), strings);
    assert_eq!(names_of(&Type::Date), strings, "a date is a string");
    let list = |extra: &[&str]| -> Vec<String> {
        ["size"]
            .iter()
            .chain(extra)
            .chain(&comprehensions)
            .map(ToString::to_string)
            .collect()
    };
    assert_eq!(names_of(&Type::list(Type::String)), list(&["join"]));
    assert_eq!(names_of(&Type::list(Type::Int)), list(&[]));
    assert_eq!(names_of(&Type::map(Type::String, Type::Int)), list(&[]));
    for none in [Type::Int, Type::Double, Type::Bool, Type::Null] {
        assert_eq!(names_of(&none), Vec::<String>::new(), "{none}");
    }
    let every = names_of(&Type::Dyn);
    let catalog: Vec<String> = all().into_iter().map(|function| function.name).collect();
    for name in &every {
        assert!(catalog.contains(name), "{name} is in the catalog");
        assert!(
            !crate::cel::functions::LEFT_OUT.contains(&name.as_str()),
            "{name} is in the profile"
        );
    }
    assert!(every.contains(&"join".to_string()) && every.contains(&"getHours".to_string()));
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
