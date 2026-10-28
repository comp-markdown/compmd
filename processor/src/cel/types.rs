//! The profile's types (spec §8.2): the type of a JSON value as frontmatter and data give it, and the type of a
//! declared param. Nested types are shared, so a type is cheap to clone when a scope is extended for each `each`.

use crate::tree::DeclaredType;
use serde_json::Value as Json;
use std::collections::BTreeMap;
use std::fmt;
use std::rc::Rc;

/// The type of an expression or a variable.
#[derive(Clone, Debug, PartialEq)]
pub enum Type {
    /// `int`: integer literals, `size()`, `md.loop` fields, `md.daysBetween`, list indexes.
    Int,
    /// `uint`: unsigned integer literals (`1u`).
    Uint,
    /// `double`: every number from frontmatter, data, params and inputs.
    Double,
    /// `bool`.
    Bool,
    /// `string`, also for dates (`"2026-10-21"`), enums and names.
    String,
    /// The type of `null`.
    Null,
    /// `timestamp`.
    Timestamp,
    /// `duration`.
    Duration,
    /// A string in one of the grammars of spec §9.4, named by its CompMD type (`length`, `color`, …): only `==`,
    /// `!=` and string concatenation take it.
    Grammar(&'static str),
    /// `list(element)`.
    List(Rc<Type>),
    /// `map(key, value)`.
    Map(Rc<Type>, Rc<Type>),
    /// A map from strings whose fields are known: frontmatter, a record of data, `md` itself.
    Record(Rc<BTreeMap<String, Type>>),
    /// Any type, checked when evaluated.
    Dyn,
}

/// The CompMD types whose values are strings in a grammar of spec §9.4.
const GRAMMARS: [&str; 7] = [
    "length", "space", "color", "tracks", "surface", "border", "radius",
];

impl Type {
    /// A list of `element`s.
    #[must_use]
    pub fn list(element: Self) -> Self {
        Self::List(Rc::new(element))
    }

    /// A map from `key` to `value`.
    #[must_use]
    pub fn map(key: Self, value: Self) -> Self {
        Self::Map(Rc::new(key), Rc::new(value))
    }

    /// A record of these fields.
    #[must_use]
    pub fn record(fields: BTreeMap<String, Self>) -> Self {
        Self::Record(Rc::new(fields))
    }

    /// The type of a JSON value: every number is a `double` (spec §8.2), an object a record of its fields, and a
    /// list's element type is what its elements share.
    #[must_use]
    pub fn of_json(value: &Json) -> Self {
        match value {
            Json::Null => Self::Null,
            Json::Bool(_) => Self::Bool,
            Json::Number(_) => Self::Double,
            Json::String(_) => Self::String,
            Json::Array(items) => Self::list(Self::joined(items.iter().map(Self::of_json))),
            Json::Object(fields) => Self::record(
                fields
                    .iter()
                    .map(|(key, value)| (key.clone(), Self::of_json(value)))
                    .collect(),
            ),
        }
    }

    /// The type of a param declared with `declared` (spec §8.2's table): a param is typed by its declaration, not
    /// by its value, so an optional `length?` without a value is still a `length`. A bound input has its input's
    /// type.
    #[must_use]
    pub fn declared(declared: &DeclaredType) -> Self {
        match declared {
            DeclaredType::Enum { .. } => Self::String,
            DeclaredType::Input { input } => Self::declared(input),
            DeclaredType::Name(name) => match name.as_str() {
                "string" | "date" | "name" | "enum" => Self::String,
                "number" => Self::Double,
                "boolean" => Self::Bool,
                "array" => Self::list(Self::Dyn),
                "object" => Self::map(Self::String, Self::Dyn),
                other => GRAMMARS
                    .iter()
                    .find(|grammar| **grammar == other)
                    .map_or(Self::Dyn, |grammar| Self::Grammar(grammar)),
            },
        }
    }

