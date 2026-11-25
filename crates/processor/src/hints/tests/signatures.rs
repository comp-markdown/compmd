//! Signature help: the call an expression's caret is in, or a component's param list.

use super::CARET;
use crate::context::RenderContext;
use crate::expand::Options;
use crate::hints::SignatureHelp;
use crate::{Ambient, ParseOptions, Processor};
use std::rc::Rc;

/// The help at the caret of `text`.
fn help(text: &str) -> Option<SignatureHelp> {
    let offset = text.chars().position(|c| c == CARET).expect("a caret");
    let context = RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::default(),
    };
    Processor::new("a.comp.md", text.replacen(CARET, "", 1))
        .parse(&ParseOptions::default())
        .resolve((), (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&Options::default())
        .signature_at(offset)
}

/// The function and argument of the call at the caret of `text`.
fn call(text: &str) -> Option<(String, usize)> {
    match help(text)? {
        SignatureHelp::Call { function, argument } => Some((function.name, argument)),
        SignatureHelp::Params { .. } => None,
    }
}

#[test]
fn a_call_with_the_argument_the_caret_is_in() {
    assert_eq!(
        call("::: box {padding=(md.number(‸))}\n::: /box\n"),
        Some(("md.number".into(), 0))
    );
    assert_eq!(
        call("::: box {padding=(md.number(1, \"a,(b\", ‸))}\n::: /box\n"),
        Some(("md.number".into(), 2))
    );
    assert_eq!(
        call("::: box {padding=(md.doc.tags.map(t, (t + ‸)))}\n::: /box\n"),
        Some(("map".into(), 1)),
        "past a group"
    );
    assert_eq!(
        call(":x[a]{n=(size(\"a\\\"b\", ‸))}\n"),
        Some(("size".into(), 1)),
        "an escaped quote"
    );
    assert_eq!(
        call("::: box {padding=(nope(‸))}\n::: /box\n"),
        None,
        "no such function"
    );
    assert_eq!(
        call("::: box {padding=(‸1)}\n::: /box\n"),
        None,
        "in the expression, not a call"
    );
    assert_eq!(call("See size(‸a) in prose.\n"), None, "prose");
}

#[test]
fn a_param_list_with_every_param_and_the_one_the_caret_is_on() {
    let Some(SignatureHelp::Params {
        component,
        params,
        active,
    }) = help("::: box {border=subtle bor‸der=x}\n::: /box\n")
    else {
        panic!("a param list");
    };
    assert_eq!(component, "box");
    assert_eq!(
        active.map(|active| params[active].name.as_str()),
        Some("border")
    );
    let Some(SignatureHelp::Params { active, .. }) = help("::: box {border=subtle ‸}\n::: /box\n")
    else {
        panic!("a param list");
    };
    assert_eq!(active, None, "between params");
    assert_eq!(help("::: bo‸x\n::: /box\n"), None, "a name");
    assert_eq!(help("```js\nf(‸)\n```\n"), None, "code");
}
