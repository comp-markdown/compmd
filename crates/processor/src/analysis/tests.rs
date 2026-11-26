//! The same cases as `@compmd/vite`'s `analysis.test.ts` had, so W104 says what it said there.

use super::*;
use crate::tree::Node;
use crate::{ParseOptions, Processor};

/// The signature of a fence whose YAML is `yaml`.
fn signature(yaml: &str) -> Signature {
    let text = format!("```compmd component=c signature\n{yaml}\n```\n");
    let parsed = Processor::new("a.comp.md", &text).parse(&ParseOptions::default());
    parsed
        .tree()
        .children
        .iter()
        .find_map(|node| match node {
            Node::ComponentSignature(fence) => fence.signature.clone(),
            _ => None,
        })
        .expect("a signature")
}

fn prop(name: &str) -> Prop {
    Prop {
        name: name.into(),
        ..Prop::default()
    }
}

fn typed(name: &str, kind: PropType) -> Prop {
    Prop {
        name: name.into(),
        kind: Some(kind),
        ..Prop::default()
    }
}

fn messages(found: Vec<Mismatch>) -> Vec<String> {
    found.into_iter().map(|mismatch| mismatch.message).collect()
}

#[test]
fn finds_a_declared_param_with_no_prop_a_prop_not_declared_and_a_slot_with_no_prop() {
    let props = Props {
        props: vec![prop("label"), prop("emphasis"), prop("children")],
        rest: false,
    };
    let found = mismatches(
        "Badge.jsx",
        &signature("params:\n  label: string\n  size: number\nslots: [default, side-panel]"),
        &props,
    );
    assert_eq!(
        found,
        [
            Mismatch {
                param: Some("size".into()),
                message: "the signature declares `size`, which Badge.jsx has no prop for (`size`)".into(),
            },
            Mismatch {
                param: None,
                message: "the signature declares the slot `side-panel`, which Badge.jsx has no prop for (`sidePanel`)".into(),
            },
            Mismatch { param: None, message: "Badge.jsx has the prop `emphasis`, which the signature doesn't declare".into() },
        ]
    );
}

#[test]
fn names_params_and_slots_as_the_bindings_do_and_takes_the_bindings_own_prop() {
    let props = Props {
        props: vec![prop("alignItems"), prop("sidePanel"), prop("compmd")],
        rest: false,
    };
    assert_eq!(
        mismatches(
            "Grid.tsx",
            &signature("params:\n  align-items: string\nslots: [side-panel]"),
            &props
        ),
        []
    );
    assert_eq!(prop_name("a--b"), "a-B");
    assert_eq!(prop_name("x-"), "x-");
    assert_eq!(prop_name("a-B"), "a-B");
    assert_eq!(prop_name("col-2"), "col2");
}

#[test]
fn checks_a_props_type_where_the_source_states_one() {
    let props = Props {
        props: vec![
            typed("count", PropType::String),
            typed(
                "tone",
                PropType::Enum {
                    members: vec!["info".into()],
                },
            ),
            typed("title", PropType::String),
            typed("pad", PropType::String),
            prop("bound"),
            typed("open", PropType::Boolean),
            typed(
                "size",
                PropType::Enum {
                    members: vec!["s".into(), "m".into()],
                },
            ),
            typed("body", PropType::Slot),
        ],
        rest: false,
    };
    let declared = signature(
        "params:\n  count: number\n  tone: info | warn\n  title: a | b\n  pad: space\n  bound: input<number>\n  open: boolean\n  size: s\n  body: string",
    );
    assert_eq!(
        messages(mismatches("Card.tsx", &declared, &props)),
        [
            "the signature declares `count` as a number, which Card.tsx's prop takes as a string",
            "the signature declares `tone` as one of info, warn, which Card.tsx's prop takes as one of info",
            "the signature declares `body` as a string, which Card.tsx's prop takes as a slot",
        ]
    );
}

#[test]
fn takes_any_declared_param_when_the_component_takes_props_it_doesnt_name() {
    assert_eq!(
        mismatches(
            "Box.jsx",
            &signature("params:\n  a: string\nslots: [default]"),
            &Props {
                props: Vec::new(),
                rest: true
            }
        ),
        []
    );
}

#[test]
fn no_reader_reads_nothing_and_props_are_json_as_hosts_write_them() {
    assert_eq!(().props(Language::Jsx, "export default () => null"), None);
    assert_eq!(
        std::rc::Rc::new(()).props(Language::Svelte, "<p>shared</p>"),
        None
    );
    let props: Props = serde_json::from_str(
        r#"{"props": [{"name": "tone", "type": {"kind": "enum", "members": ["info"]}}, {"name": "label"}]}"#,
    )
    .expect("props");
    assert_eq!(
        props,
        Props {
            props: vec![
                typed(
                    "tone",
                    PropType::Enum {
                        members: vec!["info".into()]
                    }
                ),
                prop("label")
            ],
            rest: false
        }
    );
    assert_eq!(
        serde_json::to_string(&props).expect("JSON"),
        r#"{"props":[{"name":"tone","type":{"kind":"enum","members":["info"]}},{"name":"label"}],"rest":false}"#
    );
    assert_eq!(written(&PropType::Number), "a number");
    assert_eq!(written(&PropType::Boolean), "a boolean");
}
