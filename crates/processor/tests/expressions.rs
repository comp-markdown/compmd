//! An expression outside a document, as a host uses `compmd::cel` and `compmd::md` for a computed field of a table:
//! parsed once, checked against the record type of `md.doc` for its result type, evaluated for each row with the real
//! `md.` functions, and the same values, errors and budget as the expression gets in a body.

use compmd::cel::{BUDGET, Program, Scope, Type, Value, Values};
use compmd::context::{RenderContext, parse_time};
use compmd::diagnostic::Diagnostic;
use compmd::expand::Options;
use compmd::md::{Md, Signatures};
use compmd::{Ambient, ParseOptions, Processor};
use serde_json::{Value as Json, json};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::rc::Rc;

/// The render's time: 2026-10-21T09:00:00Z.
const NOW: &str = "2026-10-21T09:00:00Z";

/// A table's rows: a title, a price, a count and a due date.
fn rows() -> Vec<Json> {
    vec![
        json!({ "title": "Harbour", "price": 12.5, "count": 3, "due": "2026-11-01" }),
        json!({ "title": "Lighthouse at dusk", "price": 4, "count": 10, "due": "2026-10-01" }),
    ]
}

/// The type of `md`: `md.doc` a record of the table's fields, its date a date.
fn md_type() -> Type {
    let fields = BTreeMap::from([
        ("title".to_string(), Type::String),
        ("price".to_string(), Type::Double),
        ("count".to_string(), Type::Double),
        ("due".to_string(), Type::Date),
    ]);
    Type::record(BTreeMap::from([("doc".to_string(), Type::record(fields))]))
}

/// The values of `md` for `row`: its fields, its date made with `Value::date`.
fn md_value(row: &Json) -> Value {
    let mut doc = row.clone();
    let due = Value::date(row["due"].as_str().expect("a date")).expect("an ISO date");
    doc["due"] = due.to_json();
    Value::from_json(&json!({ "doc": doc }))
}

fn now() -> i128 {
    parse_time(NOW).expect("a time")
}

/// The same expression in a body: `md.evaluate` in a document whose frontmatter is `row`.
fn in_a_body(row: &Json, expression: &str) -> Result<Json, Box<Diagnostic>> {
    let frontmatter =
        row.as_object()
            .expect("a row")
            .iter()
            .fold(String::new(), |mut text, (key, value)| {
                let _ = writeln!(text, "{key}: {value}");
                text
            });
    let context = RenderContext {
        now: now(),
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    };
    Processor::new("row.comp.md", format!("---\n{frontmatter}---\n"))
        .parse(&ParseOptions::default())
        .resolve((), (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&Options::default())
        .evaluate(expression)
}

/// `expression` checked once against `md_type()`, then evaluated for each row: its result type, and each row's value.
fn per_row(expression: &str) -> (Type, Vec<Result<Json, Box<Diagnostic>>>) {
    let program = Program::parse(expression).expect("an expression");
    let scope = Scope::default().with("md", md_type());
    let result_type = program.check(&scope, &Signatures).expect("it checks");
    let md = Md::new(now(), "en", "UTC", &());
    let values = rows()
        .iter()
        .map(|row| {
            program
                .evaluate(&Values::default().with("md", md_value(row)), &md)
                .map(|value| value.to_json())
                .map_err(|error| Box::new(error.diagnostic(expression)))
        })
        .collect();
    (result_type, values)
}

#[test]
fn an_expression_is_checked_once_for_its_type_and_evaluated_for_each_row() {
    let cases = [
        ("md.doc.price * md.doc.count", Type::Double),
        (
            "md.doc.title + ', due ' + md.date(md.doc.due, 'long')",
            Type::String,
        ),
        ("md.addDays(md.doc.due, 7)", Type::Date),
        ("md.doc.due < md.today()", Type::Bool),
        ("md.daysBetween(md.today(), md.doc.due)", Type::Int),
        ("md.now()", Type::Timestamp),
    ];
    for (expression, expected) in cases {
        let (result_type, values) = per_row(expression);
        assert_eq!(result_type, expected, "{expression}");
        for (row, value) in rows().iter().zip(values) {
            assert_eq!(
                value.map_err(|d| d.message),
                in_a_body(row, expression).map_err(|d| d.message),
                "{expression}"
            );
        }
    }
    let (_, values) = per_row("md.addDays(md.doc.due, 7)");
    assert_eq!(
        values.into_iter().map(Result::unwrap).collect::<Vec<_>>(),
        [json!("2026-11-08"), json!("2026-10-08")]
    );
}

#[test]
fn an_error_is_the_f024_a_body_reports_at_its_place_in_the_expression() {
    // A type error, refused when the expression is set.
    let expression = "md.doc.price +\n md.doc.title";
    let error = Program::parse(expression)
        .and_then(|program| program.check(&Scope::default().with("md", md_type()), &Signatures))
        .expect_err("a double and a string");
    let row = &rows()[0];
    assert_eq!(
        Some(error.diagnostic(expression)),
        in_a_body(row, expression).err().map(|d| *d)
    );
    // An evaluation error, in one row only: "Harbour" has 7 characters.
    let (_, values) = per_row("1 / (size(md.doc.title) - 7)");
    let first = values[0].clone().expect_err("divided by zero");
    assert_eq!(
        Some(first),
        in_a_body(row, "1 / (size(md.doc.title) - 7)").err()
    );
    assert!(values[1].is_ok());
}

#[test]
fn the_budget_is_a_body_s() {
    // Each of 50 characters joined with each of 50 with each of 50: far more than the budget's steps.
    let expression = "size(md.doc.title.split('').map(a, md.doc.title.split('').map(b, md.doc.title.split('').map(c, a + b + c))))";
    let row = json!({ "title": "x".repeat(50) });
    let program = Program::parse(expression).expect("an expression");
    let error = program
        .evaluate(
            &Values::default().with("md", Value::from_json(&json!({ "doc": row }))),
            &Md::new(now(), "en", "UTC", &()),
        )
        .expect_err("over the budget");
    const { assert!(BUDGET < 50 * 50 * 50) };
    assert_eq!(
        Some(error.diagnostic(expression)),
        in_a_body(&row, expression).err().map(|d| *d)
    );
}

#[test]
fn the_fields_an_expression_reads_are_found_to_rename_them() {
    let expression = "has(md.doc.due) ? md.doc.title + ' (md.doc.title)' : md.doc.title";
    let program = Program::parse(expression).expect("an expression");
    let chars: Vec<char> = expression.chars().collect();
    let found: Vec<(String, String)> = program
        .selections("md.doc")
        .into_iter()
        .map(|selection| (selection.name, chars[selection.range].iter().collect()))
        .collect();
    // Three, each covering its name in the source; the one in the string isn't one.
    assert_eq!(
        found,
        [("due", "due"), ("title", "title"), ("title", "title")]
            .map(|(a, b)| (a.to_string(), b.to_string()))
    );
    // Renamed from the last, so the ranges before stay where they are.
    let mut renamed = chars;
    for selection in program.selections("md.doc").into_iter().rev() {
        let id: Vec<char> = format!("f{}", selection.name.len()).chars().collect();
        renamed.splice(selection.range, id);
    }
    assert_eq!(
        renamed.into_iter().collect::<String>(),
        "has(md.doc.f3) ? md.doc.f5 + ' (md.doc.title)' : md.doc.f5"
    );
}
