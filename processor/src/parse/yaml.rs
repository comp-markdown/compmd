//! YAML 1.2 with the core schema, as JSON (spec §2.1), built from yaml-rust2's events so that tags, anchors,
//! aliases and repeated keys are refused with where they are (F033), and keys and values keep their places.

use serde_json::{Map, Number, Value};
use std::collections::{HashMap, HashSet};
use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser};
use yaml_rust2::scanner::{Marker, Scanner, TScalarStyle, Token, TokenType};

/// A place in the YAML text: 1-based line, 0-based column in code points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Mark {
    pub(crate) line: usize,
    pub(crate) column: usize,
}

impl From<Marker> for Mark {
    fn from(marker: Marker) -> Self {
        Self {
            line: marker.line(),
            column: marker.col(),
        }
    }
}

/// YAML that spec §2.1 refuses: where, and why.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Refused(pub(crate) Mark, pub(crate) &'static str);

/// A YAML document as JSON, with where each key and each value is, by JSON pointer (`/md/syntax/0`).
#[derive(Debug, Default)]
pub(crate) struct Document {
    pub(crate) value: Value,
    pub(crate) keys: HashMap<String, Mark>,
    pub(crate) values: HashMap<String, Mark>,
}

/// Parses `text`.
///
/// # Errors
///
/// Where the text isn't YAML, or holds a tag, an anchor, an alias or a repeated key.
pub(crate) fn parse(text: &str) -> Result<Document, Refused> {
    // The parser's events mark the value after a tag or an anchor, so they are found where their token starts.
    for Token(marker, token) in Scanner::new(text.chars()) {
        let why = match token {
            TokenType::Anchor(_) => "an anchor",
            TokenType::Alias(_) => "an alias",
            TokenType::Tag(..) => "a tag",
            _ => continue,
        };
        return Err(Refused(marker.into(), why));
    }
    let mut builder = Builder::default();
    Parser::new_from_str(text)
        .load(&mut builder, false)
        .map_err(|error| Refused((*error.marker()).into(), "invalid YAML"))?;
    if let Some(mark) = builder.repeated {
        return Err(Refused(mark, "a repeated key"));
    }
    Ok(Document {
        value: builder.root.unwrap_or(Value::Null),
        keys: builder.keys,
        values: builder.values,
    })
}

enum Collection {
    Sequence {
        pointer: String,
        items: Vec<Value>,
    },
    Mapping {
        pointer: String,
        entries: Map<String, Value>,
        key: Option<String>,
        seen: HashSet<String>,
    },
}

#[derive(Default)]
struct Builder {
    open: Vec<Collection>,
    root: Option<Value>,
    keys: HashMap<String, Mark>,
    values: HashMap<String, Mark>,
    repeated: Option<Mark>,
}

impl Builder {
    /// The pointer of the value placed next.
    fn next_pointer(&self) -> String {
        match self.open.last() {
            Some(Collection::Sequence { pointer, items }) => format!("{pointer}/{}", items.len()),
            Some(Collection::Mapping {
                pointer,
                key: Some(key),
                ..
            }) => format!("{pointer}/{key}"),
            _ => String::new(),
        }
    }

    /// Places a value, or a mapping's key, that ends at `mark`.
    fn place(&mut self, value: Value, mark: Mark) {
        let pointer = self.next_pointer();
        match self.open.last_mut() {
            Some(Collection::Sequence { items, .. }) => {
                self.values.insert(pointer, mark);
                items.push(value);
            }
            Some(Collection::Mapping {
                pointer: map_pointer,
                entries,
                key,
                seen,
            }) => {
                if let Some(key) = key.take() {
                    self.values.insert(pointer, mark);
                    entries.insert(key, value);
                    return;
                }
                let name = match value {
                    Value::String(text) => text,
                    other => other.to_string(),
                };
                if !seen.insert(name.clone()) {
                    self.repeated.get_or_insert(mark);
                }
                self.keys.insert(format!("{map_pointer}/{name}"), mark);
                *key = Some(name);
            }
            None => self.root = Some(value),
        }
    }
}

impl MarkedEventReceiver for Builder {
    fn on_event(&mut self, event: Event, marker: Marker) {
        let mark = Mark::from(marker);
        match event {
            Event::Scalar(text, style, ..) => {
                let value = if style == TScalarStyle::Plain {
                    core(&text)
                } else {
                    Value::String(text)
                };
                self.place(value, mark);
            }
            Event::SequenceStart(..) => {
                let pointer = self.next_pointer();
                self.open.push(Collection::Sequence {
                    pointer,
                    items: Vec::new(),
                });
            }
            Event::MappingStart(..) => {
                let pointer = self.next_pointer();
                self.open.push(Collection::Mapping {
                    pointer,
                    entries: Map::new(),
                    key: None,
                    seen: HashSet::new(),
                });
            }
            Event::SequenceEnd | Event::MappingEnd => {
                let value = match self.open.pop() {
                    Some(Collection::Sequence { items, .. }) => Value::Array(items),
                    Some(Collection::Mapping { entries, .. }) => Value::Object(entries),
                    None => Value::Null,
                };
                self.place(value, mark);
            }
            _ => {}
        }
    }
}

