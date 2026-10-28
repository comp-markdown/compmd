//! A document's frontmatter (spec §2.1, §2.2), read before the document is parsed: YAML 1.2 core without tags,
//! anchors, aliases or repeated keys (F033), a major version this processor implements (F001), and CompMD's keys
//! under `md` (W110, W111).

use super::params::string_parts;
use super::yaml::{self, Mark, Refused};
use crate::diagnostic::{Code, Diagnostic, Location};
use crate::nearest::nearest;
use serde_json::{Map, Value};

/// The version of the spec this processor implements.
const MAJOR: u64 = 1;
const MINOR: u64 = 0;

/// The keys under `md` (spec §2.1).
const MD_KEYS: [&str; 9] = [
    "version", "import", "export", "data", "inputs", "lang", "dir", "tz", "syntax",
];

/// The opt-in syntax this processor knows (spec §2.3).
const SYNTAX_NAMES: [&str; 1] = ["wiki-links"];

/// The YAML text of the frontmatter at the start of `text`: a `---` line, the YAML, and a closing `---` line, as
/// the fork of markdown-rs finds it. The YAML starts on line 2.
pub(crate) fn find(text: &str) -> Option<&str> {
    let mut lines = lines(text);
    let (_, opener) = lines.next()?;
    if !is_fence(opener) {
        return None;
    }
    let yaml_start = lines.clone().next().map_or(text.len(), |(start, _)| start);
    let mut yaml_end = yaml_start;
    for (start, line) in lines {
        if is_fence(line) {
            return Some(&text[yaml_start..yaml_end.max(yaml_start)]);
        }
        yaml_end = start + line.len();
    }
    None
}

fn is_fence(line: &str) -> bool {
    line.strip_prefix("---")
        .is_some_and(|rest| rest.bytes().all(|b| b == b' ' || b == b'\t'))
}

/// Each line of `text` with the byte index where it starts, without its line ending.
fn lines(text: &str) -> impl Iterator<Item = (usize, &str)> + Clone {
    let mut start = 0;
    std::iter::from_fn(move || {
        if start >= text.len() {
            return None;
        }
        let rest = &text[start..];
        let end = rest.find(['\n', '\r']).unwrap_or(rest.len());
        let ending = match &rest.as_bytes()[end..] {
            [b'\r', b'\n', ..] => 2,
            [] => 0,
            _ => 1,
        };
        let line = (start, &rest[..end]);
        start += end + ending;
        Some(line)
    })
}

/// Reads the frontmatter's YAML, with where each key and value is; `None` when CompMD processing doesn't apply
/// (F001, F033).
pub(crate) fn read(yaml_text: &str, diagnostics: &mut Vec<Diagnostic>) -> Option<yaml::Document> {
    let at = |mark: Mark| Location::new(1 + mark.line, mark.column + 1);
    let document = match yaml::parse(yaml_text) {
        Ok(document) => document,
        Err(Refused(mark, why)) => {
            diagnostics.push(Diagnostic::new(
                Code::F033,
                at(mark),
                format!("the frontmatter isn't YAML 1.2 core: {why}; the document is read as plain Markdown"),
            ));
            return None;
        }
    };
    let settings = document
        .value
        .get("md")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let version = settings.get("version");
    let Some((MAJOR, minor)) = version.map_or(Some((MAJOR, MINOR)), parse_version) else {
        let written = version.map_or_else(String::new, Value::to_string);
        diagnostics.push(Diagnostic::new(
            Code::F001,
            Location::new(1, 1),
            format!("unsupported version {written}: this processor implements CompMD {MAJOR}; the document is read as plain Markdown"),
        ));
        return None;
    };
    if minor > MINOR {
        diagnostics.push(Diagnostic::new(
            Code::W110,
            at(document.values["/md/version"]),
            format!("the document declares version {MAJOR}.{minor}; this processor implements {MAJOR}.{MINOR}"),
        ));
    }
    for key in settings
        .keys()
        .filter(|key| !MD_KEYS.contains(&key.as_str()))
    {
        let hint = nearest(key, MD_KEYS)
            .map_or_else(String::new, |known| format!("; did you mean `{known}`?"));
        diagnostics.push(Diagnostic::new(
            Code::W111,
            at(document.keys[&format!("/md/{key}")]),
            format!("unknown key `md.{key}`, ignored{hint}"),
        ));
    }
    if let Some(Value::Array(names)) = settings.get("syntax") {
        for (i, name) in names.iter().enumerate() {
            if !name
                .as_str()
                .is_some_and(|name| SYNTAX_NAMES.contains(&name))
            {
                diagnostics.push(Diagnostic::new(
                    Code::W111,
                    at(document.values[&format!("/md/syntax/{i}")]),
                    format!("unknown syntax {name} in `md.syntax`, ignored; known: `wiki-links`"),
                ));
            }
        }
    }
    Some(document)
}

