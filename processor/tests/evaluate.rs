//! An expression evaluated in the processed document's scope, as a developer's console asks: the document's data,
//! inputs and frontmatter, the `md.` functions at the render's time, F024 at its place in the expression, and nothing
//! of the document changed.

use compmd::context::{RenderContext, parse_time};
use compmd::expand::Options;
use compmd::processor::Expanded;
use compmd::{Ambient, ParseOptions, Processor};
use serde_json::json;
use std::collections::HashMap;
use std::rc::Rc;

const DOCUMENT: &str = "---\ntitle: Sales\nmd:\n  inputs:\n    region: eu | us = eu\n  data:\n    - from: ./sales.json\n      as: sales\n---\n\n# {{md.doc.title}}\n";

const SALES: &str = r#"[{"region": "eu", "amount": 120}, {"region": "us", "amount": 80}, {"region": "eu", "amount": 30}]"#;

/// The document, processed with `inputs` on 2026-10-21.
fn processed(inputs: &serde_json::Value) -> Processor<Expanded> {
    let files = HashMap::from([("sales.json".to_string(), SALES.to_string())]);
    let context = RenderContext {
        now: parse_time("2026-10-21T09:00:00Z").expect("a time"),
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: inputs.as_object().cloned().unwrap_or_default(),
    };
    Processor::new("sales.comp.md", DOCUMENT)
        .parse(&ParseOptions::default())
        .resolve(files, (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&Options::default())
}

/// The value of `expression` in the document, which must have one.
fn value(processor: &Processor<Expanded>, expression: &str) -> serde_json::Value {
    processor
        .evaluate(expression)
        .unwrap_or_else(|error| panic!("{expression}: {}", error.message))
}

#[test]
fn gives_the_documents_frontmatter_data_and_inputs_with_this_renders_values() {
    let processor = processed(&json!({ "region": "us" }));
    assert_eq!(value(&processor, "md.doc.title"), json!("Sales"));
    assert_eq!(value(&processor, "md.input.region"), json!("us"));
    assert_eq!(value(&processor, "md.data.sales.size()"), json!(3));
    assert_eq!(
        value(
            &processor,
            "md.data.sales.filter(s, s.region == md.input.region).map(s, s.amount)"
        ),
        json!([80])
    );
    assert_eq!(
        value(&processor, "md.data.sales[0]"),
        json!({ "region": "eu", "amount": 120 })
    );
}

#[test]
fn calls_the_md_functions_at_the_renders_time_and_in_the_documents_language() {
    let processor = processed(&json!({}));
    assert_eq!(value(&processor, "md.today()"), json!("2026-10-21"));
    assert_eq!(
        value(&processor, "md.sum(md.data.sales.map(s, s.amount))"),
        json!(230)
    );
    assert_eq!(
        value(&processor, "[md.lang, md.tz, md.version]"),
        json!(["en", "UTC", "1"])
    );
}

#[test]
fn reports_f024_at_its_place_in_the_expression_for_each_kind_of_error() {
    let processor = processed(&json!({}));
    let error = |expression: &str| {
        let diagnostic = processor.evaluate(expression).expect_err(expression);
        let start = diagnostic.range.start;
        (
            diagnostic.code.as_str(),
            start.line,
            start.column,
            diagnostic.message,
        )
    };
    // A syntax error, an unknown field (with the nearest known one), and an evaluation error on the second line.
    let (code, line, column, _) = error("md.doc.title +");
    assert_eq!((code, line), ("F024", 1));
    assert!(column > 13);
    let (_, line, column, message) = error("md.doc.titel");
    assert_eq!((line, column), (1, 8));
    assert!(message.contains("title"), "{message}");
    let (code, line, column, _) = error("[1,\n  1 / 0]");
    assert_eq!((code, line, column), ("F024", 2, 5));
}

#[test]
fn sees_only_the_documents_top_level_scope_and_changes_nothing() {
    let processor = processed(&json!({}));
    let before = serde_json::to_value(processor.expanded()).expect("the tree serializes");
    let diagnostics = processor.diagnostics().to_vec();
    assert!(
        processor.evaluate("md.loop.index").is_err(),
        "no `each` around it"
    );
    assert!(processor.evaluate("item").is_err(), "no `as` name");
    assert_eq!(
        serde_json::to_value(processor.expanded()).expect("the tree serializes"),
        before
    );
    assert_eq!(processor.diagnostics(), diagnostics);
}