/// A plain scalar in the core schema: null, a boolean, an integer, a float or a string. Infinities and NaN have no
/// JSON form, so they stay strings.
fn core(text: &str) -> Value {
    match text {
        "" | "~" | "null" | "Null" | "NULL" => return Value::Null,
        "true" | "True" | "TRUE" => return Value::Bool(true),
        "false" | "False" | "FALSE" => return Value::Bool(false),
        _ => {}
    }
    integer(text)
        .map(Value::from)
        .or_else(|| float(text).and_then(Number::from_f64).map(Value::Number))
        .unwrap_or_else(|| Value::String(text.to_string()))
}

fn integer(text: &str) -> Option<i64> {
    if let Some(octal) = text.strip_prefix("0o") {
        return i64::from_str_radix(octal, 8).ok();
    }
    if let Some(hex) = text.strip_prefix("0x") {
        return i64::from_str_radix(hex, 16).ok();
    }
    let digits = text.strip_prefix(['-', '+']).unwrap_or(text);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.strip_prefix('+').unwrap_or(text).parse().ok()
}

/// `[-+]? ( \. [0-9]+ | [0-9]+ ( \. [0-9]* )? ) ( [eE] [-+]? [0-9]+ )?`
fn float(text: &str) -> Option<f64> {
    let unsigned = text.strip_prefix(['-', '+']).unwrap_or(text);
    let (mantissa, exponent) = match unsigned.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, Some(exponent)),
        None => (unsigned, None),
    };
    let digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
    let mantissa_ok = match mantissa.split_once('.') {
        Some((whole, fraction)) => {
            digits(whole) && digits(fraction) && !(whole.is_empty() && fraction.is_empty())
        }
        None => !mantissa.is_empty() && digits(mantissa),
    };
    let exponent_ok = exponent.is_none_or(|exponent| {
        let exponent = exponent.strip_prefix(['-', '+']).unwrap_or(exponent);
        !exponent.is_empty() && digits(exponent)
    });
    (mantissa_ok && exponent_ok)
        .then(|| text.parse().ok())
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn value(text: &str) -> Value {
        parse(text).expect("valid YAML").value
    }

    fn refused(text: &str) -> Mark {
        parse(text).expect_err("refused").0
    }

    #[test]
    fn reads_the_core_schema() {
        assert_eq!(
            value(
                "a: no\nb: true\nc: 2026-10-21\nd: 1.10\ne: 0x1f\nf: -3\ng: ~\nh: '1'\ni: .inf\nj: 1e3\nk: +2\nl: .5\nm: 1.\nn: 1e\no: 0o17"
            ),
            json!({
                "a": "no", "b": true, "c": "2026-10-21", "d": 1.1, "e": 31, "f": -3, "g": null, "h": "1",
                "i": ".inf", "j": 1000.0, "k": 2, "l": 0.5, "m": 1.0, "n": "1e", "o": 15,
            })
        );
    }

    #[test]
    fn keeps_nesting_and_key_order() {
        assert_eq!(
            value("z: [1, {b: 2, a: 3}]\ny:\n  - x"),
            json!({ "z": [1, { "b": 2, "a": 3 }], "y": ["x"] })
        );
        assert_eq!(value("1: a"), json!({ "1": "a" }));
        assert_eq!(value(""), Value::Null);
    }

    #[test]
    fn marks_keys_and_values() {
        let document = parse("md:\n  version: 1.1\n  syntax: [a, b]").expect("valid YAML");
        assert_eq!(document.keys["/md"], Mark { line: 1, column: 0 });
        assert_eq!(document.keys["/md/syntax"], Mark { line: 3, column: 2 });
        assert_eq!(
            document.values["/md/version"],
            Mark {
                line: 2,
                column: 11
            }
        );
        assert_eq!(
            document.values["/md/syntax/1"],
            Mark {
                line: 3,
                column: 14
            }
        );
    }

    #[test]
    fn refuses_tags_anchors_aliases_and_repeated_keys() {
        assert_eq!(refused("a: !!str 1"), Mark { line: 1, column: 3 });
        assert_eq!(refused("a: &x 1\nb: *x"), Mark { line: 1, column: 3 });
        assert_eq!(refused("a: 1\nb: 2\na: 3"), Mark { line: 3, column: 0 });
        assert_eq!(refused("a: [1"), Mark { line: 2, column: 0 });
        assert_eq!(
            parse("a: *x").unwrap_err(),
            Refused(Mark { line: 1, column: 3 }, "an alias")
        );
    }
}