/// The frontmatter as the tree writes it (spec §12.2): a mapping, each data import's `from` as parts.
pub(crate) fn tree_value(document: &yaml::Document) -> Value {
    let mut value = match &document.value {
        Value::Object(entries) => Value::Object(entries.clone()),
        _ => Value::Object(Map::new()),
    };
    if let Some(Value::Array(entries)) = value.pointer_mut("/md/data") {
        for entry in entries {
            if let Some(from) = entry.get("from").and_then(Value::as_str) {
                let parts = serde_json::to_value(string_parts(from)).expect("parts serialize");
                entry["from"] = parts;
            }
        }
    }
    value
}

/// `md.version`: `1`, `1.2` or `"1.2"`, as its major and minor version.
fn parse_version(value: &Value) -> Option<(u64, u64)> {
    let text = match value {
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        _ => return None,
    };
    let (major, minor) = text.split_once('.').unwrap_or((&text, "0"));
    Some((major.parse().ok()?, minor.parse().ok()?))
}

/// Whether settings (a frontmatter, or a host's metadata) opt in to wiki links (spec §2.3).
pub(crate) fn wiki_links(settings: &Value) -> bool {
    settings
        .pointer("/md/syntax")
        .and_then(Value::as_array)
        .is_some_and(|names| names.iter().any(|name| name == "wiki-links"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn read_all(yaml: &str) -> (Option<Value>, Vec<(Code, Location)>) {
        let mut diagnostics = Vec::new();
        let parsed = read(yaml, &mut diagnostics).map(|document| tree_value(&document));
        let found = diagnostics
            .iter()
            .map(|d| (d.code, d.range.start))
            .collect();
        (parsed, found)
    }

    #[test]
    fn finds_frontmatter_between_fences() {
        assert_eq!(find("---\na: 1\nb: 2\n---\ntext"), Some("a: 1\nb: 2"));
        assert_eq!(find("--- \t\r\na: 1\r\n---  "), Some("a: 1"));
        assert_eq!(find("---\n---\n"), Some(""));
        assert_eq!(find("---\na: 1"), None);
        assert_eq!(find("----\n---"), None);
        assert_eq!(find(" ---\n---"), None);
        assert_eq!(find(""), None);
    }

    #[test]
    fn version_1_and_later_minors() {
        assert_eq!(
            read_all("md:\n  version: 1"),
            (Some(json!({ "md": { "version": 1 } })), Vec::new())
        );
        assert_eq!(read_all("title: x").1, []);
        assert_eq!(
            read_all("md:\n  version: \"1.2\"").1,
            [(Code::W110, Location::new(3, 12))]
        );
    }

    #[test]
    fn other_majors_make_the_document_plain() {
        for version in ["2", "\"x\"", "[1]", "\"1.x\""] {
            assert_eq!(
                read_all(&format!("md:\n  version: {version}")),
                (None, vec![(Code::F001, Location::new(1, 1))]),
                "{version}"
            );
        }
    }

    #[test]
    fn refused_yaml_makes_the_document_plain() {
        assert_eq!(
            read_all("a: !!str 1"),
            (None, vec![(Code::F033, Location::new(2, 4))])
        );
    }

    #[test]
    fn unknown_keys_and_syntax_are_w111() {
        assert_eq!(
            read_all("md:\n  input: {}\n  syntax: [wiki-links, tables]").1,
            [
                (Code::W111, Location::new(3, 3)),
                (Code::W111, Location::new(4, 24))
            ]
        );
    }

    #[test]
    fn data_imports_split_from_into_parts() {
        let (parsed, _) =
            read_all("md:\n  data:\n    - from: ./a-{{md.input.year}}.csv\n    - name: x");
        assert_eq!(
            parsed.unwrap()["md"]["data"],
            json!([
                { "from": [
                    { "type": "text", "value": "./a-" },
                    { "type": "interpolation", "path": "md.input.year", "block": false },
                    { "type": "text", "value": ".csv" },
                ] },
                { "name": "x" },
            ])
        );
    }

    #[test]
    fn frontmatter_that_is_not_a_mapping_has_no_keys() {
        assert_eq!(read_all("just text").0, Some(json!({})));
    }

    #[test]
    fn wiki_links_are_opted_in_by_md_syntax() {
        assert!(wiki_links(&json!({ "md": { "syntax": ["wiki-links"] } })));
        assert!(!wiki_links(&json!({ "md": { "syntax": "wiki-links" } })));
        assert!(!wiki_links(&json!({})));
    }
}
