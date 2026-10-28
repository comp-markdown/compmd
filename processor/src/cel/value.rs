//! Values, their equality and order, their JSON form, and how they are written as text (spec §8.1).

use super::ast::Literal;
use super::time;
use indexmap::IndexMap;
use serde_json::Value as Json;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// A value of the profile. Strings, lists and maps are shared, so a value is cheap to clone.
#[derive(Clone, Debug)]
pub enum Value {
    /// A 64-bit signed integer.
    Int(i64),
    /// A 64-bit unsigned integer.
    Uint(u64),
    /// A 64-bit float: every number from frontmatter, data, params and inputs.
    Double(f64),
    /// A string.
    String(Rc<str>),
    /// A boolean.
    Bool(bool),
    /// `null`, also the value of an optional param without one.
    Null,
    /// A point in time, in nanoseconds since 1970-01-01T00:00:00Z.
    Timestamp(i128),
    /// A length of time, in nanoseconds.
    Duration(i128),
    /// A list.
    List(Rc<Vec<Value>>),
    /// A map, in the order its keys were given.
    Map(Rc<IndexMap<Key, Value>>),
}

/// A map key: an `int`, `uint`, `bool` or `string`, as in CEL. Number keys are equal by value, so `1` and `1u`
/// are one key.
#[derive(Clone, Debug)]
pub enum Key {
    /// A boolean key.
    Bool(bool),
    /// A signed integer key.
    Int(i64),
    /// An unsigned integer key.
    Uint(u64),
    /// A string key.
    String(Rc<str>),
}

impl Key {
    /// The key of `value`, when it can be one.
    #[must_use]
    pub fn of(value: &Value) -> Option<Self> {
        match value {
            Value::Bool(b) => Some(Self::Bool(*b)),
            Value::Int(i) => Some(Self::Int(*i)),
            Value::Uint(u) => Some(Self::Uint(*u)),
            Value::String(s) => Some(Self::String(Rc::clone(s))),
            _ => None,
        }
    }

    /// The key `value` finds in a map: its own, or for a whole `double` the integer of the same value, since
    /// number keys match by value.
    #[must_use]
    pub fn to_find(value: &Value) -> Option<Self> {
        match value {
            Value::Double(d) => whole(*d).map(Self::number),
            other => Self::of(other),
        }
    }

    /// The key of a whole number: an `int` where one fits.
    fn number(n: i128) -> Self {
        i64::try_from(n).map_or_else(
            |_| Self::Uint(u64::try_from(n).unwrap_or(u64::MAX)),
            Self::Int,
        )
    }

    fn as_number(&self) -> Option<i128> {
        match self {
            Self::Int(i) => Some(i128::from(*i)),
            Self::Uint(u) => Some(i128::from(*u)),
            _ => None,
        }
    }

    /// The key as a value.
    #[must_use]
    pub fn to_value(&self) -> Value {
        match self {
            Self::Bool(b) => Value::Bool(*b),
            Self::Int(i) => Value::Int(*i),
            Self::Uint(u) => Value::Uint(*u),
            Self::String(s) => Value::String(Rc::clone(s)),
        }
    }

    /// The key as a JSON object's key.
    fn to_json_key(&self) -> String {
        match self {
            Self::String(s) => s.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Int(i) => i.to_string(),
            Self::Uint(u) => u.to_string(),
        }
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            _ => self.as_number().is_some() && self.as_number() == other.as_number(),
        }
    }
}

impl Eq for Key {}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Bool(b) => (0u8, b).hash(state),
            Self::String(s) => (1u8, s).hash(state),
            number => (2u8, number.as_number()).hash(state),
        }
    }
}

/// The integer a `double` is equal to, if it is whole and fits a `uint` or an `int`.
#[allow(clippy::cast_possible_truncation)]
fn whole(d: f64) -> Option<i128> {
    // Below 2^64 and at least -2^63, every whole double is exact as an i128.
    (d.fract() == 0.0 && (-9_223_372_036_854_775_808.0..18_446_744_073_709_551_616.0).contains(&d))
        .then_some(d as i128)
}

