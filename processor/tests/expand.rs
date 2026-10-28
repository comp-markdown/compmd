//! Stages 4a and 4b beyond the conformance set: data from packs, URLs and hosts, inputs and their defaults, every
//! mdast node, host components with slots, origins, and what fails, with the diagnostics as `(code, line, column)`.

use compmd::context::Context;
use compmd::expand;
use compmd::host::{Fetched, LoadedRelease, Needed, Release};
use compmd::{Ambient, ParseOptions, Processor};
use semver::Version;
use serde_json::{Value, json};
use std::collections::HashMap;

/// What a render is given besides the document.
#[derive(Default)]
struct Setup {
    files: Vec<(&'static str, &'static str)>,
    /// The files of one pack, `@acme/kit` 1.0.0.
    kit: Vec<(&'static str, &'static str)>,
    web: Vec<(&'static str, &'static str, Option<&'static str>)>,
    host: Value,
    inputs: Value,
    allowed: Vec<&'static str>,
    options: expand::Options,
}

/// What a render gave: the tree as JSON, the diagnostics of every stage, and the sources the host provides.
struct Rendered {
    tree: Value,
    found: Vec<(String, usize, usize)>,
    files: Vec<Option<String>>,
    sources: Needed,
}

fn render(input: &str, setup: Setup) -> Rendered {
    let mut files: HashMap<String, String> = setup
        .files
        .iter()
        .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
        .collect();
    files.insert("input.comp.md".into(), input.to_string());
    let kit = LoadedRelease {
        release: Release {
            version: Version::new(1, 0, 0),
            integrity: None,
        },
        files: setup
            .kit
            .iter()
            .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
            .collect(),
    };
    let packs = HashMap::from([("@acme/kit".to_string(), vec![kit])]);
    let web: HashMap<String, Fetched> = setup
        .web
        .iter()
        .map(|(url, text, content_type)| {
            let fetched = Fetched {
                text: (*text).to_string(),
                content_type: content_type.map(str::to_string),
            };
            ((*url).to_string(), fetched)
        })
        .collect();
    let host: HashMap<String, Value> = match setup.host {
        Value::Object(sources) => sources.into_iter().collect(),
        _ => HashMap::new(),
    };
    let allowed: Vec<String> = setup
        .allowed
        .iter()
        .map(|origin| (*origin).to_string())
        .collect();
    let context = Context {
        now: 1_792_576_800 * 1_000_000_000,
        tz: "UTC".into(),
        time: &(),
        inputs: setup.inputs.as_object().cloned().unwrap_or_default(),
    };
    let validated = Processor::new("input.comp.md", input)
        .parse(&ParseOptions::default())
        .resolve(&files, &packs, &Ambient::default())
        .validate(&allowed);
    let needed = validated.sources(&context);
    let expanded = validated.load(&context, &web, &host).expand(&setup.options);
    let diagnostics = expanded.diagnostics();
    let mut found: Vec<(String, usize, usize)> = diagnostics
        .iter()
        .map(|d| (d.code.to_string(), d.range.start.line, d.range.start.column))
        .collect();
    found.sort();
    Rendered {
        tree: serde_json::to_value(expanded.expanded()).expect("the tree serializes"),
        found,
        files: diagnostics.iter().map(|d| d.file.clone()).collect(),
        sources: needed,
    }
}

fn at(code: &str, line: usize, column: usize) -> (String, usize, usize) {
    (code.to_string(), line, column)
}

/// The text of a tree, its text nodes in order, joined by `|` between blocks.
fn text(tree: &Value) -> String {
    fn inline(node: &Value, out: &mut String) {
        if let Some(value) = node["value"].as_str().filter(|_| node["type"] == "text") {
            out.push_str(value);
        }
        for child in node["children"].as_array().into_iter().flatten() {
            inline(child, out);
        }
    }
    tree["children"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|block| {
            let mut out = String::new();
            inline(block, &mut out);
            out
        })
        .collect::<Vec<_>>()
        .join("|")
}

/// The types of a tree's top-level blocks.
fn types(tree: &Value) -> Vec<String> {
    tree["children"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|block| block["type"].as_str().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn every_mdast_node_keeps_its_shape() {
    let input = "---\nk: guide\nmd:\n  syntax: [wiki-links]\n---\n\n> [!NOTE]\n> A *quote* with ~~old~~ **text**,\\\n> `code`, $x$ and <b>html</b>.\n\n```rust\nfn main() {}\n```\n\n$$\nx^2\n$$\n\n| A | B |\n| - | - |\n| [[Page]] | [ref][r] |\n\n---\n\n![Logo][r] and a note.[^n]\n\n[^n]: The note.\n\n[r]: https://example.com/{{md.doc.k}}\n";
    let rendered = render(input, Setup::default());
    assert_eq!(rendered.found, []);
    assert_eq!(
        types(&rendered.tree),
        [
            "blockquote",
            "code",
            "math",
            "table",
            "thematicBreak",
            "paragraph",
            "footnoteDefinition",
            "definition"
        ]
    );
    let quote = &rendered.tree["children"][0];
    assert_eq!(quote["alert"], "note");
    let line = &quote["children"][0]["children"];
    let kinds: Vec<&str> = line
        .as_array()
        .expect("inline nodes")
        .iter()
        .filter_map(|node| node["type"].as_str())
        .collect();
    assert_eq!(
        kinds,
        [
            "text",
            "emphasis",
            "text",
            "delete",
            "text",
            "strong",
            "text",
            "break",
            "inlineCode",
            "text",
            "inlineMath",
            "text",
            "html",
            "text",
            "html",
            "text"
        ]
    );
    let row = &rendered.tree["children"][3]["children"][1]["children"];
    assert_eq!(row[0]["children"][0]["type"], "wikiLink");
    assert_eq!(row[1]["children"][0]["type"], "linkReference");
    let paragraph = &rendered.tree["children"][5]["children"];
    assert_eq!(paragraph[0]["type"], "imageReference");
    assert_eq!(paragraph[2]["type"], "footnoteReference");
    assert_eq!(
        rendered.tree["children"][7]["url"],
        "https://example.com/guide"
    );
    assert_eq!(rendered.tree["children"][7].get("urlParts"), None);
}

#[test]
fn origins_cover_every_node_and_what_recovery_closed() {
    let input = "```compmd component=card\nslots: [default, foot]\n---\n{{md.slot}}\n\n{{md.slot.foot}}\n```\n\n# Title\n\n- item with `code`\n\n> quote\n\n::: card\nBody.\n:: #foot\nFoot.\n";
    let options = expand::Options {
        origins: true,
        ..expand::Options::default()
    };
    let rendered = render(
        input,
        Setup {
            options,
            ..Setup::default()
        },
    );
    assert_eq!(rendered.found, [at("F004", 15, 1)]);
    let card = &rendered.tree["children"][3];
    assert_eq!(card["name"], "box");
    assert_eq!(card["origin"]["start"]["line"], 15);
    assert_eq!(card["origin"]["end"]["line"], 18);
    assert_eq!(card["children"][1]["origin"]["start"]["line"], 18);
    let item = &rendered.tree["children"][1]["children"][0];
    assert_eq!(item["origin"]["start"]["line"], 11);
    assert_eq!(
        item["children"][0]["children"][1]["origin"]["start"]["column"],
        13
    );
}

#[test]
fn urls_take_their_values_and_never_their_scheme() {
    let input = "---\nlogo: ./logo.png\nbad: \"javascript:alert(1)\"\nlist: [a]\nsite: https://a.org\n---\n\n![Logo]({{md.doc.logo}}) ![Trap]({{md.doc.bad}})\n\n[a]({{nope}}) [b](https://a.org/{{md.doc.list}}) [c]({{md.doc.site}}/x)\n\n[t]: {{md.doc.bad}}\n";
    let rendered = render(input, Setup::default());
    assert_eq!(
        rendered.found,
        [at("F014", 10, 7), at("W109", 8, 26), at("W109", 12, 1)]
    );
    let images = &rendered.tree["children"][0]["children"];
    assert_eq!(images[0]["url"], "./logo.png");
    assert_eq!(images[1], json!({ "type": "text", "value": " Trap" }));
    let links = &rendered.tree["children"][1]["children"];
    assert_eq!(links[0]["url"], "{{nope}}");
    assert_eq!(links[2]["url"], "https://a.org/{{md.doc.list}}");
    assert_eq!(links[4]["url"], "https://a.org/x");
    assert_eq!(rendered.tree["children"][2]["url"], "");
}

#[test]
fn paths_that_fail_when_evaluated_are_shown_as_written() {
    let input = "---\nmd:\n  data:\n    - from: ./team.yaml\n      as: team\n---\n\n{{md.data.team.missing}} and {{md.slot}}.\n\n{{md.data.team.lead}}\n";
    let rendered = render(
        input,
        Setup {
            files: vec![("team.yaml", "lead: Ana\n")],
            ..Setup::default()
        },
    );
    assert_eq!(rendered.found, [at("F014", 8, 32), at("F024", 8, 3)]);
    assert_eq!(
        text(&rendered.tree),
        "{{md.data.team.missing}} and {{md.slot}}.|Ana"
    );
}

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

#[test]
fn inputs_take_given_values_that_fit_or_their_defaults() {
    let input = "---\nname: Ana\nmd:\n  inputs:\n    n: number = 2\n    on: boolean = true\n    q: string = \"Hi {{md.doc.name}}\"\n    tags: array = [\"a\"]\n    flag: boolean = (1 / 0 == 1)\n    long: {type: number, default: 3}\n    none: string?\n    day: date = (md.today())\n    word: string = plain\n    when: date = 2026-10-01\n    odd: whatever = x\n---\n\n{{md.input.n}} {{md.input.on}} {{md.input.q}} {{md.input.long}} [{{md.input.none}}] {{md.input.day}} {{md.input.word}} {{md.input.when}}\n";
    let rendered = render(
        input,
        Setup {
            inputs: json!({ "n": "two", "on": false, "when": "2026-13-01" }),
            ..Setup::default()
        },
    );
    assert_eq!(rendered.found, [at("F024", 9, 5)]);
    assert_eq!(
        text(&rendered.tree),
        "2 false Hi Ana 3 [] 2026-10-21 plain 2026-10-01"
    );
    let inputs = &rendered.tree["inputs"];
    assert_eq!(inputs["tags"], json!({ "type": "array", "value": ["a"] }));
    assert_eq!(inputs["flag"]["value"], Value::Null);
    assert_eq!(inputs["n"], json!({ "type": "number", "value": 2 }));
    assert_eq!(inputs["odd"], json!({ "type": "whatever", "value": "x" }));
}

#[test]
fn each_source_is_needed_once_at_its_first_place() {
    let input = "---\nmd:\n  data:\n    - from: https://api.example.com/b.json\n      as: b\n    - from: host:tasks\n      as: tasks\n    - from: https://api.example.com/a.json\n      as: a\n    - from: https://api.example.com/b.json\n      as: again\n    - from: host:tasks\n      as: same\n---\nText.\n";
    let rendered = render(
        input,
        Setup {
            allowed: vec!["https://api.example.com"],
            ..Setup::default()
        },
    );
    assert_eq!(
        rendered.sources,
        Needed {
            remote: vec![
                "https://api.example.com/b.json".into(),
                "https://api.example.com/a.json".into(),
            ],
            host: vec!["tasks".into()],
        }
    );
}

#[test]
fn data_comes_from_packs_urls_and_hosts() {
    let input = "---\nmd:\n  import:\n    - from: \"@acme/kit@1\"\n      names: [stat]\n  inputs:\n    region: eu | us = eu\n  data:\n    - from: \"@acme/kit/icons.json\"\n      as: icons\n    - from: \"@other/kit/x.json\"\n      as: other\n    - from: https://api.example.com/s?r={{md.input.region}}\n      as: remote\n    - from: https://api.example.com/plain\n      as: plain\n    - from: https://api.example.com/gone\n      as: gone\n      fallback: ./cached.json\n    - from: http://old.example.com/x.json\n      as: old\n    - from: host:tasks\n      as: tasks\n    - from: host:none\n      as: none\n    - from: ../outside.json\n      as: outside\n    - from: ./notes.txt\n      as: notes\n---\n\n{{md.data.icons.star}} {{md.data.remote.n}} {{md.data.gone.n}} {{md.data.tasks.n}} :stat[s]\n";
    let kit = vec![
        (
            "package.json",
            "{\"name\": \"@acme/kit\", \"version\": \"1.0.0\"}",
        ),
        (
            "index.comp.md",
            "---\nmd:\n  data:\n    - from: ./count.csv\n      as: count\n  export:\n    names: [stat]\n---\n\n```compmd component=stat\n---\n{{md.data.count}}\n```\n",
        ),
        ("icons.json", "{\"star\": \"*\"}"),
    ];
    let rendered = render(
        input,
        Setup {
            files: vec![("cached.json", "{\"n\": \"cached\"}"), ("notes.txt", "x")],
            kit,
            web: vec![
                (
                    "https://api.example.com/s?r=eu",
                    "n: 5\n",
                    Some("application/yaml"),
                ),
                ("https://api.example.com/plain", "x", None),
            ],
            host: json!({ "tasks": { "n": "host" } }),
            allowed: vec!["https://api.example.com", "http://old.example.com"],
            ..Setup::default()
        },
    );
    assert_eq!(
        rendered.found,
        [
            at("F026", 4, 13),
            at("F026", 11, 13),
            at("F026", 15, 13),
            at("F026", 20, 13),
            at("F026", 24, 13),
            at("F026", 26, 13),
            at("F026", 28, 13),
        ]
    );
    assert_eq!(
        rendered.sources,
        Needed {
            remote: vec![
                "https://api.example.com/s?r=eu".into(),
                "https://api.example.com/plain".into(),
                "https://api.example.com/gone".into(),
            ],
            host: vec!["tasks".into(), "none".into()],
        }
    );
    assert_eq!(text(&rendered.tree), "* 5 cached host s");
    assert!(
        rendered
            .files
            .contains(&Some("@acme/kit@1.0.0/index.comp.md".into()))
    );
}

#[test]
fn a_host_without_remote_or_host_data_loads_fallbacks() {
    let input = "---\nmd:\n  data:\n    - from: https://a.org/x.json\n      as: x\n      fallback: ./x.json\n    - from: host:tasks\n      as: tasks\n---\n\n{{md.data.x.n}}\n";
    let files: HashMap<String, String> = HashMap::from([
        ("input.comp.md".to_string(), input.to_string()),
        ("x.json".to_string(), "{\"n\": \"local\"}".to_string()),
    ]);
    let context = Context {
        now: 0,
        tz: "UTC".into(),
        time: &(),
        inputs: serde_json::Map::new(),
    };
    let expanded = Processor::new("input.comp.md", input)
        .parse(&ParseOptions::default())
        .resolve(&files, &(), &Ambient::default())
        .validate(&["https://a.org".into()])
        .load(&context, &(), &())
        .expand(&expand::Options::default());
    let codes: Vec<String> = expanded
        .diagnostics()
        .iter()
        .map(|d| d.code.to_string())
        .collect();
    assert_eq!(codes, ["F026"]);
    let tree = serde_json::to_value(expanded.expanded()).expect("the tree serializes");
    assert_eq!(text(&tree), "local");
}

#[test]
fn a_reload_takes_newly_fetched_data() {
    let input =
        "---\nmd:\n  data:\n    - from: host:tasks\n      as: tasks\n---\n\n{{md.data.tasks.n}}\n";
    let files = HashMap::from([("input.comp.md".to_string(), input.to_string())]);
    let context = Context {
        now: 0,
        tz: "UTC".into(),
        time: &(),
        inputs: serde_json::Map::new(),
    };
    let first = Processor::new("input.comp.md", input)
        .parse(&ParseOptions::default())
        .resolve(&files, &(), &Ambient::default())
        .validate(&[])
        .load(&context, &(), &())
        .expand(&expand::Options::default());
    let codes: Vec<String> = first
        .diagnostics()
        .iter()
        .map(|d| d.code.to_string())
        .collect();
    assert_eq!(
        codes,
        ["F026", "F024"],
        "no `host:tasks` yet, so `.n` of null"
    );
    let fetched = HashMap::from([("tasks".to_string(), json!({ "n": "fresh" }))]);
    let second = first.reload(&context, &(), &fetched);
    assert_eq!(second.diagnostics(), []);
    let tree = serde_json::to_value(second.expanded()).expect("the tree serializes");
    assert_eq!(text(&tree), "fresh");
}

#[test]
fn the_limit_stops_templates_and_invocations() {
    let input = "```compmd component=pair\n---\nA\n\nB\n```\n\n::: each {in=([1, 2, 3]) as=i}\n:: pair[P]\n::: /each\n\n```compmd component=wide\nparams:\n  rows: array\n  as: name\nslots:\n  default: {each: rows, as: as}\n---\n{{md.slot}}\n```\n\n::: wide {rows=([1, 2, 3]) as=r}\nRow {{r}}\n::: /wide\n";
    let limited = |limit| {
        render(
            input,
            Setup {
                options: expand::Options {
                    limit,
                    ..expand::Options::default()
                },
                ..Setup::default()
            },
        )
    };
    let rendered = limited(4);
    assert_eq!(rendered.found, [at("F034", 9, 1)]);
    assert_eq!(text(&rendered.tree), "P|Row {{r}}");
    let unlimited = limited(1000);
    assert_eq!(unlimited.found, []);
    assert_eq!(text(&unlimited.tree), "AB|AB|AB|Row 1Row 2Row 3");
}
