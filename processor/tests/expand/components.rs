//! Components as they expand: slots, inline components, host components with their slots and inputs, what can't
//! render, cycles, `each`, `if` and `value`, and breakpoint values.

use super::*;
use crate::common::at;
use serde_json::json;

#[test]
fn slots_are_placed_inline_and_a_body_follows_a_component_without_slots() {
    let input = "```compmd component=say\nslots: [default]\n---\nThey said: {{md.slot}}\n```\n\n```compmd component=line\n---\nA line.\n```\n\n::: say\n*hello*\n::: /say\n\n::: line\nStray.\n::: /line\n";
    let rendered = render(input, Setup::default());
    assert_eq!(rendered.found, [at("F018", 16, 1)]);
    assert_eq!(text(&rendered.tree), "They said: hello|A line.|Stray.");
    let said = &rendered.tree["children"][0]["children"][0]["children"];
    assert_eq!(said[1]["type"], "emphasis");
}

#[test]
fn inline_components_give_their_paragraph_or_their_fallback() {
    let input = "```compmd component=badge {padding=s}\nparams:\n  tone: info | warn = info\n---\n**{{tone}}**\n```\n\n```compmd component=two\n---\nOne.\n\nTwo.\n```\n\nA :badge[beta]{tone=warn} and :two[both] and :nope[gone].\n";
    let rendered = render(input, Setup::default());
    assert_eq!(rendered.found, [at("F002", 15, 47)]);
    assert_eq!(text(&rendered.tree), "A warn and both and gone.");
}

#[test]
fn host_components_get_their_slots_template_slots_and_bound_inputs() {
    let input = "---\nmd:\n  inputs:\n    region: all | eu = all\n---\n\n```jsx component=board {.board padding=s}\nexport default () => null;\n```\n\n```compmd component=board signature\nparams:\n  rows: array\n  as: name\n  selected: input<enum>\n  meta: object?\nslots:\n  default: {each: rows, as: as}\n  side: {}\n  lane: {each: rows, as: as}\n```\n\n::: board {rows=([\"a\", \"b\"]) as=r selected=region meta={\"k\": 1} padding=m}\nCard {{r}}, {{md.loop.number}}\n:: #side\nSide.\n:: #lane\nLane {{r}}\n::: /board\n\n:: board[Board] {rows=([]) as=r selected=region}\n\nIn text :board[b]{rows=([]) as=r selected=region}.\n";
    let rendered = render(
        input,
        Setup {
            inputs: json!({ "region": "eu" }),
            ..Setup::default()
        },
    );
    assert_eq!(rendered.found, []);
    let board = &rendered.tree["children"][0];
    assert_eq!(board["type"], "componentContainer");
    assert_eq!(
        board["params"],
        json!([
            { "key": "rows", "breakpoint": null, "value": ["a", "b"] },
            { "key": "as", "breakpoint": null, "value": "r" },
            { "key": "selected", "breakpoint": null, "value": "eu", "input": "region" },
            { "key": "meta", "breakpoint": null, "value": { "k": 1 } },
            { "key": "padding", "breakpoint": null, "value": "m" },
            { "key": "class", "breakpoint": null, "value": "board" },
        ])
    );
    assert_eq!(board["children"], json!([]));
    assert_eq!(board["instances"][1]["value"], "b");
    assert_eq!(
        board["instances"][1]["children"][0]["children"][0]["value"],
        "Card b, 2"
    );
    assert_eq!(
        board["slots"][0]["children"][0]["children"][0]["value"],
        "Side."
    );
    assert_eq!(board["slots"][1]["instances"][0]["value"], "a");
    assert_eq!(rendered.tree["children"][1]["type"], "componentLeaf");
    assert_eq!(
        rendered.tree["children"][2]["children"][1]["type"],
        "componentInline"
    );
    assert_eq!(rendered.tree["inputs"]["region"]["value"], "eu");
}