impl Value {
    /// A string value.
    #[must_use]
    pub fn string(text: &str) -> Self {
        Self::String(Rc::from(text))
    }

    /// A list value.
    #[must_use]
    pub fn list(items: Vec<Self>) -> Self {
        Self::List(Rc::new(items))
    }

    /// The value of JSON: every number is a `double` (spec §8.2), an object a map from strings.
    ///
    /// ```
    /// use compmd::cel::Value;
    /// use serde_json::json;
    ///
    /// let value = Value::from_json(&json!({ "price": 12 }));
    /// assert_eq!(value.to_json(), json!({ "price": 12.0 }));
    /// ```
    #[must_use]
    pub fn from_json(json: &Json) -> Self {
        match json {
            Json::Null => Self::Null,
            Json::Bool(b) => Self::Bool(*b),
            Json::Number(n) => Self::Double(n.as_f64().unwrap_or(f64::NAN)),
            Json::String(s) => Self::string(s),
            Json::Array(items) => Self::list(items.iter().map(Self::from_json).collect()),
            Json::Object(fields) => Self::Map(Rc::new(
                fields
                    .iter()
                    .map(|(k, v)| (Key::String(Rc::from(k.as_str())), Self::from_json(v)))
                    .collect(),
            )),
        }
    }

    /// The JSON form: `int` and `double` stay apart (`7` against `7.0`); a non-finite `double` is `null`;
    /// timestamps and durations are written as `string()` writes them; map keys that aren't strings are written as
    /// text.
    #[must_use]
    pub fn to_json(&self) -> Json {
        match self {
            Self::Int(i) => Json::from(*i),
            Self::Uint(u) => Json::from(*u),
            Self::Double(d) => Json::from(*d),
            Self::String(s) => Json::from(&**s),
            Self::Bool(b) => Json::from(*b),
            Self::Null => Json::Null,
            Self::Timestamp(t) => Json::from(time::format_timestamp(*t)),
            Self::Duration(d) => Json::from(time::format_duration(*d)),
            Self::List(items) => Json::Array(items.iter().map(Self::to_json).collect()),
            Self::Map(entries) => Json::Object(
                entries
                    .iter()
                    .map(|(k, v)| (k.to_json_key(), v.to_json()))
                    .collect(),
            ),
        }
    }

