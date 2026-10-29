//! Hints in the frontmatter (spec §2.1, §6, §7.3, §8.6, §8.7): what each key under `md` holds, the keys of imports,
//! exports, data imports and long declarations, and the type names of inputs and schemas. A line's keys are found by
//! its indentation, as YAML nests them.

use super::{Completion, Found, KindHint, Place, Subject};
use crate::parse::frontmatter::{MD_KEYS, SYNTAX_NAMES};
use crate::tree::TypeName;

/// The keys of a component import (spec §6.1).
const IMPORT_KEYS: [(&str, &str); 3] = [
    (
        "from",
        "where the components come from: a relative path, a pack such as `\"@acme/ui@1\"`, or a component file",
    ),
    (
        "names",
        "the names it takes: a list, `{name: card, as: promo}` to rename one, or `\"*\"` for all",
    ),
    (
        "integrity",
        "the hash the pack must match, such as `sha512-…`",
    ),
];

/// The keys of the exports (spec §6.2).
const EXPORT_KEYS: [(&str, &str); 3] = [
    ("pack", "the pack's name, as its registry manifest has it"),
    (
        "version",
        "the pack's version, as its registry manifest has it",
    ),
    (
        "names",
        "the names other documents may import: a list, or `\"*\"` for all",
    ),
];

/// The keys of a data import (spec §8.6).
const DATA_KEYS: [(&str, &str); 6] = [
    (
        "from",
        "where the data comes from: a relative path, a pack's file, an `https://` URL or a `host:` source",
    ),
    ("as", "the name it gets: `as: team` makes it `md.data.team`"),
    ("schema", "the type of each field of its records"),
    (
        "cache",
        "how long a fetched copy may be reused, such as `1h`; remote only",
    ),
    (
        "fallback",
        "a local file loaded when the source fails; remote and `host:` only",
    ),
    ("delimiter", "a CSV file's field separator, `,` by default"),
];

/// The keys of a declaration's long form (spec §7.3).
const DECLARATION_KEYS: [(&str, &str); 4] = [
    ("type", "a type, with an optional `?`"),
    ("default", "a value of that type"),
    ("doc", "a description for editors and tools"),
    (
        "from",
        "in a data `schema`, the key in the data when it differs",
    ),
];

/// The values `md.dir` takes.
const DIRECTIONS: [&str; 3] = ["ltr", "rtl", "auto"];

/// A line of YAML, read for its key: its indentation, whether it is a list item, its key's code points, and where
/// its value starts.
struct Line {
    indent: usize,
    dash: bool,
    key: Option<(usize, usize)>,
    value: usize,
}

fn read(text: &[char]) -> Line {
    let indent = text.iter().take_while(|c| **c == ' ').count();
    let dash = text.get(indent) == Some(&'-') && text.get(indent + 1).is_none_or(|c| *c == ' ');
    let start = if dash {
        indent + 1 + text[indent + 1..].iter().take_while(|c| **c == ' ').count()
    } else {
        indent
    };
    let end = start
        + text[start..]
            .iter()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
            .count();
    let keyed =
        end > start && text.get(end) == Some(&':') && text.get(end + 1).is_none_or(|c| *c == ' ');
    let value_start = if keyed { end + 1 } else { start };
    Line {
        indent,
        dash,
        key: keyed.then_some((start, end)),
        value: value_start
            + text[value_start..]
                .iter()
                .take_while(|c| **c == ' ')
                .count(),
    }
}

/// The keys the line `number` is under, outermost first.
fn parents(place: &Place, number: usize, line: &Line) -> Vec<String> {
    let lines = place.document.lines();
    let mut want = match line.key {
        Some((start, _)) if !line.dash => start,
        _ => line.indent,
    };
    let mut found = Vec::new();
    for above in (2..number).rev() {
        if want == 0 {
            break;
        }
        let text: Vec<char> = lines.line(above).unwrap_or("").chars().collect();
        let blank =
            text.iter().all(|c| *c == ' ') || text.iter().find(|c| **c != ' ') == Some(&'#');
        if blank {
            continue;
        }
        let read = read(&text);
        if let Some((start, end)) = read.key.filter(|(start, _)| *start < want) {
            found.push(text[start..end].iter().collect());
            want = if read.dash { read.indent } else { start };
        } else if read.dash && read.indent < want {
            want = read.indent;
        }
    }
    found.reverse();
    found
}