    /// The type that values of all these types have: see [`Type::join`]. `dyn` when there are none.
    pub(crate) fn joined(types: impl IntoIterator<Item = Self>) -> Self {
        types.into_iter().reduce(Self::join).unwrap_or(Self::Dyn)
    }

    /// One type for values of two: the same type; the other one when one is `null` (an absent value); a record of
    /// all the fields of two records, since a field may be missing from some records of data; `dyn` otherwise.
    #[must_use]
    pub fn join(self, other: Self) -> Self {
        match (self, other) {
            (a, b) if a == b => a,
            (Self::Null, other) | (other, Self::Null) => other,
            (Self::Record(a), Self::Record(b)) => {
                let mut fields = (*a).clone();
                for (name, ty) in b.iter() {
                    let joined = match fields.remove(name) {
                        Some(known) => known.join(ty.clone()),
                        None => ty.clone(),
                    };
                    fields.insert(name.clone(), joined);
                }
                Self::record(fields)
            }
            (Self::List(a), Self::List(b)) => Self::list((*a).clone().join((*b).clone())),
            (Self::Map(k, v), Self::Map(l, w)) => Self::map(
                (*k).clone().join((*l).clone()),
                (*v).clone().join((*w).clone()),
            ),
            (Self::Grammar(_), Self::String) | (Self::String, Self::Grammar(_)) => Self::String,
            _ => Self::Dyn,
        }
    }

