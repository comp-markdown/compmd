//! Editing an expanded processor: after every edit, it gives the trees and diagnostics a new processor of the new
//! text gives, through every stage, and its splice turns the expanded tree before the edit into the one after it.

mod common;

use common::{Setup, cases, conformance, read_json};
use compmd::context::RenderContext;
use compmd::expand::Options;
use compmd::{Ambient, ParseOptions, Processor, TextEdit};
use serde_json::{Value, json};
use std::path::Path;
use std::rc::Rc;

/// A small, seeded random number generator (xorshift), so a failure repeats.
struct Random(u64);

impl Random {
    fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        usize::try_from(self.0 % bound as u64).unwrap_or(0)
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
}

/// What edits type: characters with a meaning in Markdown or CompMD.
const TYPED: &[&str] = &[
    "a", " ", "\n", "\n\n", "#", "-", "*", ":", "::", "[", "]", "`", ">", "|", "{", "}", "(", ")",
    "=", "+", "!", "\"", "\t", "é", "😀", "1.",
];

/// What edits paste: blocks and inline parts that every stage handles.
const PASTED: &[&str] = &[
    "\n\n::: card {title=\"x\"}\nBody.\n::: /card\n\n",
    "::: box {padding=s}\n",
    "::: /box\n",
    "\n```\n",
    "```compmd component=note\n---\n# Note {{md.doc.title}}\n```\n",
    "```compmd component=pill\nparams:\n  size: numbr\n---\nPill.\n```\n",
    "\n[l]: /x\n",
    "\n[^1]: A note.\n",
    "See [l] and[^1].",
    ":: chart[Weekly] {rows=(md.data.rows)}\n",
    ":: note\n",
    ":badge[b]{tone=\"{{md.input.tone}}\"}",
    "::: + gap=s\n",
    "\n# Title\n\n",
    "## Part\n",
    "{{md.doc.title}}",
    "{{md.input.n}}",
    "{{missing.path}}",
    "[a](./{{md.doc.title}}.html)",
    ":: value {of=(1 + 2)}\n",
    "::: if {when=(md.input.n > 1)}\nMore.\n::: /if\n",
    "::: each {in=(md.doc.team) as=p}\n- {{p}}\n::: /each\n",
    "\n- a\n- b\n\n  c\n",
    "| a | b |\n|---|---|\n| 1 | 2 |\n",
    "Title\n===\n",
    "\n---\nmd:\n  version: 1\n---\n",
];

fn random_edit(text: &str, random: &mut Random) -> TextEdit {
    let length = text.chars().count();
    let start = random.below(length + 1);
    let (end, text) = match random.below(6) {
        0 | 1 => (start, random.pick(TYPED).to_string()),
        2 => (start, random.pick(PASTED).to_string()),
        3 => ((start + 1 + random.below(3)).min(length), String::new()),
        4 => (
            (start + random.below(40)).min(length),
            random.pick(TYPED).to_string(),
        ),
        _ => (
            (start + random.below(200)).min(length),
            random.pick(PASTED).to_string(),
        ),
    };
    TextEdit { start, end, text }
}

/// `text` after `edit`, in code points, a position past the end being the end.
fn applied(text: &str, edit: &TextEdit) -> String {
    let body = text.strip_prefix('\u{feff}').unwrap_or(text);
    let bom = &text[..text.len() - body.len()];
    let chars: Vec<char> = body.chars().collect();
    let end = edit.end.min(chars.len());
    let start = edit.start.min(end);
    let before: String = chars[..start].iter().collect();
    let after: String = chars[end..].iter().collect();
    format!("{bom}{before}{}{after}", edit.text)
}

/// `node` with every origin in it moved by `lines` and `offset`.
fn moved(node: &Value, lines: isize, offset: isize) -> Value {
    match node {
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|item| moved(item, lines, offset))
                .collect(),
        ),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| {
                    let value = if key == "origin" && value.is_object() {
                        let mut origin = value.clone();
                        for end in ["start", "end"] {
                            let point = &mut origin[end];
                            let shift = |value: &Value, by: isize| {
                                json!(value.as_u64().unwrap_or(0).cast_signed() + by as i64)
                            };
                            point["line"] = shift(&point["line"], lines);
                            point["offset"] = shift(&point["offset"], offset);
                        }
                        origin
                    } else {
                        moved(value, lines, offset)
                    };
                    (key.clone(), value)
                })
                .collect(),
        ),
        other => other.clone(),
    }
}

