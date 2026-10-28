//! The `md.` functions' errors, and the edges of what they take. Values of type `dyn` get past the checker, as
//! values from data whose records differ do.

use super::{Md, Signatures};
use crate::cel::{Program, Scope, Type, Value, Values};
use serde_json::{Value as Json, json};

/// 2026-10-27T09:00:00Z.
const NOW: i128 = 1_793_091_600 * 1_000_000_000;

/// Checks and evaluates `source` in `lang`, with `x` of type `dyn`.
fn run_at(now: i128, lang: &str, source: &str, x: &Json) -> Result<Json, String> {
    let program = Program::parse(source).map_err(|e| e.message().to_string())?;
    let scope = Scope::default().with("x", Type::Dyn);
    program
        .check(&scope, &Signatures)
        .map_err(|e| format!("refused: {}", e.message()))?;
    let values = Values::default().with("x", Value::from_json(x));
    program
        .evaluate(&values, &Md::new(now, lang, "UTC", &()))
        .map(|value| value.typed_json())
        .map_err(|e| e.message().to_string())
}

fn run(source: &str, x: &Json) -> Result<Json, String> {
    run_at(NOW, "en", source, x)
}

fn says(message: &str) -> Result<Json, String> {
    Err(message.to_string())
}

#[test]
fn dates_must_be_dates() {
    assert_eq!(
        run("md.addDays(x, 1)", &json!(1)),
        says("md.addDays() doesn't take (double, int)")
    );
    assert_eq!(
        run("md.addDays(x, 1)", &json!("10/21/2026")),
        says("'10/21/2026' is not a date (YYYY-MM-DD)")
    );
    assert_eq!(
        run("md.addDays('2026-10-21', 1.5)", &json!(null)),
        says("1.5 is not a whole number")
    );
    assert_eq!(
        run("md.addDays('2026-10-21', x)", &json!("a")),
        says("md.addDays() doesn't take (string, string)")
    );
    assert_eq!(
        run("md.addDays('2026-10-21', 2u)", &json!(null)),
        Ok(json!("2026-10-23"))
    );
    assert_eq!(
        run("md.startOf('2026-10-21', 'day')", &json!(null)),
        says("md.startOf() takes the unit \"week\", \"month\" or \"year\", not \"day\"")
    );
    assert_eq!(
        run("md.date('2026-10-21', 'tiny')", &json!(null)),
        says("md.date() takes the style \"short\", \"medium\", \"long\" or \"full\", not \"tiny\"")
    );
    assert_eq!(
        run_at(i128::MAX, "en", "md.today()", &json!(null)),
        says("the timestamp is out of range")
    );
}

#[test]
fn numbers_and_currencies() {
    assert_eq!(
        run("md.number(1.5, 21)", &json!(null)),
        says("md.number() takes 0 to 20 digits, not 21")
    );
    assert_eq!(
        run("md.number(1.5, -1)", &json!(null)),
        says("md.number() takes 0 to 20 digits, not -1")
    );
    assert_eq!(
        run("md.number(x, 0)", &json!("a")),
        says("md.number() doesn't take (string, int)")
    );
    assert_eq!(
        run("md.number(1.0 / 0.0, 0)", &json!(null)),
        says("inf is not a finite number")
    );
    assert_eq!(
        run("md.number(1234567, 0)", &json!(null)),
        Ok(json!("1,234,567"))
    );
    assert_eq!(
        run("md.currency(12, 'JPY')", &json!(null)),
        Ok(json!("¥12"))
    );
    assert_eq!(
        run("md.currency(-1.5, 'USD')", &json!(null)),
        Ok(json!("-$1.50"))
    );
    assert_eq!(
        run("md.currency(1, 'dollars')", &json!(null)),
        says("'dollars' is not a currency code such as \"USD\"")
    );
}