    /// The name of its type, for messages.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Int(_) => "int",
            Self::Uint(_) => "uint",
            Self::Double(_) => "double",
            Self::String(_) => "string",
            Self::Bool(_) => "bool",
            Self::Null => "null",
            Self::Timestamp(_) => "timestamp",
            Self::Duration(_) => "duration",
            Self::List(_) => "list",
            Self::Map(_) => "map",
        }
    }

    /// CEL's equality: numbers are equal by value across `int`, `uint` and `double`; values of different types are
    /// unequal; `NaN` equals nothing.
    #[must_use]
    pub fn equals(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::List(a), Self::List(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.equals(y))
            }
            (Self::Map(a), Self::Map(b)) => {
                a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).is_some_and(|w| v.equals(w)))
            }
            (Self::Null, Self::Null) => true,
            _ => self.order(other) == Ok(Some(Ordering::Equal)),
        }
    }

    /// The order of two values, as `<` sees it: `Ok(None)` when a `NaN` leaves them unordered, `Err` when their
    /// types have no order between them.
    pub(crate) fn order(&self, other: &Self) -> Result<Option<Ordering>, ()> {
        #[allow(clippy::cast_precision_loss)]
        let ordering = match (self, other) {
            (Self::Int(a), Self::Int(b)) => Some(a.cmp(b)),
            (Self::Uint(a), Self::Uint(b)) => Some(a.cmp(b)),
            (Self::Double(a), Self::Double(b)) => a.partial_cmp(b),
            (Self::Int(a), Self::Uint(b)) => Some(i128::from(*a).cmp(&i128::from(*b))),
            (Self::Uint(a), Self::Int(b)) => Some(i128::from(*a).cmp(&i128::from(*b))),
            (Self::Int(a), Self::Double(b)) => compare_exact(i128::from(*a), *b),
            (Self::Uint(a), Self::Double(b)) => compare_exact(i128::from(*a), *b),
            (Self::Double(a), Self::Int(b)) => {
                compare_exact(i128::from(*b), *a).map(Ordering::reverse)
            }
            (Self::Double(a), Self::Uint(b)) => {
                compare_exact(i128::from(*b), *a).map(Ordering::reverse)
            }
            (Self::String(a), Self::String(b)) => Some(a.cmp(b)),
            (Self::Bool(a), Self::Bool(b)) => Some(a.cmp(b)),
            (Self::Timestamp(a), Self::Timestamp(b)) | (Self::Duration(a), Self::Duration(b)) => {
                Some(a.cmp(b))
            }
            _ => return Err(()),
        };
        Ok(ordering)
    }

    /// The value as text, as interpolation writes it (spec §8.1): `None` for `null`, which writes nothing, and a
    /// map by its string `name`.
    ///
    /// # Errors
    ///
    /// A list, or a map without a string `name`, which aren't written.
    pub fn written(&self) -> Result<Option<String>, &'static str> {
        Ok(Some(match self {
            Self::Null => return Ok(None),
            Self::String(s) => s.to_string(),
            Self::Double(d) => double_text(*d),
            Self::Map(entries) => match entries.get(&Key::String(Rc::from("name"))) {
                Some(Self::String(name)) => name.to_string(),
                _ => return Err("a map is written only when it has a string 'name'"),
            },
            Self::List(_) => return Err("a list isn't written; use each, or md.list()"),
            other => other.text(),
        }))
    }

    /// The text `string()` gives for a scalar.
    pub(crate) fn text(&self) -> String {
        match self {
            Self::Int(i) => i.to_string(),
            Self::Uint(u) => u.to_string(),
            Self::Double(d) => double_text(*d),
            Self::String(s) => s.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Timestamp(t) => time::format_timestamp(*t),
            Self::Duration(d) => time::format_duration(*d),
            Self::Null | Self::List(_) | Self::Map(_) => self.to_json().to_string(),
        }
    }
}

/// The order of an integer against a `double`, exactly: an `i128` holds every `int` and `uint`, and a whole
/// `double` in that range converts without loss.
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn compare_exact(integer: i128, d: f64) -> Option<Ordering> {
    if d.is_nan() {
        return None;
    }
    // Beyond ±1e30 no int or uint comes close; inside it, the double's whole part is exact as an i128.
    if d.abs() >= 1.0e30 {
        return Some(if d > 0.0 {
            Ordering::Less
        } else {
            Ordering::Greater
        });
    }
    let floor = d.floor();
    Some(integer.cmp(&(floor as i128)).then(if d > floor {
        Ordering::Less
    } else {
        Ordering::Equal
    }))
}

