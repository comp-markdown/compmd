//! A data import's `schema` (spec §8.6): each record of a list read field by field, as its declared type, under its
//! declared name. A record that violates it is dropped (F027).

use super::Violation;
use super::format::Parsed;
use crate::cel::time::parse_date;
use crate::settings::Field;
use crate::tree::{DeclaredType, TypeName};
use serde_json::{Map, Number, Value};

/// The data typed by `fields`, and the records dropped, with what was wrong.
pub(super) fn apply(parsed: Parsed, fields: &[Field]) -> (Value, Vec<Violation>) {
    let Parsed {
        value,
        records,
        mut dropped,
        text,
    } = parsed;
    let Value::Array(items) = value else {
        return (value, dropped);
    };
    let at = |index: usize| {
        records
            .as_ref()
            .and_then(|(file, lines)| Some((file.clone(), *lines.get(index)?)))
    };
    let mut typed = Vec::new();
    for (index, item) in items.into_iter().enumerate() {
        match record(item, fields, text) {
            Ok(record) => typed.push(Value::Object(record)),
            Err(why) => dropped.push(Violation {
                message: format!("record {}: {why}; it is dropped", index + 1),
                at: at(index),
            }),
        }
    }
    (Value::Array(typed), dropped)
}

/// One record: its declared fields typed and renamed, its other keys kept.
fn record(item: Value, fields: &[Field], text: bool) -> Result<Map<String, Value>, String> {
    let Value::Object(mut record) = item else {
        return Err("it isn't a record".into());
    };
    let mut typed = Map::new();
    for field in fields {
        let declaration = &field.declaration;
        let raw = record.remove(&field.key).unwrap_or(Value::Null);
        let value = read(raw, &declaration.declared, declaration.optional, text)
            .map_err(|why| format!("`{}` {why}", field.key))?;
        typed.insert(declaration.name.clone(), value);
    }
    typed.extend(record);
    Ok(typed)
}

/// A value read as `declared`: from text for a CSV cell, where an empty cell is no value.
pub(super) fn read(
    raw: Value,
    declared: &DeclaredType,
    optional: bool,
    text: bool,
) -> Result<Value, String> {
    let empty = raw.is_null() || (text && raw.as_str() == Some(""));
    if empty {
        return if optional {
            Ok(Value::Null)
        } else {
            Err("has no value".into())
        };
    }
    let fits = match declared {
        DeclaredType::Enum { members } => raw
            .as_str()
            .is_some_and(|value| members.iter().any(|member| member == value)),
        DeclaredType::Name(name) => match (name, &raw) {
            (TypeName::Number, Value::String(cell)) if text => {
                return cell
                    .parse::<f64>()
                    .ok()
                    .and_then(Number::from_f64)
                    .map(Value::Number)
                    .ok_or_else(|| format!("isn't a number: `{cell}`"));
            }
            (TypeName::Boolean, Value::String(cell)) if text => {
                return match cell.as_str() {
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    _ => Err(format!("isn't `true` or `false`: `{cell}`")),
                };
            }
            (TypeName::Number, value) => value.is_number(),
            (TypeName::Boolean, value) => value.is_boolean(),
            (TypeName::Date, Value::String(day)) => parse_date(day).is_some(),
            (TypeName::Array, value) => value.is_array(),
            (TypeName::Object, value) => value.is_object(),
            (_, value) => value.is_string(),
        },
        DeclaredType::Other(_) => true,
        DeclaredType::Input { .. } => false,
    };
    if fits {
        Ok(raw)
    } else {
        Err(format!("isn't {}: `{raw}`", declared.written()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Location;
    use crate::parse::definitions::declaration;
    use serde_json::json;

    fn field(name: &str, written: &Value) -> Field {
        Field {
            declaration: declaration(name, written),
            key: written
                .get("from")
                .and_then(Value::as_str)
                .unwrap_or(name)
                .to_string(),
        }
    }

    fn csv(value: Value) -> Parsed {
        Parsed {
            value,
            records: Some((
                "t.csv".into(),
                vec![Location::new(2, 1), Location::new(3, 1)],
            )),
            dropped: Vec::new(),
            text: true,
        }
    }

    #[test]
    fn cells_are_read_by_their_type_and_renamed() {
        let fields = [
            field("count", &json!("number")),
            field("day", &json!({ "type": "date", "from": "Day" })),
            field("note", &json!("string?")),
            field("on", &json!("boolean")),
        ];
        let rows = json!([
            { "count": "410", "Day": "2026-10-20", "note": "", "on": "true", "extra": "x" },
            { "count": "x", "Day": "2026-10-20", "on": "false" },
        ]);
        let (value, dropped) = apply(csv(rows), &fields);
        assert_eq!(
            value,
            json!([{ "count": 410.0, "day": "2026-10-20", "note": null, "on": true, "extra": "x" }])
        );
        assert_eq!(dropped.len(), 1);
        assert_eq!(dropped[0].at, Some(("t.csv".into(), Location::new(3, 1))));
    }

    #[test]
    fn values_of_json_keep_their_types() {
        let tone = DeclaredType::Enum {
            members: vec!["a".into()],
        };
        assert_eq!(read(json!("a"), &tone, false, false), Ok(json!("a")));
        assert!(read(json!("b"), &tone, false, false).is_err());
        let name = DeclaredType::named;
        assert_eq!(
            read(json!(""), &name("string"), false, false),
            Ok(json!(""))
        );
        assert!(read(json!("3"), &name("number"), false, false).is_err());
        assert!(read(json!(true), &name("boolean"), false, false).is_ok());
        assert!(read(json!("maybe"), &name("boolean"), false, true).is_err());
        assert!(read(json!([1]), &name("array"), false, false).is_ok());
        assert!(read(json!({}), &name("object"), false, false).is_ok());
        assert!(read(json!(1), &name("dyn"), false, false).is_ok());
        assert!(read(json!(1), &name("color"), false, false).is_err());
        assert!(read(json!("2026-02-30"), &name("date"), false, false).is_err());
        assert!(read(Value::Null, &name("string"), false, false).is_err());
        let input = DeclaredType::Input {
            input: Box::new(name("date")),
        };
        assert!(read(json!("x"), &input, false, false).is_err());
    }

    #[test]
    fn data_that_isnt_a_list_of_records_stays_as_it_is() {
        let fields = [field("a", &json!("string"))];
        let (value, dropped) = apply(Parsed::value(json!({ "a": 1 })), &fields);
        assert_eq!((value, dropped.len()), (json!({ "a": 1 }), 0));
        let (value, dropped) = apply(Parsed::value(json!([1])), &fields);
        assert_eq!((value, dropped[0].at.clone()), (json!([]), None));
    }
}