#[test]
fn components_that_cant_render_give_their_fallback() {
    let input = "```jsx component=old runtime=react@17\nexport default () => null;\n```\n\n```compmd component=old signature\nparams: {}\nslots: [default]\n```\n\n```compmd component=loop\n---\n:: loop[Again]\n```\n\n```compmd component=need\nparams:\n  n: number\n---\nN {{n}}\n```\n\n::: old\nOld body.\n::: /old\n\n:: loop[Looped]\n\n:: need[Needed] {n=(1 / 0)}\n";
    let rendered = render(input, Setup::default());
    assert_eq!(
        rendered.found,
        [at("F019", 12, 4), at("F024", 28, 23), at("W102", 22, 1)]
    );
    assert_eq!(text(&rendered.tree), "Old body.|Again|Needed");
}

/// Each interpolation in a quoted param reports at its own path.
#[test]
fn a_quoted_params_paths_report_where_each_is() {
    let input = "---\na: x\nm: {k: 1}\n---\n\n```compmd component=c\nparams:\n  t: string\n---\n{{t}}\n```\n\n:: c[C] {t=\"{{md.doc.a}} {{md.doc.m}}\"}\n";
    let rendered = render(input, Setup::default());
    assert_eq!(rendered.found, [at("F024", 13, 28)]);
}

/// The invocation F019 is reported at renders its fallback, whichever component the document invokes first.
#[test]
fn a_cycle_falls_back_where_it_is_reported() {
    let input = "```compmd component=a\n---\nA\n\n:: b[From a]\n```\n\n```compmd component=b\n---\nB\n\n:: a[From b]\n```\n\n:: b[Top]\n";
    let rendered = render(input, Setup::default());
    assert_eq!(rendered.found, [at("F019", 12, 4)]);
    assert_eq!(text(&rendered.tree), "BFrom b");
}

#[test]
fn each_if_and_value_fail_to_their_fallbacks() {
    let input = "---\nmd:\n  data:\n    - from: ./d.yaml\n      as: d\n---\n\n::: each {in=(md.data.d.map) as=x}\nT {{x}}\n:: #empty\nE\n::: /each\n\n::: each {in=(md.data.d.list)}\nNo name {{x}}\n::: /each\n\n::: each {in=(md.data.d.list) as=x join=list}\n1. {{x}}\n\n- {{x}}\n::: /each\n\n::: each {in=(md.data.d.list) as=x join=inline}\nA {{x}}\n\nB {{x}}\n::: /each\n\n::: if {when=(md.data.d.list)}\nYes.\n:: #else\nNo.\n::: /if\n\n:: value[Fallback] {of=(md.data.d.list)}\n";
    let rendered = render(
        input,
        Setup {
            files: vec![("d.yaml", "map: {a: 1}\nlist: [1, 2]\n")],
            ..Setup::default()
        },
    );
    assert_eq!(
        rendered.found,
        [
            at("F010", 14, 1),
            at("F014", 15, 11),
            at("F024", 8, 14),
            at("F025", 18, 1),
            at("F025", 24, 1),
        ]
    );
    assert_eq!(
        text(&rendered.tree),
        "T {{x}}|No name {{x}}|1|1|2|2|A 1|B 1|A 2|B 2|Yes.|No.|Fallback"
    );
}

#[test]
fn breakpoint_values_reach_expressions_in_templates() {
    let input = "```compmd component=tiles {width=(size == \"x\" ? string(1 / 0) : size)}\nparams:\n  size: string = \"4rem\"\n  items: array\nslots: [default]\n---\n::: each {in=(items) as=i}\n::: box {height=(size)}\n{{i}}\n::: /box\n::: /each\n```\n\n::: tiles {items=([1, 2]) md:size=8rem lg:size=x}\nIgnored.\n::: /tiles\n";
    let rendered = render(input, Setup::default());
    assert_eq!(rendered.found, [at("F024", 1, 58)]);
    let tiles = &rendered.tree["children"][0];
    assert_eq!(
        tiles["params"],
        json!([
            { "key": "width", "breakpoint": null, "value": "4rem" },
            { "key": "width", "breakpoint": "md", "value": "8rem" },
        ])
    );
    assert_eq!(
        tiles["children"][1]["params"],
        json!([
            { "key": "height", "breakpoint": null, "value": "4rem" },
            { "key": "height", "breakpoint": "md", "value": "8rem" },
            { "key": "height", "breakpoint": "lg", "value": "x" },
        ])
    );
}
