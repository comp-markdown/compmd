//! The budget: what would build a string or a list far larger than what it is built of is stopped before it is built,
//! and comparisons cost what they visit. Each runaway case here would allocate gigabytes, or run for minutes, if it
//! were spent for only after it ran.

use super::*;
use crate::cel::eval::evaluate as evaluate_within;
use crate::cel::functions::Call;
use crate::cel::overload::Size;
use crate::cel::strings;
use serde_json::Value as Json;

/// The budget's error, at `column`.
fn too_much_work(column: usize) -> Result<Json, (usize, String)> {
    Err(says(column, "the expression takes too much work"))
}

/// A string of `n` `a`s.
fn letters(n: usize) -> Json {
    Json::String("a".repeat(n))
}

#[test]
fn a_replacement_is_spent_for_before_it_is_built() {
    // 50,000 replacements of 50,000 characters each: 2.5 billion characters.
    assert_eq!(
        evaluate("x.replace('a', x)", letters(50_000), Json::Null),
        too_much_work(3)
    );
    assert_eq!(
        evaluate("x.replace('a', 'bb', 2).size()", letters(5), Json::Null),
        Ok(json!(7))
    );
}

#[test]
fn a_join_is_spent_for_before_it_is_built() {
    // 50,000 parts with the 50,000-character string between each two.
    assert_eq!(
        evaluate("x.split('').join(x)", letters(50_000), Json::Null),
        too_much_work(13)
    );
    assert_eq!(
        evaluate("x.split('').join('-')", letters(3), Json::Null),
        Ok(json!("a-a-a"))
    );
}

#[test]
fn a_split_is_spent_for_by_its_parts_before_they_are_built() {
    assert_eq!(
        evaluate("x.split('')", letters(100_001), Json::Null),
        too_much_work(3)
    );
    assert_eq!(
        evaluate("x.split('a', 3).size()", letters(5), Json::Null),
        Ok(json!(3))
    );
}

#[test]
fn concatenation_is_spent_for_before_it_is_built() {
    assert_eq!(
        evaluate("x + x", letters(50_001), Json::Null),
        too_much_work(3)
    );
    assert_eq!(
        evaluate("[x, x] + [x]", letters(1), Json::Null),
        Ok(json!(["a", "a", "a"]))
    );
}

#[test]
fn a_written_list_is_spent_for_before_it_is_written() {
    // Three elements of 40,000 characters each, written as one string.
    assert_eq!(
        evaluate("md.list([y, y, y])", Json::Null, letters(40_000)),
        too_much_work(4)
    );
    assert_eq!(
        evaluate("md.list([y, y, y])", Json::Null, letters(1)),
        Ok(json!("a, a, and a"))
    );
}

#[test]
fn a_string_literal_costs_its_length() {
    let long = format!("'{}'", "a".repeat(100_001));
    assert_eq!(evaluate(&long, Json::Null, Json::Null), too_much_work(1));
}

#[test]
fn membership_and_equality_cost_the_values_they_compare() {
    // 1,000 records, each looked for in the 1,000: half a million comparisons.
    let rows = Json::Array((0..1_000).map(|n| json!(n)).collect());
    assert_eq!(
        evaluate("x.filter(r, r in x).size()", rows, Json::Null),
        too_much_work(15)
    );
    let long = Json::Array((0..100_000).map(|n| json!([n])).collect());
    assert_eq!(evaluate("x == y", long.clone(), long), too_much_work(3));
    assert_eq!(
        evaluate(
            "[[1, 2]] == [[1, 2]] && 2 in [1, 2] && 'a' in {'a': 1}",
            Json::Null,
            Json::Null
        ),
        Ok(json!(true))
    );
}

#[test]
fn the_sizes_spent_for_are_the_results_sizes() {
    let md = Md::new(NOW, "en", "UTC", &());
    let size = |value: Value| match value {
        Value::String(s) => s.chars().count(),
        Value::List(items) => items.len(),
        other => panic!("{other:?} has no size"),
    };
    let texts = |items: &[&str]| Value::list(items.iter().map(|s| Value::string(s)).collect());
    let cases: [(Call, Size, Vec<Value>); 9] = [
        (
            strings::replace,
            strings::replace_size,
            vec![
                Value::string("aéa"),
                Value::string("a"),
                Value::string("xyz"),
            ],
        ),
        (
            strings::replace,
            strings::replace_size,
            vec![Value::string("ab"), Value::string(""), Value::string("-")],
        ),
        (
            strings::replace,
            strings::replace_size,
            vec![
                Value::string("aaa"),
                Value::string("a"),
                Value::string(""),
                Value::Int(2),
            ],
        ),
        (
            strings::split,
            strings::split_size,
            vec![Value::string("a,b,,c"), Value::string(",")],
        ),
        (
            strings::split,
            strings::split_size,
            vec![Value::string("aéb"), Value::string("")],
        ),
        (
            strings::split,
            strings::split_size,
            vec![Value::string(""), Value::string(",")],
        ),
        (
            strings::split,
            strings::split_size,
            vec![Value::string("a,b,c"), Value::string(","), Value::Int(0)],
        ),
        (
            strings::join,
            strings::join_size,
            vec![texts(&["ab", "é"]), Value::string("--")],
        ),
        (strings::join, strings::join_size, vec![texts(&[])]),
    ];
    for (call, declared, args) in cases {
        let result = call(&md, &args).expect("calls");
        assert_eq!(declared(&args), size(result), "{args:?}");
    }
}

#[test]
fn what_is_spent_for_before_a_call_is_its_result_s_size() {
    let md = Md::new(NOW, "en", "UTC", &());
    // The fewest steps it evaluates within.
    let steps = |source: &str| {
        let program = Program::parse(source).expect("parses");
        (0..=BUDGET)
            .find(|&budget| evaluate_within(&program.expr, &Values::default(), &md, budget).is_ok())
            .expect("within the budget")
    };
    // Each call costs its nodes, its literals' characters, and its result's characters or elements.
    assert_eq!(steps("'abc'.replace('b', 'xyz')"), 4 + 3 + 1 + 3 + 5);
    assert_eq!(steps("'a,b'.split(',')"), 3 + 3 + 1 + 2);
    assert_eq!(steps("['ab', 'c'].join('--')"), 5 + 2 + 1 + 2 + 2 + 5);
    assert_eq!(steps("md.list(['ab'])"), 3 + 2 + 1 + 18);
}