/// A `double` as JavaScript's `String(n)` writes it (spec §8.1): the shortest digits that read back as the same
/// number, plain from 1e-6 up to 1e21 and with an exponent outside (`1e+21`, `1.5e-7`), and `NaN`, `Infinity` and
/// `-Infinity`.
pub(crate) fn double_text(d: f64) -> String {
    if d.is_nan() {
        return "NaN".into();
    }
    if d.is_infinite() {
        return if d > 0.0 { "Infinity" } else { "-Infinity" }.into();
    }
    if d == 0.0 {
        return "0".into();
    }
    let sign = if d < 0.0 { "-" } else { "" };
    // Rust's shortest round-trip digits, as `d.ddde±x`.
    let scientific = format!("{:e}", d.abs());
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let count = digits.len();
    // The decimal point's place, counted from the first digit: 0.digits × 10^point.
    let point = exponent.parse::<isize>().unwrap_or(0) + 1;
    let body = match usize::try_from(point) {
        Ok(point) if count <= point && point <= 21 => {
            format!("{digits}{}", "0".repeat(point - count))
        }
        Ok(point) if (1..=21).contains(&point) => {
            format!("{}.{}", &digits[..point], &digits[point..])
        }
        _ if point > -6 && point <= 0 => format!("0.{}{digits}", "0".repeat(point.unsigned_abs())),
        _ => {
            let exponent = point - 1;
            let exponent_sign = if exponent < 0 { '-' } else { '+' };
            let (first, rest) = digits.split_at(1);
            let fraction = if rest.is_empty() {
                String::new()
            } else {
                format!(".{rest}")
            };
            format!(
                "{first}{fraction}e{exponent_sign}{}",
                exponent.unsigned_abs()
            )
        }
    };
    format!("{sign}{body}")
}