/// The keys a mapping at `path` has, each with what it holds.
fn keys(path: &[&str]) -> &'static [(&'static str, &'static str)] {
    match path {
        [] => &[(
            "md",
            "CompMD's own keys; every other key is the author's, read as `md.doc.<key>`",
        )],
        ["md"] => &MD_KEYS,
        ["md", "import"] => &IMPORT_KEYS,
        ["md", "export"] => &EXPORT_KEYS,
        ["md", "data"] => &DATA_KEYS,
        ["md", "inputs", _] | ["md", "data", "schema", _] => &DECLARATION_KEYS,
        _ => &[],
    }
}

/// Whether a value at `path` is a declaration, whose type comes first: an input's, a schema field's, or a long
/// form's `type`.
fn declares(path: &[&str]) -> bool {
    matches!(
        path,
        ["md", "inputs", _] | ["md", "data", "schema", _] | ["md", "inputs" | "data", .., "type"]
    )
}

/// The hint on the frontmatter's line at the caret.
pub(super) fn hint(place: &Place) -> Option<Found> {
    let text = &place.text;
    let at = place.at;
    let line = read(text);
    let mut path = parents(place, place.line, &line);
    if let Some((start, end)) = line.key {
        let key: String = text[start..end].iter().collect();
        if (start..=end).contains(&at) {
            let parents: Vec<&str> = path.iter().map(String::as_str).collect();
            return Some(key_hint(&parents, &key, start, end));
        }
        path.push(key);
    }
    if at < line.value {
        return None;
    }
    let path: Vec<&str> = path.iter().map(String::as_str).collect();
    value_hint(&path, text, line.value, at)
}

/// The hint on the key `key` at `start..end`, under `path`.
fn key_hint(path: &[&str], key: &str, start: usize, end: usize) -> Found {
    let known = keys(path);
    let about = known
        .iter()
        .find(|(name, _)| *name == key)
        .map(|(_, about)| (*about).to_string())
        .or_else(|| match path {
            [] => Some(format!("the author's key, read as `md.doc.{key}`")),
            ["md", "inputs"] => Some(format!("an input, read as `md.input.{key}`")),
            ["md", "data", "schema"] => Some(format!("a field of each record, read as `.{key}`")),
            [first, ..] if *first != "md" => Some(format!(
                "the author's key, read as `md.doc.{}.{key}`",
                path.join(".")
            )),
            _ => None,
        });
    let mut written: Vec<String> = path.iter().map(ToString::to_string).collect();
    written.push(key.to_string());
    Found {
        start,
        end,
        subject: Subject::Frontmatter {
            path: written,
            about,
        },
        completions: known
            .iter()
            .map(|(name, about)| Completion::new(name, Some((*about).to_string())))
            .collect(),
    }
}

/// The hint on the value at `path`, which starts at `value` of `text`.
fn value_hint(path: &[&str], text: &[char], value: usize, at: usize) -> Option<Found> {
    let word = |from: usize| {
        let end = from
            + text[from..]
                .iter()
                .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-'))
                .count();
        (from, end)
    };
    if declares(path) {
        let (start, end) = word(value);
        return (at <= end).then(|| type_hint(text, start, end));
    }
    if text.get(value) == Some(&'{') && matches!(path, ["md", "inputs"] | ["md", "data", "schema"])
    {
        let entry = text[..at].iter().rposition(|c| matches!(c, '{' | ','))?;
        let colon = entry + text[entry..].iter().position(|c| *c == ':')?;
        let type_start = colon + 1 + text[colon + 1..].iter().take_while(|c| **c == ' ').count();
        let (start, end) = word(type_start);
        return (start <= at && at <= end).then(|| type_hint(text, start, end));
    }
    let tokens: &[&str] = match path {
        ["md", "syntax"] => &SYNTAX_NAMES,
        ["md", "dir"] => &DIRECTIONS,
        _ => return None,
    };
    let start = at
        - text[value..at]
            .iter()
            .rev()
            .take_while(|c| c.is_ascii_alphanumeric() || **c == '-')
            .count();
    let (_, end) = word(at);
    let about = MD_KEYS
        .iter()
        .find(|(name, _)| Some(name) == path.last())
        .map(|(_, about)| (*about).to_string());
    Some(Found {
        start,
        end,
        subject: Subject::Frontmatter {
            path: path.iter().map(ToString::to_string).collect(),
            about,
        },
        completions: tokens
            .iter()
            .map(|token| Completion::new(token, None))
            .collect(),
    })
}

