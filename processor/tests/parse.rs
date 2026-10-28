//! Stage 1 beyond the conformance set: the mdast nodes CompMD keeps as they are, the base syntax's edge cases,
//! recovery in nested parents, definitions, and documents read as plain Markdown.

use compmd::processor::Parsed;
use compmd::{ParseOptions, Processor};
use serde_json::{Value, json};

fn parse(text: &str) -> Processor<Parsed> {
    Processor::new("input.comp.md", text).parse(&ParseOptions::default())
}

/// The tree of `text`, without positions.
fn tree(text: &str) -> Value {
    without_positions(&serde_json::to_value(parse(text).tree()).unwrap())
}

/// The first block of `text`'s tree.
fn first(text: &str) -> Value {
    tree(text)["children"][0].clone()
}

/// The diagnostics of `text`: code, line and column.
fn diagnostics(text: &str) -> Vec<(String, usize, usize)> {
    parse(text)
        .diagnostics()
        .iter()
        .map(|d| (d.code.to_string(), d.range.start.line, d.range.start.column))
        .collect()
}

fn without_positions(value: &Value) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(without_positions).collect()),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .filter(|(key, _)| *key != "position" && *key != "closePosition")
                .map(|(key, value)| (key.clone(), without_positions(value)))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn text_node(value: &str) -> Value {
    json!({ "type": "text", "value": value })
}

#[test]
fn mdast_inline_nodes_keep_their_fields() {
    let paragraph = first(
        "<b>a</b> **b** ~~c~~ d  \ne ![i](./{{md.doc.logo}}.png \"t\") ![r][x] [f][] [g][x]\n\n[x]: /x\n[f]: /f\n",
    );
    assert_eq!(
        paragraph["children"],
        json!([
            { "type": "html", "value": "<b>" },
            text_node("a"),
            { "type": "html", "value": "</b>" },
            text_node(" "),
            { "type": "strong", "children": [text_node("b")] },
            text_node(" "),
            { "type": "delete", "children": [text_node("c")] },
            text_node(" d"),
            { "type": "break" },
            text_node("e "),
            { "type": "image", "alt": "i", "url": "./{{md.doc.logo}}.png", "title": "t", "urlParts": [
                text_node("./"),
                { "type": "interpolation", "path": "md.doc.logo", "block": false },
                text_node(".png"),
            ] },
            text_node(" "),
            { "type": "imageReference", "alt": "r", "referenceType": "full", "identifier": "x", "label": "x" },
            text_node(" "),
            { "type": "linkReference", "children": [text_node("f")], "referenceType": "collapsed", "identifier": "f", "label": "f" },
            text_node(" "),
            { "type": "linkReference", "children": [text_node("g")], "referenceType": "full", "identifier": "x", "label": "x" },
        ])
    );
}

#[test]
fn mdast_blocks_keep_their_fields() {
    let blocks = tree(
        "***\n\n| a | b | c | d |\n|:--|--:|:-:|---|\n| 1 | 2 | 3 | 4 |\n\n[x]: ./{{md.doc.slug}} \"t\"\n",
    );
    assert_eq!(blocks["children"][0], json!({ "type": "thematicBreak" }));
    assert_eq!(
        blocks["children"][1]["align"],
        json!(["left", "right", "center", null])
    );
    assert_eq!(
        blocks["children"][1]["children"][1]["children"][3],
        json!({ "type": "tableCell", "children": [text_node("4")] })
    );
    assert_eq!(
        blocks["children"][2],
        json!({ "type": "definition", "identifier": "x", "label": "x", "url": "./{{md.doc.slug}}", "title": "t", "urlParts": [
            text_node("./"),
            { "type": "interpolation", "path": "md.doc.slug", "block": false },
        ] })
    );
}

#[test]
fn math_fences_are_display_math() {
    assert_eq!(
        first("```math\nx^2\n```\n"),
        json!({ "type": "math", "meta": null, "value": "x^2" })
    );
    assert_eq!(
        first("$$\nx\n$$\n"),
        json!({ "type": "math", "meta": null, "value": "x" })
    );
}

#[test]
fn alerts_need_their_marker_alone_on_the_first_line() {
    assert_eq!(
        first("> [!NOTE]\n> Read *this*.\n"),
        json!({ "type": "blockquote", "alert": "note", "children": [
            { "type": "paragraph", "children": [text_node("Read "), { "type": "emphasis", "children": [text_node("this")] }, text_node(".")] },
        ] })
    );
    assert_eq!(
        first("> [!TIP]\n"),
        json!({ "type": "blockquote", "alert": "tip", "children": [] })
    );
    assert_eq!(
        first("> [!WARNING]\n> *x*\n"),
        json!({ "type": "blockquote", "alert": "warning", "children": [
            { "type": "paragraph", "children": [{ "type": "emphasis", "children": [text_node("x")] }] },
        ] })
    );
    assert_eq!(first("> - [!NOTE]\n")["alert"], Value::Null);
    assert_eq!(first("> *[!NOTE]*\n")["alert"], Value::Null);
    assert_eq!(first("> [!NOTE] Not alone.\n")["alert"], Value::Null);
}

#[test]
fn a_closer_in_a_nested_list_closes_its_container_at_the_parents_end() {
    let text = "::: card\n- item\n\n  ::: /card\n";
    assert_eq!(diagnostics(text), [("F032".into(), 4, 3)]);
    let card = first(text);
    assert_eq!(card["autoClosed"], json!(true));
    assert_eq!(card["children"][0]["type"], "list");
}