/// Edits `text`, the document `path` of `setup`, with random edits from `seed`, checking each against a new processor.
/// How many edits it made, and how many of them replaced part of the expanded tree, not all of it.
fn edit_and_compare(setup: &Setup, path: &str, text: &str, seed: u64) -> (usize, usize) {
    let mut text = text.to_string();
    let mut processor = setup.process(path, &text);
    let mut random = Random(seed);
    let (mut edits, mut partial) = (0, 0);
    for _ in 0..12 {
        let edit = random_edit(&text, &mut random);
        let before =
            serde_json::to_value(processor.expanded()).expect("a tree")["children"].clone();
        let new_text = applied(&text, &edit);
        processor = processor.edit(&edit);
        let fresh = setup.process(path, &new_text);
        let context = || {
            format!(
                "{path}, origins {}\nthe text {text:?}\nthe edit {edit:?}\nthe new text {new_text:?}",
                setup.options.origins
            )
        };
        assert!(processor.tree() == fresh.tree(), "the tree: {}", context());
        assert!(
            processor.diagnostics() == fresh.diagnostics(),
            "the diagnostics {:#?} and {:#?}: {}",
            processor.diagnostics(),
            fresh.diagnostics(),
            context()
        );
        let after = serde_json::to_value(processor.expanded()).expect("a tree");
        let expected = serde_json::to_value(fresh.expanded()).expect("a tree");
        assert!(
            after == expected,
            "the expanded tree {after:#} and {expected:#}: {}",
            context()
        );
        let splice = processor.splice();
        let items = before.as_array().cloned().unwrap_or_default();
        let nodes = serde_json::to_value(&splice.nodes).expect("nodes");
        let spliced: Vec<Value> = items[..splice.from]
            .iter()
            .cloned()
            .chain(nodes.as_array().cloned().unwrap_or_default())
            .chain(
                items[splice.to..]
                    .iter()
                    .map(|node| moved(node, splice.lines, splice.offset)),
            )
            .collect();
        assert!(
            Value::Array(spliced) == after["children"],
            "the splice {} {} {} {}: {}",
            splice.from,
            splice.to,
            splice.lines,
            splice.offset,
            context()
        );
        edits += 1;
        if splice.to - splice.from < items.len() {
            partial += 1;
        }
        text = new_text;
    }
    (edits, partial)
}

/// Every processor case of the conformance set and every example, with origins and without.
#[test]
fn an_edited_processor_gives_what_a_new_one_gives() {
    let mut documents = Vec::new();
    let mut folders = Vec::new();
    cases(&conformance(), &mut folders);
    for folder in folders {
        let case = read_json(&folder.join("case.json")).expect("case.json");
        let requires = case["requires"].as_array().cloned().unwrap_or_default();
        if requires.contains(&json!("processor")) {
            documents.push((folder, case, "input.comp.md".to_string()));
        }
    }
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples");
    for entry in std::fs::read_dir(&examples)
        .expect("the examples")
        .flatten()
    {
        let folder = entry.path();
        for file in std::fs::read_dir(&folder).into_iter().flatten().flatten() {
            let name = file.file_name().to_string_lossy().to_string();
            if name.ends_with(".comp.md") {
                documents.push((folder.clone(), Value::Null, name));
            }
        }
    }
    let bench = Path::new(env!("CARGO_MANIFEST_DIR")).join("../wasm/bench/fixtures/notes");
    documents.push((bench, Value::Null, "notes.comp.md".to_string()));
    documents.sort_by(|one, other| (&one.0, &one.2).cmp(&(&other.0, &other.2)));
    assert!(
        documents.len() > 70,
        "the processor cases and the examples: {}",
        documents.len()
    );
    let (mut edits, mut partial) = (0, 0);
    for (index, (folder, case, path)) in documents.iter().enumerate() {
        let mut setup = Setup::of(folder, case);
        let text = setup.files[path].clone();
        for origins in [false, true] {
            setup.options.origins = origins;
            let seed = 0x2545_f491_4f6c_dd1d ^ ((index as u64 + 1) << 1 | u64::from(origins));
            let (made, spliced) = edit_and_compare(&setup, path, &text, seed);
            edits += made;
            partial += spliced;
        }
    }
    // The cases are short and most define components, so an edit often lands in a definition or the frontmatter,
    // which runs the later stages on the whole document.
    assert!(
        partial * 4 > edits,
        "a quarter of the edits replace part of the tree: {partial} of {edits}"
    );
}

/// `text` processed with `options` and no files, then edited with each of `edits` in turn, each checked against a new
/// processor; the splice of each, and the codes of its diagnostics.
fn spliced(
    text: &str,
    options: &Options,
    edits: &[TextEdit],
) -> Vec<((usize, usize, usize), Vec<String>)> {
    let context = RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::default(),
    };
    let process = |text: &str| {
        Processor::new("a.comp.md", text)
            .parse(&ParseOptions::default())
            .resolve((), (), &Ambient::default())
            .validate(&[])
            .load(context.clone(), (), ())
            .expand(options)
    };
    let mut text = text.to_string();
    let mut processor = process(&text);
    let mut splices = Vec::new();
    for edit in edits {
        text = applied(&text, edit);
        processor = processor.edit(edit);
        let fresh = process(&text);
        assert_eq!(processor.diagnostics(), fresh.diagnostics(), "{text:?}");
        assert_eq!(
            serde_json::to_value(processor.expanded()).unwrap(),
            serde_json::to_value(fresh.expanded()).unwrap(),
            "{text:?}"
        );
        let splice = processor.splice();
        let codes = processor
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code.to_string())
            .collect();
        splices.push(((splice.from, splice.to, splice.nodes.len()), codes));
    }
    splices
}

