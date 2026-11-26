use compmd::analysis::{Prop, PropType, Props};

use super::{generated, updated};
use crate::features::code_actions::signatures::declarations::Declared;

fn prop(name: &str, kind: Option<PropType>) -> Prop {
    Prop {
        name: name.into(),
        kind,
        ..Prop::default()
    }
}

fn declared(props: Vec<Prop>) -> Declared {
    Declared::of(&Props { props, rest: false })
}

/// `body` brought up to date with `props`, the component taking others when `rest`, its slots now `slots`.
fn update(body: &str, props: Vec<Prop>, rest: bool, slots: &[&str]) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let slots: Vec<String> = slots.iter().map(|slot| (*slot).to_owned()).collect();
    updated(&lines, &declared(props), rest, &slots).join("\n")
}

#[test]
fn a_new_fence_lists_params_and_slots_or_none() {
    let both = declared(vec![
        prop("title", Some(PropType::String)),
        prop("children", Some(PropType::Slot)),
        prop("data", None),
    ]);
    assert_eq!(
        generated(&both),
        [
            "params:",
            "  title: string",
            "  # data: the component's code doesn't say its type",
            "slots: [default]"
        ]
    );
    assert_eq!(generated(&declared(Vec::new())), ["params: {}"]);
}

#[test]
fn the_writers_declarations_stay_and_props_come_and_go() {
    let body = "params:\n  # The card's heading.\n  title:\n    type: string\n    doc: The heading\n  gone: number\nprint: fallback";
    assert_eq!(
        update(
            body,
            vec![
                prop("title", Some(PropType::String)),
                prop("size", Some(PropType::Number))
            ],
            false,
            &[]
        ),
        "params:\n  # The card's heading.\n  title:\n    type: string\n    doc: The heading\n  size: number\nprint: fallback"
    );
    // A component that takes other props keeps what its signature declares.
    assert_eq!(
        update(body, vec![prop("title", Some(PropType::String))], true, &[]),
        "params:\n  # The card's heading.\n  title:\n    type: string\n    doc: The heading\n  gone: number\nprint: fallback"
    );
}

#[test]
fn slots_are_written_again_only_when_they_change() {
    let body = "params:\n  title: string\nslots: [default, footer]";
    let props = |more: &str| {
        vec![
            prop("title", Some(PropType::String)),
            prop("children", Some(PropType::Slot)),
            prop(more, Some(PropType::Slot)),
        ]
    };
    assert_eq!(
        update(body, props("footer"), false, &["default", "footer"]),
        body
    );
    assert_eq!(
        update(body, props("aside"), false, &["default", "footer"]),
        "params:\n  title: string\nslots: [default, aside]"
    );
    assert_eq!(
        update("params:\n  title: string", props("aside"), false, &[]),
        "params:\n  title: string\nslots: [default, aside]"
    );
}

#[test]
fn a_param_left_for_the_writer_stays_once_and_flow_params_are_written_again() {
    let body = "params:\n  title: string\n  # data: the component's code doesn't say its type";
    assert_eq!(
        update(
            body,
            vec![prop("title", Some(PropType::String)), prop("data", None)],
            false,
            &[]
        ),
        body
    );
    assert_eq!(
        update(
            "params: {title: string}\nheadings: absolute",
            vec![prop("title", Some(PropType::String))],
            false,
            &[]
        ),
        "params:\n  title: string\nheadings: absolute"
    );
    assert_eq!(
        update(
            "print: fallback",
            vec![prop("title", Some(PropType::String))],
            false,
            &[]
        ),
        "params:\n  title: string\nprint: fallback"
    );
}