    /// Whether this is a number type, or `dyn`.
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::Int | Self::Uint | Self::Double | Self::Dyn)
    }

    /// Whether a value of this type can be given where `expected` is asked for: `dyn` fits everywhere and takes
    /// everything; a record fits a map from strings; a string fits a grammar's type, which is checked when the
    /// value is used.
    #[must_use]
    pub fn fits(&self, expected: &Self) -> bool {
        match (self, expected) {
            (Self::Dyn, _) | (_, Self::Dyn) | (Self::String, Self::Grammar(_)) => true,
            (Self::List(a), Self::List(b)) => a.fits(b),
            (Self::Map(k, v), Self::Map(l, w)) => k.fits(l) && v.fits(w),
            (Self::Record(fields), Self::Map(key, value)) => {
                Self::String.fits(key) && fields.values().all(|field| field.fits(value))
            }
            (a, b) => a == b,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => f.write_str("int"),
            Self::Uint => f.write_str("uint"),
            Self::Double => f.write_str("double"),
            Self::Bool => f.write_str("bool"),
            Self::String => f.write_str("string"),
            Self::Null => f.write_str("null"),
            Self::Timestamp => f.write_str("timestamp"),
            Self::Duration => f.write_str("duration"),
            Self::Grammar(name) => f.write_str(name),
            Self::List(element) => write!(f, "list({element})"),
            Self::Map(key, value) => write!(f, "map({key}, {value})"),
            Self::Record(_) => f.write_str("map(string, dyn)"),
            Self::Dyn => f.write_str("dyn"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn record(fields: &[(&str, Type)]) -> Type {
        Type::record(
            fields
                .iter()
                .map(|(k, t)| ((*k).to_string(), t.clone()))
                .collect(),
        )
    }

    #[test]
    fn json_numbers_are_doubles_and_objects_records() {
        assert_eq!(Type::of_json(&json!(3)), Type::Double);
        assert_eq!(Type::of_json(&json!(null)), Type::Null);
        assert_eq!(Type::of_json(&json!(true)), Type::Bool);
        assert_eq!(Type::of_json(&json!("a")), Type::String);
        assert_eq!(Type::of_json(&json!([])), Type::list(Type::Dyn));
        assert_eq!(Type::of_json(&json!([1, null])), Type::list(Type::Double));
        assert_eq!(Type::of_json(&json!([1, "a"])), Type::list(Type::Dyn));
        assert_eq!(
            Type::of_json(&json!({ "a": 1, "b": [true] })),
            record(&[("a", Type::Double), ("b", Type::list(Type::Bool))])
        );
    }

    #[test]
    fn records_of_data_join_all_their_fields() {
        let rows = Type::of_json(&json!([
            { "name": "a", "note": "x" },
            { "name": "b", "amount": 2 },
            { "name": null, "amount": 3 },
        ]));
        assert_eq!(
            rows,
            Type::list(record(&[
                ("amount", Type::Double),
                ("name", Type::String),
                ("note", Type::String),
            ]))
        );
        assert_eq!(
            Type::of_json(&json!([{ "a": 1 }, { "a": "x" }])),
            Type::list(record(&[("a", Type::Dyn)]))
        );
        assert_eq!(
            Type::map(Type::String, Type::Int).join(Type::map(Type::String, Type::Double)),
            Type::map(Type::String, Type::Dyn)
        );
        assert_eq!(
            Type::of_json(&json!([[1], ["a"]])),
            Type::list(Type::list(Type::Dyn))
        );
        assert_eq!(Type::Grammar("length").join(Type::String), Type::String);
        assert_eq!(Type::String.join(Type::Grammar("color")), Type::String);
        assert_eq!(
            Type::Grammar("length").join(Type::Grammar("color")),
            Type::Dyn
        );
    }

    #[test]
    fn declarations_give_types() {
        let name = |n: &str| Type::declared(&DeclaredType::Name(n.into()));
        assert_eq!(name("string"), Type::String);
        assert_eq!(name("date"), Type::String);
        assert_eq!(name("name"), Type::String);
        assert_eq!(name("number"), Type::Double);
        assert_eq!(name("boolean"), Type::Bool);
        assert_eq!(name("array"), Type::list(Type::Dyn));
        assert_eq!(name("object"), Type::map(Type::String, Type::Dyn));
        assert_eq!(name("length"), Type::Grammar("length"));
        assert_eq!(name("radius").to_string(), "radius");
        assert_eq!(name("nonsense"), Type::Dyn);
        assert_eq!(
            Type::declared(&DeclaredType::Enum {
                members: vec!["a".into()]
            }),
            Type::String
        );
        let input = |inner: &str| DeclaredType::Input {
            input: Box::new(DeclaredType::Name(inner.into())),
        };
        assert_eq!(Type::declared(&input("number")), Type::Double);
        assert_eq!(Type::declared(&input("enum")), Type::String);
    }

    #[test]
    fn types_fit_where_values_can_go() {
        assert!(Type::Int.fits(&Type::Int));
        assert!(!Type::Int.fits(&Type::Double));
        assert!(Type::Dyn.fits(&Type::Int) && Type::Int.fits(&Type::Dyn));
        assert!(Type::String.fits(&Type::Grammar("length")));
        assert!(!Type::Grammar("length").fits(&Type::String));
        assert!(Type::list(Type::Int).fits(&Type::list(Type::Dyn)));
        assert!(!Type::list(Type::Int).fits(&Type::list(Type::String)));
        assert!(Type::map(Type::String, Type::Int).fits(&Type::map(Type::String, Type::Dyn)));
        assert!(record(&[("a", Type::Int)]).fits(&Type::map(Type::String, Type::Dyn)));
        assert!(!record(&[("a", Type::Int)]).fits(&Type::map(Type::String, Type::String)));
        assert!(!record(&[]).fits(&Type::map(Type::Int, Type::Dyn)));
        assert!(!Type::Null.fits(&Type::Int));
    }

    #[test]
    fn types_are_written_as_cel_writes_them() {
        let written: Vec<String> = [
            Type::Int,
            Type::Uint,
            Type::Double,
            Type::Bool,
            Type::String,
            Type::Null,
            Type::Timestamp,
            Type::Duration,
            Type::list(Type::Int),
            Type::map(Type::String, Type::Dyn),
            record(&[]),
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        assert_eq!(
            written,
            [
                "int",
                "uint",
                "double",
                "bool",
                "string",
                "null",
                "timestamp",
                "duration",
                "list(int)",
                "map(string, dyn)",
                "map(string, dyn)"
            ]
        );
    }
}