impl From<&Literal> for Value {
    fn from(literal: &Literal) -> Self {
        match literal {
            Literal::Int(i) => Self::Int(*i),
            Literal::Uint(u) => Self::Uint(*u),
            Literal::Double(d) => Self::Double(*d),
            Literal::String(s) => Self::string(s),
            Literal::Bool(b) => Self::Bool(*b),
            Literal::Null => Self::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_numbers_are_doubles_and_come_back_apart_from_ints() {
        let value = Value::from_json(&json!({ "n": 1, "list": [1.5, "a", null, true] }));
        assert_eq!(
            value.to_json(),
            json!({ "n": 1.0, "list": [1.5, "a", null, true] })
        );
        assert_eq!(Value::Int(7).to_json(), json!(7));
        assert_ne!(Value::Int(7).to_json(), json!(7.0));
        assert_eq!(Value::Uint(7).to_json(), json!(7));
        assert_eq!(Value::Double(f64::NAN).to_json(), Json::Null);
        assert_eq!(Value::Timestamp(0).to_json(), json!("1970-01-01T00:00:00Z"));
        assert_eq!(Value::Duration(1_500_000_000).to_json(), json!("1.5s"));
        let keys: IndexMap<Key, Value> = [
            (Key::Int(1), Value::Null),
            (Key::Uint(2), Value::Null),
            (Key::Bool(true), Value::Null),
        ]
        .into_iter()
        .collect();
        assert_eq!(
            Value::Map(Rc::new(keys)).to_json(),
            json!({ "1": null, "2": null, "true": null })
        );
    }

    #[test]
    fn numbers_are_equal_by_value_across_types() {
        assert!(Value::Int(2).equals(&Value::Double(2.0)));
        assert!(Value::Double(2.0).equals(&Value::Uint(2)));
        assert!(Value::Uint(2).equals(&Value::Int(2)));
        assert!(Value::Int(2).equals(&Value::Uint(2)));
        assert!(!Value::Int(-1).equals(&Value::Uint(u64::MAX)));
        assert!(!Value::Double(f64::NAN).equals(&Value::Double(f64::NAN)));
        assert!(!Value::Int(1).equals(&Value::string("1")));
        assert!(Value::Null.equals(&Value::Null));
        assert!(!Value::Int(i64::MAX).equals(&Value::Double(9_223_372_036_854_775_807.0)));
        let less = Ok(Some(Ordering::Less));
        assert_eq!(Value::Int(1).order(&Value::Double(1.5)), less);
        assert_eq!(Value::Double(-1.5).order(&Value::Int(-1)), less);
        assert_eq!(Value::Uint(3).order(&Value::Double(1e300)), less);
        assert_eq!(Value::Double(-1e300).order(&Value::Uint(3)), less);
        assert_eq!(Value::Int(1).order(&Value::Double(f64::NAN)), Ok(None));
        assert!(Value::Int(1).order(&Value::string("a")).is_err());
    }

    #[test]
    fn lists_and_maps_compare_by_their_elements() {
        let list = |items: &[i64]| Value::list(items.iter().map(|i| Value::Int(*i)).collect());
        assert!(list(&[1, 2]).equals(&list(&[1, 2])));
        assert!(!list(&[1, 2]).equals(&list(&[2, 1])));
        assert!(!list(&[1]).equals(&list(&[1, 1])));
        let map = |key: Key, value: i64| {
            Value::Map(Rc::new([(key, Value::Int(value))].into_iter().collect()))
        };
        assert!(map(Key::Int(1), 1).equals(&map(Key::Uint(1), 1)));
        assert!(!map(Key::Int(1), 1).equals(&map(Key::Int(1), 2)));
        assert!(!map(Key::Int(1), 1).equals(&map(Key::Bool(true), 1)));
    }

    #[test]
    fn number_keys_match_by_value() {
        let find = |value: Value| Key::to_find(&value);
        assert_eq!(find(Value::Double(1.0)), Some(Key::Int(1)));
        assert_eq!(find(Value::Double(1.5)), None);
        assert_eq!(
            find(Value::Double(1e19)),
            Some(Key::Uint(10_000_000_000_000_000_000))
        );
        assert_eq!(find(Value::Double(1e20)), None);
        assert_eq!(find(Value::Uint(1)), Some(Key::Int(1)));
        assert_eq!(find(Value::Null), None);
        assert_ne!(Key::Bool(true), Key::Int(1));
        assert_ne!(Key::String(Rc::from("1")), Key::Int(1));
        let mut seen = std::collections::HashSet::new();
        seen.insert(Key::Int(1));
        assert!(seen.contains(&Key::Uint(1)));
        assert_eq!(Key::Uint(3).to_value().kind(), "uint");
        assert_eq!(Key::Bool(true).to_value().kind(), "bool");
        assert_eq!(Key::String(Rc::from("a")).to_value().kind(), "string");
    }

    #[test]
    fn values_are_written_as_interpolation_writes_them() {
        let name = Value::from_json(&json!({ "name": "Ursula K. Le Guin" }));
        assert_eq!(name.written(), Ok(Some("Ursula K. Le Guin".into())));
        assert_eq!(Value::Null.written(), Ok(None));
        assert_eq!(Value::Double(12.5).written(), Ok(Some("12.5".into())));
        assert_eq!(Value::Bool(true).written(), Ok(Some("true".into())));
        assert_eq!(Value::string("a").written(), Ok(Some("a".into())));
        assert!(Value::list(vec![]).written().is_err());
        assert!(Value::from_json(&json!({ "name": 1 })).written().is_err());
        assert_eq!(Value::list(vec![Value::Int(1)]).text(), "[1]");
        assert_eq!(Value::Uint(4).text(), "4");
    }

    #[test]
    fn doubles_are_written_as_javascript_writes_them() {
        let cases = [
            (3.0, "3"),
            (12.5, "12.5"),
            (0.1, "0.1"),
            (-0.0, "0"),
            (1e6, "1000000"),
            (123_456_789.0, "123456789"),
            (1e20, "100000000000000000000"),
            (1e21, "1e+21"),
            (1.5e21, "1.5e+21"),
            (1e-6, "0.000001"),
            (1.5e-6, "0.0000015"),
            (1e-7, "1e-7"),
            (1.5e-7, "1.5e-7"),
            (-2.5, "-2.5"),
            (0.1 + 0.2, "0.30000000000000004"),
            (f64::MAX, "1.7976931348623157e+308"),
            (5e-324, "5e-324"),
            (f64::NAN, "NaN"),
            (f64::INFINITY, "Infinity"),
            (f64::NEG_INFINITY, "-Infinity"),
        ];
        for (d, text) in cases {
            assert_eq!(double_text(d), text, "{d:e}");
        }
    }
}
