//! Helpers the test files share. Each file uses some of them, and cargo also builds this file as a test of its own.
#![allow(
    dead_code,
    reason = "each test file uses some of the helpers, and this file's own build none"
)]

use compmd::diagnostic::Diagnostic;
use serde_json::Value;

/// A diagnostic as `(code, line, column)`.
pub(crate) fn at(code: &str, line: usize, column: usize) -> (String, usize, usize) {
    (code.to_string(), line, column)
}

/// Where a diagnostic is, as [`at`] writes it.
pub(crate) fn place(diagnostic: &Diagnostic) -> (String, usize, usize) {
    let start = diagnostic.range.start;
    (diagnostic.code.to_string(), start.line, start.column)
}

/// `value` without positions, at any depth.
pub(crate) fn without_positions(value: &Value) -> Value {
    without(value, &["position", "closePosition"])
}

/// `value` without the fields `keys`, at any depth.
pub(crate) fn without(value: &Value, keys: &[&str]) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(|item| without(item, keys)).collect()),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .filter(|(key, _)| !keys.contains(&key.as_str()))
                .map(|(key, value)| (key.clone(), without(value, keys)))
                .collect(),
        ),
        other => other.clone(),
    }
}
