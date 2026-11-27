//! What the last render evaluated in the processed document: each interpolation and expression param written in it,
//! by where it is, with the first value it gave and how many times; none for a failure, nothing for what the render
//! didn't reach or another document wrote; kept in place across edits.

use compmd::context::RenderContext;
use compmd::expand::Options;
use compmd::processor::Expanded;
use compmd::{Ambient, ParseOptions, Processor, TextEdit};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::rc::Rc;

/// `text` processed, with `files` beside it.
fn processed(text: &str, files: &[(&str, &str)]) -> Processor<Expanded> {
    let files: HashMap<String, String> = files
        .iter()
        .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
        .collect();
    let context = RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    };
    Processor::new("notes.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve(files, (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&Options::default())
}

/// Each evaluation's line, column, first value as JSON (`None` for a failure) and count.
fn evaluated(processor: &Processor<Expanded>) -> Vec<(usize, usize, Option<Value>, usize)> {
    processor
        .evaluations()
        .iter()
        .map(|evaluation| {
            let value = evaluation.value.as_ref().map(compmd::cel::Value::to_json);
            (
                evaluation.at.line,
                evaluation.at.column,
                value,
                evaluation.count,
            )
        })
        .collect()
}

const NOTES: &str = "---\ntitle: Harbour notes\nteam: [Ana, Kenji, Priya]\n---\n\n# {{md.doc.title}}\n\n```compmd component=badge\nparams:\n  status: string\n---\n:value[s]{of=(status == \"done\" ? \"success\" : \"neutral\")}\n```\n\n:: badge[B] {status=done}\n\n:: badge[B] {status=todo}\n\n::: each {in=(md.doc.team) as=p join=inline sep=\", \"}\n{{p}}\n::: /each\n\n::: if {when=(false)}\n{{md.doc.title}}\n::: /if\n";

#[test]
fn each_is_at_its_path_or_its_parenthesis_with_its_first_value_and_count() {
    assert_eq!(
        evaluated(&processed(NOTES, &[])),
        [
            (6, 5, Some(json!("Harbour notes")), 1),
            (12, 14, Some(json!("success")), 2),
            (19, 14, Some(json!(["Ana", "Kenji", "Priya"])), 1),
            (20, 3, Some(json!("Ana")), 3),
            (23, 14, Some(json!(false)), 1),
        ],
        "the branch `if` didn't take isn't there"
    );
}

#[test]
fn a_failure_has_no_value() {
    let text = "---\nteam: [Ana]\n---\n\n{{md.doc.team}}\n\n:value[v]{of=(string(1 / 0))}\n\n::: each {in=([0, 1]) as=n}\n:value[v]{of=(string(1 / n))}\n::: /each\n";
    assert_eq!(
        evaluated(&processed(text, &[])),
        [
            (5, 3, None, 1),
            (7, 14, None, 1),
            (9, 14, Some(json!([0, 1])), 1),
            (10, 14, None, 2)
        ],
        "a list isn't written; one iteration of two fails"
    );
}

#[test]
fn each_interpolation_of_an_alt_text_is_at_its_own_path() {
    let text = "---\na: One\nb: Two\n---\n\n![{{md.doc.a}} and {{md.doc.b}}](x.png)\n";
    assert_eq!(
        evaluated(&processed(text, &[])),
        [
            (6, 5, Some(json!("One")), 1),
            (6, 22, Some(json!("Two")), 1)
        ]
    );
}

#[test]
fn what_another_document_writes_is_not_there() {
    let ui = "---\ntitle: UI\nmd:\n  export:\n    names: [note]\n---\n\n```compmd component=note\n---\n{{md.doc.title}}\n```\n";
    let text =
        "---\nmd:\n  import:\n    - from: ./ui.comp.md\n      names: [note]\n---\n\n:: note[N]\n";
    let processor = processed(text, &[("ui.comp.md", ui)]);
    assert_eq!(processor.diagnostics(), []);
    assert_eq!(evaluated(&processor), []);
}

#[test]
fn an_edit_moves_them_and_evaluates_again_what_it_changed() {
    let text = "---\ntitle: Harbour notes\n---\n\nIntro.\n\n{{md.doc.title}}\n";
    let processor = processed(text, &[]);
    assert_eq!(
        evaluated(&processor),
        [(7, 3, Some(json!("Harbour notes")), 1)]
    );
    // A line typed after `Intro.`: the interpolation, expanded as it was, moves a line down.
    let processor = processor.edit(&TextEdit {
        start: 36,
        end: 36,
        text: "\nMore.".into(),
    });
    assert_eq!(
        evaluated(&processor),
        [(8, 3, Some(json!("Harbour notes")), 1)]
    );
    // The title changed in the frontmatter.
    let processor = processor.edit(&TextEdit {
        start: 11,
        end: 18,
        text: "Quay".into(),
    });
    assert_eq!(
        evaluated(&processor),
        [(8, 3, Some(json!("Quay notes")), 1)]
    );
}
