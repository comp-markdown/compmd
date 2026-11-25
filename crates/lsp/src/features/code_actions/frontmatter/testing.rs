//! What the writers' tests share: frontmatter values of every shape, and reading a written frontmatter back with the
//! processor.

use compmd::frontmatter::Written;
use proptest::prelude::*;
use serde_json::{Map, Value};

/// Strings a writer could get wrong: syntax, look-alikes of other types, white space, what must be escaped.
const TRICKY: [&str; 20] = [
    "",
    " ",
    "a: b",
    "# no",
    "@myorg/ui@1",
    "./chart.jsx",
    "no",
    "true",
    "null",
    "~",
    "1.10",
    "0x1f",
    ".inf",
    "2026-10-21",
    "a\nb",
    "tab\there",
    "\u{7f}\u{85}\u{9f}",
    "\"'\\",
    "- x",
    "...",
];

/// A frontmatter key.
fn keys() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z][a-z0-9_-]{0,6}",
        ".{0,6}",
        proptest::sample::select(TRICKY.as_slice()).prop_map(str::to_owned),
    ]
}

/// A value at the end of a branch: `null` only when `nulls`.
fn leaves(nulls: bool) -> BoxedStrategy<Value> {
    let leaf = prop_oneof![
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::from),
        any::<f64>()
            .prop_filter("finite", |float| float.is_finite())
            .prop_map(Value::from),
        keys().prop_map(Value::String),
        ".{0,16}".prop_map(Value::String),
    ];
    if nulls {
        prop_oneof![leaf, Just(Value::Null)].boxed()
    } else {
        leaf.boxed()
    }
}

/// A frontmatter's mapping, three levels deep at most, with `null`s only when `nulls`.
pub(super) fn mappings(nulls: bool) -> impl Strategy<Value = Map<String, Value>> {
    let values = leaves(nulls).prop_recursive(3, 24, 4, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            prop::collection::vec((keys(), inner), 0..4)
                .prop_map(|entries| Value::Object(entries.into_iter().collect())),
        ]
    });
    prop::collection::vec((keys(), values), 0..5).prop_map(|entries| entries.into_iter().collect())
}

/// The value of the frontmatter whose text is `text`, between `fence`s, as the processor reads it.
pub(super) fn read_back(fence: &str, text: &str) -> Value {
    Written::of(&format!("{fence}\n{text}{fence}\n"))
        .unwrap_or_else(|| panic!("doesn't read:\n{text}"))
        .value
}