#[test]
fn a_closer_two_lists_down_closes_a_container_two_parents_up() {
    let text = "::: card\n- a\n  - b\n\n    ::: /card\n";
    assert_eq!(diagnostics(text), [("F032".into(), 5, 5)]);
    assert_eq!(first(text)["autoClosed"], json!(true));
}

#[test]
fn a_leaf_inside_a_container_takes_its_continuation_lines() {
    let text = "::: box\n:: chart[Sales]\n::: + rows=(md.data.sales) rows=x\n::: /box\n";
    assert_eq!(diagnostics(text), [("F017".into(), 3, 28)]);
    assert_eq!(
        first(text)["children"][0]["params"],
        json!([{ "key": "rows", "breakpoint": null, "kind": "expr", "value": "md.data.sales" }])
    );
}

#[test]
fn a_continuation_line_that_breaks_marks_its_owner_recovered() {
    let text = "::: box {gap=s}\n::: + title=\"x\n::: /box\n\n:: chart[x]\n::: + a=(b\n";
    assert_eq!(
        diagnostics(text),
        [("F011".into(), 2, 13), ("F011".into(), 6, 9)]
    );
    let blocks = tree(text);
    assert_eq!(blocks["children"][0]["recovered"], json!(true));
    assert_eq!(
        blocks["children"][0]["params"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(blocks["children"][1]["recovered"], json!(true));
}

#[test]
fn an_inline_component_whose_params_break_keeps_none() {
    let text = "A :badge[x]{a=[1,]} b.";
    assert_eq!(diagnostics(text), [("F011".into(), 1, 15)]);
    assert_eq!(
        first(text)["children"],
        json!([
            text_node("A "),
            { "type": "componentInline", "name": "badge", "fallback": [text_node("x")], "params": [], "recovered": true },
            text_node("{a=[1,]} b."),
        ])
    );
}

#[test]
fn a_fallback_that_is_not_inline_content_is_empty() {
    assert_eq!(first(":: chart[# Sales]\n")["fallback"], json!([]));
}

#[test]
fn definitions_need_a_valid_name_and_a_param_list_that_ends_the_line() {
    assert_eq!(first("```js component=1x\nx\n```\n")["type"], "code");
    assert_eq!(first("```js title=x\nx\n```\n")["type"], "code");
    let text = "```compmd component=card {.card} trailing\nslots: x\n---\nBody.\n```\n";
    assert_eq!(diagnostics(text), [("F011".into(), 1, 34)]);
    let card = first(text);
    assert_eq!(
        card["params"],
        json!([{ "key": null, "breakpoint": null, "kind": "class", "value": "card" }])
    );
    assert_eq!(
        card["signature"],
        json!({ "params": [], "slots": null, "headings": "relative" })
    );
    assert_eq!(
        card["children"],
        json!([{ "type": "paragraph", "children": [text_node("Body.")] }])
    );
}

#[test]
fn signatures_read_short_and_long_forms() {
    let signature = first(
        "```compmd component=chart signature\nparams:\n  kind: bar | line\t=\tbar\n  rows:\n    type: array?\n    default: []\n    doc: The rows\n  pick: input<date>\n  size: number = (\nslots: [default, 3]\nheadings: absolute\n```\n",
    )["signature"]
        .clone();
    assert_eq!(
        signature,
        json!({
            "params": [
                { "name": "kind", "type": { "enum": ["bar", "line"] }, "optional": false, "default": { "kind": "bare", "value": "bar" }, "doc": null },
                { "name": "rows", "type": "array", "optional": true, "default": { "kind": "yaml", "value": [] }, "doc": "The rows" },
                { "name": "pick", "type": { "input": "date" }, "optional": false, "default": null, "doc": null },
                { "name": "size", "type": "number", "optional": false, "default": null, "doc": null },
            ],
            "slots": ["default", "3"],
            "headings": "absolute",
        })
    );
    assert_eq!(
        first("```compmd component=x signature\na: [\n```\n")["signature"],
        Value::Null
    );
    assert_eq!(
        first("```compmd component=x signature {a=1}\n```\n")["signature"]["slots"],
        Value::Null
    );
}

#[test]
fn a_definition_with_an_empty_body_or_none() {
    assert_eq!(
        first("```compmd component=x\n---\n```\n")["children"],
        json!([])
    );
    let header_only = first("```compmd component=x\nslots: [default]\n```\n");
    assert_eq!(header_only["children"], json!([]));
    assert_eq!(header_only["signature"]["slots"], json!(["default"]));
}

#[test]
fn plain_documents_have_no_compmd() {
    let text = "---\nmd:\n  version: 2\n---\n\n> [!NOTE]\n\n<div>\n:: x\n\n[a](./{{x}})\n\n```math\nx\n```\n";
    assert_eq!(diagnostics(text), [("F001".into(), 1, 1)]);
    let blocks = tree(text);
    assert_eq!(
        blocks["children"][0],
        json!({ "type": "yaml", "value": "md:\n  version: 2" })
    );
    assert_eq!(blocks["children"][1]["alert"], Value::Null);
    assert_eq!(
        blocks["children"][3]["children"][0]["urlParts"],
        Value::Null
    );
    assert_eq!(blocks["children"][4]["type"], "code");
}

#[test]
fn parsing_never_fails_on_odd_input() {
    for text in [
        "",
        "\n",
        "::",
        ":::",
        ":: ",
        "::: /",
        "{{",
        ":a[",
        "`",
        "---",
        "\u{feff}",
        "> ::: a\n> ::: /b",
    ] {
        assert!(serde_json::to_value(parse(text).tree()).is_ok(), "{text:?}");
    }
}