#[test]
fn plurals_need_an_other_form() {
    assert_eq!(
        run("md.plural(1, {'one': 'a'})", &json!(null)),
        Ok(json!("a"))
    );
    assert_eq!(
        run("md.plural(2, {'one': 'a'})", &json!(null)),
        says("md.plural() needs an \"other\" form")
    );
    assert_eq!(
        run("md.plural(2, x)", &json!("forms")),
        says("md.plural() doesn't take (int, string)")
    );
    assert_eq!(
        run(
            "md.plural(2, {'one': 'a', 'fwe': 'b', 'other': 'c'})",
            &json!(null)
        ),
        says("md.plural() takes forms named by plural categories, not \"fwe\"")
    );
    assert_eq!(
        run("md.plural(2, {1: 'a'})", &json!(null)),
        says("md.plural() takes forms named by plural categories, not 1")
    );
    assert_eq!(
        run("md.plural(2, x)", &json!({ "other": 5 })),
        Ok(json!(5.0))
    );
    assert_eq!(
        run("md.plural(2, 'a')", &json!(null)),
        says("refused: md.plural() doesn't take (int, string)")
    );
    let forms = "{'zero': 'z', 'one': 'o', 'two': 't', 'few': 'f', 'many': 'm', 'other': 'x'}";
    let in_arabic = |n: u32| run_at(NOW, "ar", &format!("md.plural({n}, {forms})"), &json!(null));
    assert_eq!(
        [0, 1, 2, 3, 11, 100].map(in_arabic),
        ["z", "o", "t", "f", "m", "x"].map(|form| Ok(json!(form)))
    );
}

#[test]
fn lists_are_written_as_interpolation_writes_values() {
    assert_eq!(
        run("md.list([1.5, null, true])", &json!(null)),
        Ok(json!("1.5, , and true"))
    );
    assert_eq!(
        run(
            "md.list(x)",
            &json!([{ "name": "Ana Lima" }, { "name": "Kenji Sato" }])
        ),
        Ok(json!("Ana Lima and Kenji Sato"))
    );
    assert_eq!(
        run("md.list(x, 'and')", &json!(["a", "b"])),
        Ok(json!("a and b"))
    );
    assert_eq!(
        run("md.list(x)", &json!([[1]])),
        says("a list isn't written; use each, or md.list()")
    );
    assert_eq!(
        run("md.list(x, 'xor')", &json!([])),
        says("md.list() takes \"and\" or \"or\", not \"xor\"")
    );
    assert_eq!(
        run("md.list(x)", &json!("a")),
        says("md.list() doesn't take (string)")
    );
}

#[test]
fn list_functions_need_lists_of_the_right_values() {
    assert_eq!(
        run("md.sort(x)", &json!([1, "a"])),
        says("can't order a string and a double")
    );
    assert_eq!(
        run("md.sort(x)", &json!([3, 1, 2])),
        Ok(json!([1.0, 2.0, 3.0]))
    );
    assert_eq!(
        run("md.sort([1.0, 0.0 / 0.0, 0.5])", &json!(null)),
        Ok(json!([0.5, 1.0, null]))
    );
    assert_eq!(
        run("md.sortBy(x, 'a')", &json!([1])),
        says("expected records with the field 'a', not a double")
    );
    assert_eq!(
        run("md.sortBy(x, 'a')", &json!([{ "b": 1 }])),
        says("an element has no field 'a'")
    );
    assert_eq!(
        run(
            "md.sortBy(x, 'n').map(r, r.k)",
            &json!([{ "n": 2, "k": "b" }, { "n": 1, "k": "a" }, { "n": 2, "k": "c" }])
        ),
        Ok(json!(["a", "b", "c"]))
    );
    assert_eq!(
        run("md.sum(x)", &json!([1, "a"])),
        says("md.sum() needs numbers, not string")
    );
    assert_eq!(run("md.min(x)", &json!([3, 1])), Ok(json!(1.0)));
    assert_eq!(run("md.sum([1, 2u, 3.5])", &json!(null)), Ok(json!(6.5)));
    assert_eq!(
        run("md.sum(['a'])", &json!(null)),
        says("refused: md.sum() doesn't take (list(string))")
    );
    assert_eq!(run("md.sum(x)", &json!([1, 2.5])), Ok(json!(3.5)));
    assert_eq!(run("md.slice(x, 2, 1)", &json!([1, 2, 3])), Ok(json!([])));
    assert_eq!(
        run("md.slice(x, -1, 9)", &json!([1, 2])),
        Ok(json!([1.0, 2.0]))
    );
    assert_eq!(run("md.reverse(x)", &json!([])), Ok(json!([])));
    assert_eq!(
        run("md.sortBy(x, 1)", &json!([])),
        says("refused: md.sortBy() doesn't take (dyn, int)")
    );
    assert_eq!(
        run("md.sortBy(x, y)", &json!([])),
        says("refused: unknown name 'y'")
    );
    assert_eq!(
        run("md.groupBy(x, 1)", &json!([])),
        says("refused: md.groupBy() doesn't take (dyn, int)")
    );
}