/// The hint on the type name at `start..end` of `text`.
fn type_hint(text: &[char], start: usize, end: usize) -> Found {
    let name: String = text[start..end].iter().collect();
    Found {
        start,
        end,
        subject: Subject::Type {
            takes: TypeName::from_name(&name).map(|known| KindHint::of(&known.kind())),
            name,
        },
        completions: TypeName::NAMES
            .iter()
            .filter(|name| **name != "enum")
            .map(|name| Completion::new(name, None))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{CARET, hint, range, texts};
    use super::*;
    use serde_json::{Value, json};

    /// The hint at the caret of a document with the frontmatter `yaml`.
    fn yaml(yaml: &str) -> Value {
        hint(&format!("---\n{yaml}\n---\n\nText.\n"))
    }

    #[test]
    fn a_key_says_what_it_holds() {
        let version = yaml(&format!("md:\n  ver{CARET}sion: 1"));
        assert_eq!(version["range"], range(3, 3, 10));
        let subject = &version["subject"];
        assert_eq!(
            (&subject["kind"], &subject["path"]),
            (&json!("frontmatter"), &json!(["md", "version"]))
        );
        assert_eq!(subject["about"], MD_KEYS[0].1);
        assert_eq!(texts(&version).len(), MD_KEYS.len());
        assert!(
            yaml(&format!("m{CARET}d:\n  version: 1"))["subject"]["about"]
                .as_str()
                .is_some_and(|about| about.starts_with("CompMD's own keys"))
        );
        let author = yaml(&format!("ti{CARET}tle: Reading"));
        assert_eq!(
            author["subject"]["about"],
            "the author's key, read as `md.doc.title`"
        );
        let nested = yaml(&format!("shelf:\n  # where\n\n  na{CARET}me: Home"));
        assert_eq!(
            nested["subject"]["about"],
            "the author's key, read as `md.doc.shelf.name`"
        );
        let below = yaml(&format!("title: x\nmd:\n  ver{CARET}sion: 1"));
        assert_eq!(below["subject"]["path"], json!(["md", "version"]));
        assert_eq!(
            yaml(&format!("md:\n  syntax:\n    {CARET}- wiki-links")),
            Value::Null,
            "on the dash"
        );
        let unknown = yaml(&format!("md:\n  imprt{CARET}: []"));
        assert_eq!(unknown["subject"].get("about"), None);
        assert_eq!(unknown["diagnostics"][0]["code"], "W111");
        assert_eq!(hint(&format!("--{CARET}-\ntitle: x\n---\n")), Value::Null);
    }

    #[test]
    fn imports_exports_and_data_have_their_keys() {
        let data = format!("md:\n  data:\n    - fr{CARET}om: ./books.csv\n      as: books\n");
        assert_eq!(yaml(&data)["subject"]["about"], DATA_KEYS[0].1);
        assert_eq!(
            yaml(&data)["subject"]["path"],
            json!(["md", "data", "from"])
        );
        let name = data.replacen(CARET, "", 1).replacen("as:", "a‸s:", 1);
        assert_eq!(yaml(&name)["subject"]["about"], DATA_KEYS[1].1);
        let import = yaml(&format!(
            "md:\n  import:\n    - from: ./kit.comp.md\n      na{CARET}mes: [card]"
        ));
        assert_eq!(import["subject"]["about"], IMPORT_KEYS[1].1);
        let export = yaml(&format!(
            "md:\n  export:\n    pa{CARET}ck: \"@a/b\"\n    names: \"*\""
        ));
        assert_eq!(export["subject"]["about"], EXPORT_KEYS[0].1);
        let input = yaml(&format!("md:\n  inputs:\n    si{CARET}nce: number = 1970"));
        assert_eq!(
            input["subject"]["about"],
            "an input, read as `md.input.since`"
        );
        let long = yaml(&format!(
            "md:\n  inputs:\n    since:\n      type: number\n      do{CARET}c: The first year"
        ));
        assert_eq!(long["subject"]["about"], DECLARATION_KEYS[2].1);
        let field = yaml(&format!(
            "md:\n  data:\n    - from: ./b.csv\n      schema:\n        ti{CARET}tle: string"
        ));
        assert_eq!(
            field["subject"]["about"],
            "a field of each record, read as `.title`"
        );
    }

    #[test]
    fn a_declaration_names_its_type() {
        let number = yaml(&format!("md:\n  inputs:\n    since: num{CARET}ber = 1970"));
        assert_eq!(number["range"], range(4, 12, 18));
        let subject = &number["subject"];
        assert_eq!(
            (&subject["kind"], &subject["name"]),
            (&json!("type"), &json!("number"))
        );
        assert_eq!(
            subject["takes"]["description"],
            "a number, such as `12` or `0.5`"
        );
        let types = texts(&number);
        assert!(types.contains(&"string") && types.contains(&"radius") && !types.contains(&"enum"));
        assert_eq!(
            yaml(&format!("md:\n  inputs:\n    since: number = 19{CARET}70")),
            Value::Null
        );
        let partial = yaml(&format!("md:\n  inputs:\n    since: nu{CARET}"));
        assert_eq!(
            (&partial["subject"]["name"], partial["subject"].get("takes")),
            (&json!("nu"), None)
        );
        let flow = yaml(&format!(
            "md:\n  data:\n    - from: ./b.csv\n      schema: {{title: string, pages: num{CARET}ber}}"
        ));
        assert_eq!(flow["subject"]["name"], "number");
        let key = yaml(&format!(
            "md:\n  data:\n    - from: ./b.csv\n      schema: {{title: string, pa{CARET}ges: number}}"
        ));
        assert_eq!(key, Value::Null, "a key in a flow mapping");
        let block = yaml(&format!(
            "md:\n  data:\n    - from: ./b.csv\n      schema:\n        title: str{CARET}ing"
        ));
        assert_eq!(block["subject"]["name"], "string");
        let long = yaml(&format!(
            "md:\n  inputs:\n    since:\n      type: bool{CARET}ean"
        ));
        assert_eq!(long["subject"]["takes"]["tokens"], json!(["true", "false"]));
        let inputs = yaml(&format!("md:\n  inputs: {{since: num{CARET}ber}}"));
        assert_eq!(inputs["subject"]["name"], "number");
    }

    #[test]
    fn syntax_and_direction_offer_their_values() {
        let syntax = yaml(&format!("md:\n  syntax: [wiki{CARET}-links]"));
        assert_eq!(syntax["range"], range(3, 12, 22));
        assert_eq!(syntax["subject"]["path"], json!(["md", "syntax"]));
        assert_eq!(texts(&syntax), ["wiki-links"]);
        let item = yaml(&format!("md:\n  syntax:\n    - wi{CARET}ki-links"));
        assert_eq!(texts(&item), ["wiki-links"]);
        let dir = yaml(&format!("md:\n  dir: r{CARET}tl"));
        assert_eq!(texts(&dir), ["ltr", "rtl", "auto"]);
        assert_eq!(dir["subject"]["about"], MD_KEYS[6].1);
        assert_eq!(yaml(&format!("md:\n  lang: e{CARET}n")), Value::Null);
    }

    /// The text of the spec's section that starts with the heading `from` and ends at `to`.
    fn section(from: &str, to: &str) -> String {
        let spec =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../compmd-spec-v1.md"))
                .expect("the spec");
        spec.split(from)
            .nth(1)
            .and_then(|rest| rest.split(to).next())
            .unwrap_or_else(|| panic!("{from}"))
            .to_string()
    }

    #[test]
    fn the_spec_lists_every_key() {
        for (from, to, keys) in [
            ("### 2.1 ", "### 2.2 ", &MD_KEYS[..]),
            ("### 6.1 ", "### 6.2 ", &IMPORT_KEYS[..]),
            ("### 6.2 ", "### 6.3 ", &EXPORT_KEYS[..]),
            ("### 7.3 ", "### 7.4 ", &DECLARATION_KEYS[..]),
            ("### 8.6 ", "### 8.7 ", &DATA_KEYS[..]),
        ] {
            let text = section(from, to);
            for (key, _) in keys {
                assert!(
                    text.contains(&format!("{key}`")),
                    "spec {from}doesn't name `{key}`"
                );
            }
        }
    }
}
