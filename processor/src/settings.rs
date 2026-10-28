//! A document's settings, the keys under `md` (spec §2.1), read with where each key and value is written, for the
//! diagnostics of stages 2 and 3. Entries that don't have the shape the spec gives them are left out.

use crate::diagnostic::Location;
use crate::parse::definitions::declaration;
use crate::parse::yaml::{self, Mark};
use crate::tree::{Declaration, Node, Root};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// A value and where it is written.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Located<T> {
    pub(crate) value: T,
    pub(crate) at: Location,
}

/// The names an import takes or an export gives (spec §6.1, §6.2).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Names {
    /// `"*"`, written at this place.
    All(Location),
    /// Listed names.
    Listed(Vec<NameEntry>),
}

/// One listed name: `card`, or `{name: card, as: promo}`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NameEntry {
    /// The name in the source: the exported name of an import, the local name of an export.
    pub(crate) name: String,
    /// The name it gets: the local name of an import, the public name of an export.
    pub(crate) renamed: String,
    /// Where the entry is.
    pub(crate) at: Location,
}

/// A component import (spec §6.1).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Import {
    pub(crate) from: Located<String>,
    pub(crate) integrity: Option<Located<String>>,
    pub(crate) names: Names,
}

/// The document's exports (spec §6.2).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Export {
    pub(crate) pack: Option<String>,
    pub(crate) version: Option<String>,
    /// Where the `export` key is.
    pub(crate) at: Location,
    pub(crate) names: Names,
}

/// A data import (spec §8.6).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct DataImport {
    /// `from` as written.
    pub(crate) from: Located<String>,
    /// Where the text of `from` starts, past a quote.
    pub(crate) text_at: Location,
    /// The name it gets under `md.data`.
    pub(crate) name: String,
    /// The declared fields of its records.
    pub(crate) schema: Option<Vec<Field>>,
    /// The local file loaded when a remote or `host:` source fails.
    pub(crate) fallback: Option<String>,
    /// A CSV file's field separator, when it is one ASCII character.
    pub(crate) delimiter: Option<u8>,
}

/// A field of a data import's schema.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Field {
    /// Its name and type.
    pub(crate) declaration: Declaration,
    /// The key it has in the data: its `from`, or its name.
    pub(crate) key: String,
}

/// A declared input (spec §8.7).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Input {
    pub(crate) declaration: Declaration,
    /// Where its name is.
    pub(crate) at: Location,
}

/// The settings of a document.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Settings {
    /// The author's keys, what `md.doc` holds.
    pub(crate) doc: Map<String, Value>,
    pub(crate) imports: Vec<Import>,
    pub(crate) export: Option<Export>,
    pub(crate) data: Vec<DataImport>,
    pub(crate) inputs: Vec<Input>,
    /// `md.version` as written, `1` or `1.2`.
    pub(crate) version: Option<String>,
    /// `md.lang`, a BCP 47 tag.
    pub(crate) lang: Option<String>,
    /// `md.tz`, a time zone.
    pub(crate) tz: Option<String>,
}

impl Settings {
    /// The settings of the document `root` is the tree of: none without frontmatter, or when CompMD processing
    /// doesn't apply to it.
    pub(crate) fn of(root: &Root) -> Self {
        let read = match root.children.first() {
            Some(Node::Yaml(frontmatter)) if frontmatter.parsed.is_some() => {
                yaml::parse(&frontmatter.value).ok().zip(Some(frontmatter))
            }
            _ => None,
        };
        let Some((document, frontmatter)) = read else {
            return Self::default();
        };
        let reader = Reader {
            lines: frontmatter.value.split('\n').collect(),
            keys: document.keys,
            values: document.values,
        };
        let Value::Object(mut doc) = document.value else {
            return Self::default();
        };
        let md = doc.remove("md").unwrap_or(Value::Null);
        let text = |key: &str| match md.get(key) {
            Some(Value::String(text)) => Some(text.clone()),
            Some(Value::Number(number)) => Some(number.to_string()),
            _ => None,
        };
        Self {
            imports: reader.list(&md, "import", Reader::import),
            export: md.get("export").and_then(|export| reader.export(export)),
            data: reader.list(&md, "data", Reader::data),
            inputs: reader.inputs(&md),
            version: text("version"),
            lang: text("lang"),
            tz: text("tz"),
            doc,
        }
    }
}