fn typed(start: usize, text: &str) -> TextEdit {
    TextEdit {
        start,
        end: start,
        text: text.into(),
    }
}

#[test]
fn a_component_with_relative_headings_follows_a_heading_before_it() {
    let text = "```compmd component=section\nparams:\n  title: string\n---\n# {{title}}\n```\n\n# A\n\nPara.\n\n:: section[S] {title=\"S\"}\n\nEnd.\n";
    let heading = text.find("# A").expect("the heading");
    let para = text.find("Para.").expect("the paragraph");
    let splices = spliced(
        text,
        &Options::default(),
        &[
            typed(para + 4, "!"),
            typed(heading, "#"),
            typed(para + 6, "?"),
        ],
    );
    assert_eq!(splices[0].0, (1, 2, 1), "the paragraph alone");
    assert_eq!(
        splices[1].0,
        (0, 3, 3),
        "the heading, and the section that follows it, with the paragraph between"
    );
    assert_eq!(splices[2].0, (1, 2, 1), "the paragraph alone again");
}

#[test]
fn an_edit_that_crosses_the_node_limit_expands_what_follows_it_again() {
    let text = "::: each {in=([1, 2]) as=i}\n{{i}}\n::: /each\n\nAfter.\n\n::: each {in=([1, 2]) as=i}\n{{i}}\n::: /each\n";
    let list = text.find("2]").expect("the first list");
    let options = Options {
        limit: 12,
        ..Options::default()
    };
    let splices = spliced(
        text,
        &options,
        &[
            typed(list + 1, ", 3, 4, 5"),
            typed(text.find("After").expect("the paragraph") + 9, "!"),
            TextEdit {
                start: list + 1,
                end: list + 10,
                text: String::new(),
            },
        ],
    );
    let (crossed, typed, back) = (&splices[0], &splices[1], &splices[2]);
    assert_eq!(crossed.1, ["F034"], "the second list crosses the limit");
    assert_eq!(crossed.0.0, 0, "from the first list on");
    assert_eq!(typed.1, ["F034"]);
    assert!(back.1.is_empty(), "under the limit again");
    assert_eq!(back.0.0, 0);
}

#[test]
fn the_origins_after_an_edit_move_with_their_slots() {
    let text = "Intro.\n\n```jsx component=card\nexport default () => null;\n```\n\n::: card\nBody.\n:: #footer\nEnd.\n::: /card\n";
    let options = Options {
        origins: true,
        ..Options::default()
    };
    let splices = spliced(text, &options, &[typed(5, "\n\nMore")]);
    assert_eq!(splices[0].0, (0, 1, 2), "the paragraph became two");
}

#[test]
fn an_edit_that_changes_which_data_may_load_loads_again() {
    let text = "---\nmd:\n  inputs:\n    picked: string = \"\"\n  data:\n    - from: ./data-{{md.input.picked}}.json\n      as: picked\n---\n\n```compmd component=chart signature\nparams:\n  selected: input<string>\n```\n\n```vue component=chart\n<template><svg /></template>\n```\n\nIntro.\n\n:: chart[Sales] {selected=picked}\n";
    let line = text.find(":: chart").expect("the invocation");
    let removed = TextEdit {
        start: line,
        end: text.chars().count(),
        text: String::new(),
    };
    let splices = spliced(
        text,
        &Options::default(),
        &[removed, typed(line, ":: chart[Sales] {selected=picked}\n")],
    );
    assert!(
        splices[0].1.contains(&"F026".to_string()),
        "the data loads, and isn't there: {:?}",
        splices[0].1
    );
    assert!(
        splices[1].1.contains(&"F031".to_string()),
        "{:?}",
        splices[1].1
    );
}

#[test]
fn an_edit_past_the_end_appends() {
    let context = RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::default(),
    };
    let processor = Processor::new("a.comp.md", "\u{feff}One.\n")
        .parse(&ParseOptions::default())
        .resolve((), (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&Options::default())
        .edit(&TextEdit {
            start: 99,
            end: 50,
            text: "\nTwo.\n".into(),
        });
    let json = serde_json::to_value(processor.tree()).unwrap();
    assert_eq!(json["children"][1]["children"][0]["value"], "Two.");
}
