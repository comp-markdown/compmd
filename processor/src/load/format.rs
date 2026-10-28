//! The formats of data files (spec §8.6), chosen by extension or by `Content-Type`: YAML and JSON as their natural
//! value, CSV and TSV as a list of records keyed by the header row, and a CompMD document as its author's
//! frontmatter keys.

use super::csv::{Row, rows};
use super::{Failure, Violation};
use crate::diagnostic::Location;
use crate::parse::yaml;
use crate::settings::Settings;
use crate::{ParseOptions, parse};
use serde_json::{Map, Value};
use std::collections::HashSet;

/// A data format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Format {
    Json,
    Yaml,
    /// CSV with this delimiter.
    Delimited(char),
    /// A CompMD document, whose frontmatter is the data.
    Compmd,
}

/// A source's data, with where each of its records is when the format tells, and the records already dropped.
#[derive(Debug, Default)]
pub(super) struct Parsed {
    pub(super) value: Value,
    /// The file, and where each record of a list starts in it.
    pub(super) records: Option<(String, Vec<Location>)>,
    /// Records that couldn't be read (F027).
    pub(super) dropped: Vec<Violation>,
    /// Whether its values are text to be read by the schema, as CSV cells are.
    pub(super) text: bool,
}

impl Parsed {
    /// Data given as a value, with no file behind it.
    pub(super) fn value(value: Value) -> Self {
        Self {
            value,
            ..Self::default()
        }
    }
}

impl Format {
    /// The format of a file by its extension; `delimiter` is a CSV import's own.
    pub(super) fn of_path(path: &str, delimiter: Option<char>) -> Option<Self> {
        if path.ends_with(".comp.md") {
            return Some(Self::Compmd);
        }
        match path.rsplit_once('.')?.1 {
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "csv" => Some(Self::Delimited(delimiter.unwrap_or(','))),
            "tsv" => Some(Self::Delimited(delimiter.unwrap_or('\t'))),
            _ => None,
        }
    }

    /// The format of a fetched body by its `Content-Type`.
    pub(super) fn of_content_type(content_type: &str) -> Option<Self> {
        let media = content_type.split(';').next()?.trim().to_ascii_lowercase();
        match media.as_str() {
            "application/json" => Some(Self::Json),
            "application/yaml" | "application/x-yaml" | "text/yaml" => Some(Self::Yaml),
            "text/csv" => Some(Self::Delimited(',')),
            "text/tab-separated-values" => Some(Self::Delimited('\t')),
            "text/markdown" => Some(Self::Compmd),
            _ if media.ends_with("+json") => Some(Self::Json),
            _ => None,
        }
    }

    /// Reads `text`, the file `name`.
    pub(super) fn parse(self, name: &str, text: &str) -> Result<Parsed, Failure> {
        match self {
            Self::Json => serde_json::from_str(text)
                .map(Parsed::value)
                .map_err(|error| Failure {
                    message: format!("`{name}` isn't JSON: {error}"),
                    inside: Some((
                        name.to_string(),
                        Location::new(error.line(), error.column()),
                    )),
                }),
            Self::Yaml => {
                let document = yaml::parse(text).map_err(|yaml::Refused(mark, why)| Failure {
                    message: format!("`{name}` isn't YAML 1.2 core: {why}"),
                    inside: Some((name.to_string(), Location::new(mark.line, mark.column + 1))),
                })?;
                let count = document.value.as_array().map_or(0, Vec::len);
                // A record is at its line, as a CSV record is: the parser marks a block mapping at its first `:`.
                let records = (0..count)
                    .map(|index| {
                        let line = document
                            .values
                            .get(&format!("/{index}"))
                            .map_or(1, |mark| mark.line);
                        Location::new(line, 1)
                    })
                    .collect();
                Ok(Parsed {
                    value: document.value,
                    records: Some((name.to_string(), records)),
                    ..Parsed::default()
                })
            }
            Self::Delimited(delimiter) => table(name, rows(text, delimiter).map_err(Failure::new)?),
            Self::Compmd => {
                let parsed = parse(text, &ParseOptions::default());
                Ok(Parsed::value(Value::Object(Settings::of(&parsed.root).doc)))
            }
        }
    }
}