/// Reads entries with the places of their keys and values, by JSON pointer.
struct Reader<'a> {
    lines: Vec<&'a str>,
    keys: HashMap<String, Mark>,
    values: HashMap<String, Mark>,
}

impl Reader<'_> {
    /// Where a mark is in the document: the YAML starts on line 2.
    fn location(mark: Mark) -> Location {
        Location::new(1 + mark.line, mark.column + 1)
    }

    fn key_at(&self, pointer: &str) -> Location {
        self.keys
            .get(pointer)
            .copied()
            .map_or(Location::new(1, 1), Self::location)
    }

    /// Where the value at `pointer` starts, at its quote if it is quoted.
    fn value_at(&self, pointer: &str) -> Location {
        self.values
            .get(pointer)
            .copied()
            .map_or(Location::new(1, 1), Self::location)
    }

    /// Where the text of the string at `pointer` starts: past its quote, if it is quoted.
    fn content_at(&self, pointer: &str) -> Location {
        let at = self.value_at(pointer);
        let quoted = self
            .lines
            .get(at.line.saturating_sub(2))
            .and_then(|line| line.chars().nth(at.column - 1))
            .is_some_and(|c| c == '"' || c == '\'');
        Location::new(at.line, at.column + usize::from(quoted))
    }

    /// The text of `entry`'s `key`, where `pointer` points at `entry`.
    fn text(&self, entry: &Value, pointer: &str, key: &str) -> Option<Located<String>> {
        Some(Located {
            value: entry.get(key)?.as_str()?.to_string(),
            at: self.value_at(&format!("{pointer}/{key}")),
        })
    }

    /// Each entry of the list `md.<key>` that `read` takes, with its pointer.
    fn list<T>(&self, md: &Value, key: &str, read: fn(&Self, &Value, &str) -> Option<T>) -> Vec<T> {
        md.get(key)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
            .filter_map(|(i, entry)| read(self, entry, &format!("/md/{key}/{i}")))
            .collect()
    }

    fn import(&self, entry: &Value, pointer: &str) -> Option<Import> {
        Some(Import {
            from: self.text(entry, pointer, "from")?,
            integrity: self.text(entry, pointer, "integrity"),
            names: self.names(entry.get("names")?, &format!("{pointer}/names"))?,
        })
    }

    fn export(&self, export: &Value) -> Option<Export> {
        let text = |key: &str| export.get(key).and_then(Value::as_str).map(str::to_string);
        Some(Export {
            pack: text("pack"),
            version: text("version"),
            at: self.key_at("/md/export"),
            names: self.names(export.get("names")?, "/md/export/names")?,
        })
    }

    /// `"*"`, or a list of names and `{name, as}` mappings.
    fn names(&self, names: &Value, pointer: &str) -> Option<Names> {
        match names {
            Value::String(all) if all == "*" => Some(Names::All(self.value_at(pointer))),
            Value::Array(entries) => Some(Names::Listed(
                entries
                    .iter()
                    .enumerate()
                    .filter_map(|(i, entry)| self.name_entry(entry, &format!("{pointer}/{i}")))
                    .collect(),
            )),
            _ => None,
        }
    }

    fn name_entry(&self, entry: &Value, pointer: &str) -> Option<NameEntry> {
        if let Some(name) = entry.as_str() {
            return Some(NameEntry {
                name: name.to_string(),
                renamed: name.to_string(),
                at: self.value_at(pointer),
            });
        }
        let name = entry.get("name")?.as_str()?;
        let renamed = entry.get("as").and_then(Value::as_str).unwrap_or(name);
        Some(NameEntry {
            name: name.to_string(),
            renamed: renamed.to_string(),
            at: self.value_at(&format!("{pointer}/name")),
        })
    }

    fn data(&self, entry: &Value, pointer: &str) -> Option<DataImport> {
        let schema = entry
            .get("schema")
            .and_then(Value::as_object)
            .map(|fields| {
                fields
                    .iter()
                    .map(|(name, declared)| Field {
                        declaration: declaration(name, declared),
                        key: declared
                            .get("from")
                            .and_then(Value::as_str)
                            .unwrap_or(name)
                            .to_string(),
                    })
                    .collect()
            });
        let text = |key: &str| entry.get(key).and_then(Value::as_str);
        Some(DataImport {
            from: self.text(entry, pointer, "from")?,
            text_at: self.content_at(&format!("{pointer}/from")),
            name: text("as")?.to_string(),
            schema,
            fallback: text("fallback").map(str::to_string),
            delimiter: text("delimiter")
                .and_then(|delimiter| <[u8; 1]>::try_from(delimiter.as_bytes()).ok())
                .map(|[byte]| byte),
        })
    }

    fn inputs(&self, md: &Value) -> Vec<Input> {
        md.get("inputs")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .map(|(name, declared)| Input {
                declaration: declaration(name, declared),
                at: self.key_at(&format!("/md/inputs/{name}")),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ParseOptions, parse};

    fn settings(text: &str) -> Settings {
        Settings::of(&parse(text, &ParseOptions::default()).root)
    }

    #[test]
    fn reads_imports_with_their_places() {
        let read = settings(
            "---\nmd:\n  import:\n    - from: \"@acme/kit@1\"\n      integrity: sha512-A\n      names: [chip, {name: tag, as: label}]\n    - from: ./a.comp.md\n      names: \"*\"\n    - names: [x]\n    - from: ./b.comp.md\n      names: 3\n---\n",
        );
        assert_eq!(read.imports.len(), 2);
        let kit = &read.imports[0];
        assert_eq!(kit.from.value, "@acme/kit@1");
        assert_eq!(kit.from.at, Location::new(4, 13));
        assert_eq!(
            kit.integrity.as_ref().map(|i| i.at),
            Some(Location::new(5, 18))
        );
        let entry = |name: &str, renamed: &str, at| NameEntry {
            name: name.into(),
            renamed: renamed.into(),
            at,
        };
        assert_eq!(
            kit.names,
            Names::Listed(vec![
                entry("chip", "chip", Location::new(6, 15)),
                entry("tag", "label", Location::new(6, 28)),
            ])
        );
        assert_eq!(read.imports[1].names, Names::All(Location::new(8, 14)));
    }

    #[test]
    fn reads_exports_data_inputs_and_the_authors_keys() {
        let read = settings(
            "---\ntitle: T\nmd:\n  export:\n    pack: \"@a/b\"\n    names: \"*\"\n  data:\n    - from: \"./x-{{md.input.y}}.csv\"\n      as: x\n      schema: {n: number}\n    - from: ./y.csv\n  inputs:\n    y: string = \"\"\n---\n",
        );
        assert_eq!(read.doc.keys().collect::<Vec<_>>(), ["title"]);
        let export = read.export.expect("an export");
        assert_eq!(
            (export.pack.as_deref(), export.at),
            (Some("@a/b"), Location::new(4, 3))
        );
        assert_eq!(read.data.len(), 1);
        assert_eq!(read.data[0].from.value, "./x-{{md.input.y}}.csv");
        assert_eq!(
            (read.data[0].from.at, read.data[0].text_at),
            (Location::new(8, 13), Location::new(8, 14))
        );
        assert_eq!(read.data[0].schema.as_ref().map(Vec::len), Some(1));
        assert_eq!(read.inputs[0].at, Location::new(13, 5));
    }

    #[test]
    fn a_document_without_settings_has_none() {
        assert_eq!(settings("text"), Settings::default());
        assert_eq!(
            settings("---\nmd:\n  version: 2\n---\n"),
            Settings::default()
        );
        assert_eq!(settings("---\n- a\n---\n"), Settings::default());
    }
}