#[test]
fn list_functions_that_cel_checks_only_when_evaluated() {
    let scope = Scope::default().with("x", Type::Dyn).with("f", Type::Dyn);
    let program = Program::parse("md.sortBy(x, f)").expect("parses");
    assert!(program.check(&scope, &Signatures).is_ok());
    let values = Values::default()
        .with("x", Value::from_json(&json!([])))
        .with("f", Value::from_json(&json!(1)));
    let error = program
        .evaluate(&values, &Md::new(NOW, "en", "UTC", &()))
        .expect_err("a field name");
    assert_eq!(
        error,
        crate::cel::Error::Evaluation {
            at: 3,
            message: "md.sortBy() doesn't take (list, double)".into()
        }
    );
}

#[test]
fn unique_finds_first_occurrences_of_any_value() {
    assert_eq!(
        run("md.unique(x)", &json!([1, 1.0, "1", true, 1, "1"])),
        Ok(json!([1.0, "1", true]))
    );
    assert_eq!(
        run(
            "md.unique(x)",
            &json!([{ "a": 1 }, [1], { "a": 1 }, 1.5, 1.5, [1]])
        ),
        Ok(json!([{ "a": 1.0 }, [1.0], 1.5]))
    );
    assert_eq!(
        run("md.unique([0.0 / 0.0, 0.0 / 0.0]).size()", &json!(null)),
        Ok(json!(2))
    );
}

#[test]
fn group_by_needs_values_that_can_be_keys() {
    assert_eq!(
        run("md.groupBy(x, 'n')", &json!([{ "n": 1.5 }])),
        says("can't group by 'n': 1.5 isn't a map key")
    );
    assert_eq!(
        run("md.groupBy(x, 'n')", &json!([{ "m": 1 }])),
        says("an element has no field 'n'")
    );
    assert_eq!(
        run(
            "md.groupBy(x, 'ok')",
            &json!([{ "ok": true }, { "ok": false }, { "ok": true }])
        ),
        Ok(json!({ "true": [{ "ok": true }, { "ok": true }], "false": [{ "ok": false }] }))
    );
}

#[test]
fn plural_forms_from_a_map_check_as_a_string() {
    let scope = Scope::default().with("forms", Type::of_json(&json!({ "one": "a", "other": "b" })));
    let program = Program::parse("md.plural(2, forms)").expect("parses");
    assert_eq!(program.check(&scope, &Signatures), Ok(Type::String));
}

#[test]
fn md_calls_check_their_argument_types() {
    let refused = |source: &str| run(source, &json!(null)).expect_err("refused");
    assert_eq!(
        refused("md.dateOf('2026-10-21')"),
        "refused: md.dateOf() doesn't take (string)"
    );
    assert_eq!(
        refused("md.daysBetween(1, 2)"),
        "refused: md.daysBetween() doesn't take (int, int)"
    );
    assert_eq!(
        refused("md.startOf(1, 'week')"),
        "refused: md.startOf() doesn't take (int, string)"
    );
    assert_eq!(
        refused("md.date(1, 'long')"),
        "refused: md.date() doesn't take (int, string)"
    );
    assert_eq!(
        refused("md.currency('a', 'USD')"),
        "refused: md.currency() doesn't take (string, string)"
    );
    assert_eq!(
        refused("md.list(1)"),
        "refused: md.list() doesn't take (int)"
    );
    assert_eq!(
        refused("md.slice([1], 'a', 1)"),
        "refused: md.slice() doesn't take (list(int), string, int)"
    );
    assert_eq!(refused("md.sum(1)"), "refused: md.sum() doesn't take (int)");
    assert_eq!(
        run("md.plural(1, {'one': 1, 'other': 2}) + 1", &json!(null)),
        Ok(json!(2))
    );
    assert_eq!(
        run("md.plural(1, md.groupBy([], 'a'))", &json!(null)),
        says("md.plural() needs an \"other\" form")
    );
}