/// CSV rows as a list of records keyed by the header row, every value a string. A header with an empty or repeated
/// name is F026; a row with more or fewer cells than the header is dropped (F027).
fn table(name: &str, rows: Vec<Row>) -> Result<Parsed, Failure> {
    let mut rows = rows.into_iter();
    let header = rows.next().ok_or_else(|| Failure {
        message: format!("`{name}` has no header row"),
        inside: Some((name.to_string(), Location::new(1, 1))),
    })?;
    let mut seen = HashSet::new();
    if let Some(bad) = header
        .cells
        .iter()
        .find(|cell| cell.is_empty() || !seen.insert(cell.as_str()))
    {
        let why = if bad.is_empty() {
            "an empty name".to_string()
        } else {
            format!("`{bad}` twice")
        };
        return Err(Failure {
            message: format!("the header of `{name}` has {why}"),
            inside: Some((name.to_string(), Location::new(header.line, 1))),
        });
    }
    let mut parsed = Parsed {
        text: true,
        ..Parsed::default()
    };
    let mut records = Vec::new();
    let mut lines = Vec::new();
    for row in rows {
        let at = Some((name.to_string(), Location::new(row.line, 1)));
        if row.cells.len() != header.cells.len() {
            parsed.dropped.push(Violation {
                message: format!(
                    "this record has {} cells and the header {}; it is dropped",
                    row.cells.len(),
                    header.cells.len()
                ),
                at,
            });
            continue;
        }
        let fields: Map<String, Value> = header
            .cells
            .iter()
            .cloned()
            .zip(row.cells.into_iter().map(Value::String))
            .collect();
        records.push(Value::Object(fields));
        lines.push(Location::new(row.line, 1));
    }
    parsed.value = Value::Array(records);
    parsed.records = Some((name.to_string(), lines));
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn formats_come_from_extensions_and_content_types() {
        assert_eq!(Format::of_path("a.comp.md", None), Some(Format::Compmd));
        assert_eq!(
            Format::of_path("a.tsv", None),
            Some(Format::Delimited('\t'))
        );
        assert_eq!(
            Format::of_path("a.csv", Some(';')),
            Some(Format::Delimited(';'))
        );
        assert_eq!(Format::of_path("a.yml", None), Some(Format::Yaml));
        assert_eq!(Format::of_path("a.txt", None), None);
        assert_eq!(Format::of_path("noext", None), None);
        assert_eq!(
            Format::of_content_type("application/json; charset=utf-8"),
            Some(Format::Json)
        );
        assert_eq!(
            Format::of_content_type("application/geo+json"),
            Some(Format::Json)
        );
        assert_eq!(Format::of_content_type("text/yaml"), Some(Format::Yaml));
        assert_eq!(
            Format::of_content_type("text/csv"),
            Some(Format::Delimited(','))
        );
        assert_eq!(
            Format::of_content_type("text/tab-separated-values"),
            Some(Format::Delimited('\t'))
        );
        assert_eq!(
            Format::of_content_type("text/markdown"),
            Some(Format::Compmd)
        );
        assert_eq!(Format::of_content_type("text/html"), None);
    }

    #[test]
    fn broken_files_say_where() {
        let json = Format::Json
            .parse("a.json", "{\n  \"a\": }")
            .expect_err("an error");
        assert_eq!(json.inside.map(|(_, at)| at.line), Some(2));
        let yaml = Format::Yaml
            .parse("a.yaml", "a: &x 1\n")
            .expect_err("an error");
        assert_eq!(yaml.inside.map(|(_, at)| at), Some(Location::new(1, 4)));
    }

    #[test]
    fn yaml_records_keep_their_lines_and_documents_give_their_frontmatter() {
        let parsed = Format::Yaml
            .parse("t.yaml", "- a: 1\n- a: 2\n")
            .expect("YAML");
        assert_eq!(
            parsed.records.map(|(_, lines)| lines),
            Some(vec![Location::new(1, 1), Location::new(2, 1)])
        );
        let site = Format::Compmd
            .parse(
                "s.comp.md",
                "---\ntitle: Site\nmd:\n  version: 1\n---\nText.\n",
            )
            .expect("a document");
        assert_eq!(site.value, json!({ "title": "Site" }));
    }

    #[test]
    fn headers_need_names_once() {
        assert!(Format::Delimited(',').parse("a.csv", "").is_err());
        assert!(Format::Delimited(',').parse("a.csv", "a,\n1,2\n").is_err());
        let twice = Format::Delimited(',')
            .parse("a.csv", "a,a\n")
            .expect_err("an error");
        assert!(twice.message.contains("`a` twice"));
        let short = Format::Delimited(',')
            .parse("a.csv", "a,b\n1\n2,3\n")
            .expect("CSV");
        assert_eq!(short.value, json!([{ "a": "2", "b": "3" }]));
        assert_eq!(short.dropped.len(), 1);
    }
}
