//! TOML frontmatter (`+++`), as JSON (spec §2.1): read into the same document as YAML, so a document reads the same
//! whichever it uses. Dates and times are their text as TOML writes it, integers and floats are numbers (infinities
//! and NaN, with no JSON form, their text), tables are mappings in the order they are written, and arrays of tables
//! lists of mappings. Keys and values keep their places.

use super::yaml::{Document, Mark, Refused};
use ::toml::Spanned;
use ::toml::de::{DeTable, DeValue};
use serde_json::{Map, Number, Value};

/// Parses `text`.
///
/// # Errors
///
/// Where the text isn't TOML, a repeated key among the ways it can't be.
pub(crate) fn parse(text: &str) -> Result<Document, Refused> {
    let table = DeTable::parse(text).map_err(|error| {
        let at = error.span().map_or(0, |span| span.start);
        let why = if error.message().contains("duplicate") {
            "a repeated key"
        } else {
            "invalid TOML"
        };
        Refused(mark(text, at), why)
    })?;
    let mut builder = Builder {
        text,
        document: Document::default(),
    };
    builder.document.value = builder.table(table.get_ref(), "");
    Ok(builder.document)
}

/// Where byte `at` of `text` is: its line from 1, its column from 0 in code points, as YAML's marks are.
fn mark(text: &str, at: usize) -> Mark {
    let before = &text[..at.min(text.len())];
    let line_start = before.rfind('\n').map_or(0, |newline| newline + 1);
    Mark {
        line: 1 + before.matches('\n').count(),
        column: before[line_start..].chars().count(),
    }
}

struct Builder<'t> {
    text: &'t str,
    document: Document,
}

impl Builder<'_> {
    /// `table` as a mapping at `pointer`, its entries in the order they are written.
    fn table(&mut self, table: &DeTable<'_>, pointer: &str) -> Value {
        let mut entries: Vec<_> = table.iter().collect();
        entries.sort_by_key(|(key, _)| key.span().start);
        let mut map = Map::new();
        for (key, value) in entries {
            let name = key.get_ref().to_string();
            let at = format!("{pointer}/{name}");
            self.document
                .keys
                .insert(at.clone(), mark(self.text, key.span().start));
            let value = self.value(value, &at);
            map.insert(name, value);
        }
        Value::Object(map)
    }

    /// `value` at `pointer`, with where it starts.
    fn value(&mut self, value: &Spanned<DeValue<'_>>, pointer: &str) -> Value {
        self.document
            .values
            .insert(pointer.to_string(), mark(self.text, value.span().start));
        match value.get_ref() {
            DeValue::String(text) => Value::String(text.to_string()),
            DeValue::Integer(integer) => i64::from_str_radix(integer.as_str(), integer.radix())
                .map_or_else(|_| Value::String(integer.to_string()), Value::from),
            DeValue::Float(float) => float
                .as_str()
                .parse()
                .ok()
                .and_then(Number::from_f64)
                .map_or_else(|| Value::String(float.to_string()), Value::Number),
            DeValue::Boolean(boolean) => Value::Bool(*boolean),
            DeValue::Datetime(datetime) => Value::String(datetime.to_string()),
            DeValue::Array(items) => Value::Array(
                items
                    .iter()
                    .enumerate()
                    .map(|(i, item)| self.value(item, &format!("{pointer}/{i}")))
                    .collect(),
            ),
            DeValue::Table(table) => self.table(table, pointer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn value(text: &str) -> Value {
        parse(text).expect("valid TOML").value
    }

    fn refused(text: &str) -> Refused {
        parse(text).expect_err("refused")
    }

    #[test]
    fn reads_values_as_yaml_does() {
        assert_eq!(
            value(
                "a = \"no\"\nb = true\nc = 2026-10-21\nd = 1.10\ne = 0x1f\nf = -3\ng = 'x'\ni = inf\nj = 1e3\nk = +2\nl = 1_000\nm = 1979-05-27T07:32:00Z\nn = 07:32:00\no = nan"
            ),
            json!({
                "a": "no", "b": true, "c": "2026-10-21", "d": 1.1, "e": 31, "f": -3, "g": "x",
                "i": "inf", "j": 1000.0, "k": 2, "l": 1000, "m": "1979-05-27T07:32:00Z", "n": "07:32:00",
                "o": "nan",
            })
        );
    }

    #[test]
    fn tables_and_arrays_of_tables_are_mappings_and_lists() {
        assert_eq!(
            value(
                "title = \"x\"\n[md]\nversion = 1\n[[md.data]]\nfrom = \"./a.csv\"\nas = \"a\"\n[[md.data]]\nfrom = \"./b.csv\"\n"
            ),
            json!({ "title": "x", "md": { "version": 1, "data": [
                { "from": "./a.csv", "as": "a" },
                { "from": "./b.csv" },
            ] } })
        );
    }

    #[test]
    fn keys_keep_the_order_they_are_written_in() {
        let keys: Vec<String> = match value("zebra = 1\napple = 2\nmango = 3") {
            Value::Object(map) => map.keys().cloned().collect(),
            _ => Vec::new(),
        };
        assert_eq!(keys, ["zebra", "apple", "mango"]);
    }

    #[test]
    fn keys_and_values_keep_their_places() {
        let document =
            parse("title = \"x\"\n[md]\n  version = 1\n  syntax = [\"wiki-links\"]").unwrap();
        assert_eq!(document.keys["/md/version"], Mark { line: 3, column: 2 });
        assert_eq!(
            document.values["/md/version"],
            Mark {
                line: 3,
                column: 12
            }
        );
        assert_eq!(
            document.values["/md/syntax/0"],
            Mark {
                line: 4,
                column: 12
            }
        );
        assert_eq!(document.values["/title"], Mark { line: 1, column: 8 });
    }

    #[test]
    fn refuses_what_isnt_toml_with_where() {
        assert_eq!(refused("a = 1\na = 2").1, "a repeated key");
        let Refused(at, why) = refused("a = 1\nb = = 2");
        assert_eq!((at.line, why), (2, "invalid TOML"));
    }

    #[test]
    fn places_count_code_points() {
        assert_eq!(mark("é = 1\nab", 8), Mark { line: 2, column: 1 });
        assert_eq!(mark("", 5), Mark { line: 1, column: 0 });
    }
}
