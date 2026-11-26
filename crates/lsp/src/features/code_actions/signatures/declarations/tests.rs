use compmd::analysis::{Prop, PropType, Props};
use serde_json::Value;

use super::{Declared, kebab};

fn prop(name: &str, kind: Option<PropType>) -> Prop {
    Prop {
        name: name.into(),
        kind,
        ..Prop::default()
    }
}

fn lines_of(props: Vec<Prop>) -> Vec<String> {
    let declared = Declared::of(&Props { props, rest: false });
    declared
        .params
        .into_iter()
        .flat_map(|param| param.lines)
        .collect()
}

#[test]
fn each_prop_is_a_param_in_kebab_case_with_its_type() {
    let tone = PropType::Enum {
        members: vec!["info".into(), "warn".into()],
    };
    assert_eq!(
        lines_of(vec![
            prop("title", Some(PropType::String)),
            prop("maxCount", Some(PropType::Number)),
            prop("open", Some(PropType::Boolean)),
            prop("tone", Some(tone)),
        ]),
        [
            "  title: string",
            "  max-count: number",
            "  open: boolean",
            "  tone: info | warn"
        ]
    );
}

#[test]
fn optional_and_defaults_take_the_short_form_or_the_long_one() {
    let with = |name: &str, kind: PropType, default: Option<Value>| Prop {
        optional: true,
        default,
        ..prop(name, Some(kind))
    };
    let tone = PropType::Enum {
        members: vec!["info".into(), "warn".into()],
    };
    assert_eq!(
        lines_of(vec![
            with("icon", PropType::String, None),
            with("size", PropType::Number, Some(3.into())),
            with("label", PropType::String, Some("Read more".into())),
            with("tone", tone, Some("warn".into())),
            with("note", PropType::String, Some("a: b".into())),
            with("open", PropType::Boolean, Some(true.into())),
            with("id", PropType::String, Some("intro".into())),
        ]),
        [
            "  icon: string?",
            "  size: number = 3",
            "  label: string = \"Read more\"",
            "  tone: info | warn = warn",
            "  note:",
            "    type: string",
            "    default: \"a: b\"",
            "  open: boolean = true",
            "  id: string = intro",
        ]
    );
}

#[test]
fn what_the_code_doesnt_state_is_left_as_a_comment() {
    let one = PropType::Enum {
        members: vec!["only".into()],
    };
    let spaced = PropType::Enum {
        members: vec!["to do".into(), "done".into()],
    };
    let wrong = Prop {
        optional: true,
        default: Some("x".into()),
        ..prop("count", Some(PropType::Number))
    };
    assert_eq!(
        lines_of(vec![
            prop("data", None),
            prop("kind", Some(one)),
            prop("status", Some(spaced)),
            wrong
        ]),
        [
            "  # data: the component's code doesn't say its type",
            "  # kind: the component's code doesn't say its type",
            "  # status: the component's code doesn't say its type",
            "  # count: its default isn't a value its type takes",
        ]
    );
}

#[test]
fn children_and_slot_props_are_slots_and_the_bindings_own_prop_nothing() {
    let declared = Declared::of(&Props {
        props: vec![
            prop("children", Some(PropType::Slot)),
            prop("sidePanel", Some(PropType::Slot)),
            prop("compmd", None),
            prop("title", Some(PropType::String)),
        ],
        rest: false,
    });
    assert_eq!(declared.slots, ["default", "side-panel"]);
    assert_eq!(declared.params.len(), 1);
}

#[test]
fn camel_case_becomes_kebab_case() {
    assert_eq!(kebab("alignItems"), "align-items");
    assert_eq!(kebab("URL"), "u-r-l");
    assert_eq!(kebab("title"), "title");
}
