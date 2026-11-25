use lsp_types::{HoverContents, Position, Range};

use super::*;
use crate::features::testing::{caret, processed};

/// The hover's text at the caret `|` of `marked`, and its range; `None` where there is none.
fn hovered(marked: &str) -> Option<(String, Range)> {
    let (text, at) = caret(marked);
    let hint = processed(&text).hint_at(at)?;
    let found = hover(&hint, &text, Encoding::Utf16)?;
    let HoverContents::Markup(markup) = found.contents else {
        panic!("Markdown");
    };
    Some((markup.value, found.range.expect("a range")))
}

/// The hover's text at the caret of `marked`.
fn text_at(marked: &str) -> String {
    hovered(marked).map(|(text, _)| text).unwrap_or_default()
}

#[test]
fn a_components_name_its_params_and_its_slots() {
    let (component, range) =
        hovered("::: bo|x {border=subtle}\nText.\n::: /box\n").expect("a hover");
    assert!(component.starts_with("**`box`**"), "{component}");
    assert!(
        component.contains("Params:\n- `"),
        "its signature: {component}"
    );
    assert_eq!(
        range,
        Range {
            start: Position {
                line: 0,
                character: 4
            },
            end: Position {
                line: 0,
                character: 7
            }
        }
    );
    assert!(text_at("::: box {bor|der=subtle}\n::: /box\n").starts_with("On `box`: `border`: "));
    assert!(
        text_at("::: box {border=subtle |}\n::: /box\n")
            .starts_with("The params `box` takes:\n- `")
    );
    let slot = text_at("::: box\nA.\n\n:: #foo|ter\nB.\n::: /box\n");
    assert!(
        slot.starts_with("**`footer`**, a slot of `box`, which declares"),
        "{slot}"
    );
}

#[test]
fn a_name_that_stands_for_nothing_with_its_diagnostic() {
    let unknown = text_at(":: nop|e[A]\n");
    assert!(
        unknown.starts_with("**`nope`** stands for no component here"),
        "{unknown}"
    );
    assert!(
        unknown.contains("---\n\n**F002**"),
        "the diagnostic: {unknown}"
    );
}

#[test]
fn paths_functions_frontmatter_keys_and_types() {
    let path = text_at("Hello {{ md.do|c.title }}.\n");
    assert!(path.starts_with("`md.doc`"), "{path}");
    let function = text_at("::: box {padding=(md.numb|er(1))}\n::: /box\n");
    assert!(
        function.starts_with("**`md.number`**\n\n```cel\n"),
        "{function}"
    );
    assert!(function.contains("(spec §8.2)"), "{function}");
    let nothing = text_at("::: box {padding=(md.nop|e(1))}\n::: /box\n");
    assert!(
        nothing.starts_with("**`md.nope`** is no function"),
        "{nothing}"
    );
    assert!(text_at("---\nmd:\n  la|ng: en\n---\n").starts_with("`md.lang`: "));
    assert!(text_at("---\nmd:\n  inputs:\n    n: num|ber\n---\n").starts_with("**`number`**: "));
    assert!(
        text_at("---\nmd:\n  inputs:\n    n: numbr|\n---\n").starts_with("**`numbr`** is no type")
    );
}

#[test]
fn prose_has_no_hover() {
    assert_eq!(hovered("Just te|xt.\n"), None);
}
