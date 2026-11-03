//! Data and inputs: inputs and their defaults, the sources a render needs, data from packs, URLs and hosts,
//! fallbacks, and a reload with data fetched later.

use super::*;
use crate::common::at;
use serde_json::json;

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
    // `none` has no default: F010, and the optional input is `null`. `whatever` isn't a type: F037, and the input
    // still takes its value.
    assert_eq!(
        rendered.found,
        [at("F010", 11, 5), at("F024", 9, 5), at("F037", 15, 10)]
    );
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
            packs: Vec::new(),
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
            packs: Vec::new(),
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
    let expanded = Processor::new("input.comp.md", input)
        .parse(&ParseOptions::default())
        .resolve(files, (), &Ambient::default())
        .validate(&["https://a.org".into()])
        .load(context(), (), ())
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
    let first = Processor::new("input.comp.md", input)
        .parse(&ParseOptions::default())
        .resolve(files, (), &Ambient::default())
        .validate(&[])
        .load(context(), (), ())
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
    let second = first.reload(context(), (), fetched);
    assert_eq!(second.diagnostics(), []);
    let tree = serde_json::to_value(second.expanded()).expect("the tree serializes");
    assert_eq!(text(&tree), "fresh");
}
