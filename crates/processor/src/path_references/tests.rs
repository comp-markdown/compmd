use std::rc::Rc;

use serde_json::Value;

use super::{KeyRead, MetadataPath, PathError};
use crate::context::RenderContext;
use crate::processor::Expanded;
use crate::{Ambient, ParseOptions, Processor};

/// A document reading `md.doc.due_date` in each place an expression can be, and the same text where it is no read: in
/// code, in prose, in a CEL string. `KEY` is the frontmatter's key.
const DOCUMENT: &str = r#"---
KEY: "1 December"
key: KEY
items: [a, b]
---

Due {{md.doc.due_date}}.

::: if {when=(has(md.doc.due_date))}
::: box {title="Due {{ md.doc.due_date }}" padding=(md.doc['due_date'] == "" ? "s" : "m")}
Soon.
::: /box
::: /if

::: each {in=(md.doc.items.filter(i, md.doc.due_date != "")) as=i}
Item {{i}}, :value[due]{of=(md.doc.due_date)}.
::: /each

```compmd component=due
params:
  when: string
---
Due {{when}}.
```

:: due[Due] {when=(md.doc.due_date)}

Not reads: `md.doc.due_date` in code, md.doc.due_date in prose, :value[x]{of=("md.doc.due_date")} in a string, and {{md.doc.due_dates}}.

```text
{{md.doc.due_date}}
```

A key built when it runs: :value[x]{of=(md.doc[md.doc.key])}.
"#;

fn processed(text: &str) -> Processor<Expanded> {
    let context = RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    };
    Processor::new("a.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve((), (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&crate::expand::Options::default())
}

/// The document with its frontmatter's key `due_date`, or `due` after a rename.
fn document(key: &str) -> String {
    DOCUMENT.replacen("KEY", key, 1).replacen("KEY", key, 1)
}

/// What each read covers in `text`, with its line.
fn written(text: &str, reads: &[KeyRead]) -> Vec<(usize, String)> {
    let chars: Vec<char> = text.chars().collect();
    reads
        .iter()
        .map(|read| {
            let line = chars[..read.start].iter().filter(|c| **c == '\n').count() + 1;
            (line, chars[read.start..read.end].iter().collect())
        })
        .collect()
}

/// The expanded tree without what says where it is, which a rename moves, and without the frontmatter, whose key the
/// rename changes.
fn placeless(value: Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .filter(|(key, _)| !key.to_lowercase().contains("position") && key != "origin")
                .map(|(key, value)| (key, placeless(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(
            items
                .into_iter()
                .filter(|item| item["type"] != "yaml")
                .map(placeless)
                .collect(),
        ),
        other => other,
    }
}

#[test]
fn every_read_of_the_key_in_every_place_and_none_of_the_look_alikes() {
    let text = document("due_date");
    let found = processed(&text)
        .path_references("md.doc.due_date")
        .expect("a path");
    let due = |line: usize| (line, "due_date".to_string());
    assert_eq!(
        written(&text, &found.reads),
        [due(7), due(9), due(10), due(10), due(15), due(16), due(26)],
        "text, an if, a quoted param and an expression, an each, :value and a param of a definition"
    );
    assert_eq!(
        written(&text, &found.unrewritable),
        [(34, "[".to_string())],
        "the index whose key is computed"
    );
}

#[test]
fn renamed_the_document_processes_to_the_same_output_with_the_new_key() {
    let text = document("due_date");
    let before = processed(&text);
    let renamed = before
        .rename_key("md.doc.due_date", "due")
        .expect("a rename");
    assert_eq!(renamed.edits.len(), 7);
    assert!(renamed.edits.iter().all(|edit| edit.text == "due"));
    let mut chars: Vec<char> = text.chars().collect();
    for edit in renamed.edits.iter().rev() {
        chars.splice(edit.start..edit.end, edit.text.chars());
    }
    // The frontmatter is the host's to rename: its key, and the value the computed index reads.
    let edited: String = chars
        .into_iter()
        .collect::<String>()
        .replacen("due_date:", "due:", 1)
        .replacen("key: due_date", "key: due", 1);
    for untouched in [
        "Not reads: `md.doc.due_date` in code, md.doc.due_date in prose, :value[x]{of=(\"md.doc.due_date\")} in a string, and {{md.doc.due_dates}}.",
        "```text\n{{md.doc.due_date}}\n```",
        ":value[x]{of=(md.doc[md.doc.key])}",
    ] {
        assert!(edited.contains(untouched), "{untouched}");
    }
    assert_eq!(
        edited.matches("due_date").count(),
        5,
        "the look-alikes only"
    );
    let after = processed(&edited);
    assert_eq!(
        placeless(serde_json::to_value(after.expanded()).expect("JSON")),
        placeless(serde_json::to_value(before.expanded()).expect("JSON")),
    );
}

#[test]
fn a_path_is_md_and_two_names_and_a_key_one_a_dot_can_select() {
    let path = MetadataPath::new("md.doc.address.city").expect("a path");
    assert_eq!((path.parent(), path.key()), ("md.doc.address", "city"));
    for wrong in ["md.doc", "doc.due", "md.doc.due-date", "md..due", ""] {
        assert_eq!(
            MetadataPath::new(wrong),
            Err(PathError::NotAPath(wrong.into())),
            "{wrong}"
        );
    }
    let processor = processed("{{md.doc.due}}\n");
    for wrong in ["due date", "due-date", "1st", ""] {
        assert_eq!(
            processor.rename_key("md.doc.due", wrong),
            Err(PathError::NotAKey(wrong.into())),
            "{wrong}"
        );
    }
    assert_eq!(
        PathError::NotAKey("due date".into()).to_string(),
        "`due date` isn't a key an expression can select, such as `due`"
    );
    assert_eq!(
        PathError::NotAPath("md".into()).to_string(),
        "`md` isn't a metadata path such as `md.doc.title`"
    );
}

#[test]
fn other_paths_literals_with_escapes_and_a_plain_markdown_file() {
    let text = ":value[x]{of=(md.input.week + md.data.sales.size() + md.doc[\"due\\u005fdate\"].size() + md.doc[r'''due_date'''].size())}\n";
    let processor = processed(text);
    let week = processor.path_references("md.input.week").expect("a path");
    assert_eq!(written(text, &week.reads), [(1, "week".to_string())]);
    let sales = processor.path_references("md.data.sales").expect("a path");
    assert_eq!(written(text, &sales.reads), [(1, "sales".to_string())]);
    let due = processor
        .path_references("md.doc.due_date")
        .expect("a path");
    assert_eq!(
        written(text, &due.reads),
        [(1, "due_date".to_string())],
        "a raw triple-quoted string as written"
    );
    assert_eq!(due.unrewritable.len(), 1, "one with an escape");
    let context = RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    };
    let plain = Processor::new("notes.md", "{{md.doc.due}}\n")
        .parse(&ParseOptions {
            plain_markdown: true,
            ..ParseOptions::default()
        })
        .resolve((), (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&crate::expand::Options::default());
    assert_eq!(
        plain.path_references("md.doc.due").expect("a path").reads,
        []
    );
    assert_eq!(
        plain
            .rename_key("md.doc.due", "when")
            .expect("a rename")
            .edits,
        []
    );
}
