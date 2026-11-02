//! Where hints are and aren't: leaves, inline components and continuation lines; code, math and prose, which have
//! none; a diagnostic alone; the edges of component lines.

use super::*;

#[test]
fn leaves_inline_components_and_continuation_lines() {
    let leaf = hint(&format!("{CARD}:: card[Sum{CARET}mary] {{title=x}}\n"));
    assert_eq!(leaf, Value::Null, "the fallback is text");
    let params = hint(&format!("{CARD}:: card[S] {{title=x ki{CARET}nd=note}}\n"));
    assert_eq!(params["subject"]["param"]["name"], "kind");
    let inline = hint(
        &format!("{CARD}Read :card[it]{{kind={CARET}}} and :bo{CARET}x[b].\n")
            .replacen(CARET, "", 1),
    );
    assert_eq!(inline["subject"]["name"], "box");
    let inline = hint(&format!(
        "{CARD}Read :card[it :box[b]{{bor{CARET}der=s}}] now.\n"
    ));
    assert_eq!(
        inline["subject"]["param"]["name"], "border",
        "inside another's fallback"
    );
    let quoted = hint(&format!("> - :: card[S] {{ti{CARET}tle=x}}\n"));
    assert_eq!(
        quoted["subject"]["kind"], "params",
        "no `card` here, so the universal params"
    );
    let continued = hint(&format!(
        "::: box {{gap=s}}\n::: + bor{CARET}der=subtle\nText.\n::: /box\n"
    ));
    assert_eq!(continued["subject"]["param"]["name"], "border");
    assert_eq!(continued["range"], range(2, 7, 13));
    let orphan = hint(&format!("Text.\n::: + bor{CARET}der=subtle\n"));
    assert_eq!(orphan["subject"], Value::Null, "F012 only");
    assert_eq!(orphan["diagnostics"][0]["code"], "F012");
}

#[test]
fn code_math_and_prose_have_no_hint() {
    assert_eq!(
        hint(&format!("```md\n::: box {{bor{CARET}der=x}}\n```\n")),
        Value::Null
    );
    assert_eq!(
        hint(&format!("Some `:box[b]{{bor{CARET}der=x}}` code.\n")),
        Value::Null
    );
    assert_eq!(hint(&format!("Plain pro{CARET}se.\n")), Value::Null);
    assert_eq!(hint(&format!("$$\n:: box[{CARET}]\n$$\n")), Value::Null);
    let definition = format!(
        "```compmd component=note\npar{CARET}ams:\n  tone: string?\n---\n::: box {{border=subtle}}\nN.\n::: /box\n```\n"
    );
    assert_eq!(hint(&definition), Value::Null, "a definition's header");
    let body = format!(
        "```compmd component=note\nparams:\n  tone: string?\n---\n::: box {{border=sub{CARET}tle}}\nN.\n::: /box\n```\n"
    );
    assert_eq!(
        hint(&body)["subject"]["param"]["name"],
        "border",
        "a definition's body"
    );
    let host = format!("```jsx component=hello\nconst x = <a>{{bor{CARET}der=1}}</a>;\n```\n");
    assert_eq!(hint(&host), Value::Null);
}

#[test]
fn a_diagnostic_alone_has_a_hint_without_a_subject() {
    let json = hint(&format!("Say {{{{ 1 {CARET}}}}}.\n"));
    assert_eq!(json["subject"], Value::Null);
    assert_eq!(json["diagnostics"][0]["code"], "W106");
    assert_eq!(json["range"], range(1, 10, 10));
}

#[test]
fn the_edges_of_component_lines() {
    let malformed = hint(&format!("::: ca{CARET}rd x\n"));
    assert_eq!(malformed["subject"]["name"], "card");
    assert_eq!(
        hint(&format!("::: card {CARET}x\n"))["subject"],
        Value::Null
    );
    let numbered = hint(&format!("1. ::: bo{CARET}x\n   B.\n   ::: /box\n"));
    assert_eq!(numbered["subject"]["name"], "box");
    let before = hint(&format!("::: box {CARET}{{gap=s}}\n::: /box\n"));
    assert_eq!(before["subject"], Value::Null, "before the `{{`");
    let continued = hint(&format!(
        "::: box {{gap=s}}\n::: + padding=s\n::: + bor{CARET}der=subtle\nB.\n::: /box\n"
    ));
    assert_eq!(continued["subject"]["param"]["name"], "border");
    let closed = hint(&format!("::: box\n::: /box\n::: + bor{CARET}der=subtle\n"));
    assert_eq!(closed["subject"], Value::Null, "after a closer");
    let class = hint(&format!("::: box {{.{CARET}"));
    assert_eq!(
        (
            &class["subject"]["param"]["name"],
            &class["subject"]["value"]
        ),
        (&json!("class"), &json!(""))
    );
    let colon = hint(&format!(
        "{CARD}::: card {{title=x}}\n:{CARET}: #footer\n::: /card\n"
    ));
    assert_eq!(colon["subject"], Value::Null, "on the colons");
    let nested = hint(&format!(
        "{CARD}::: card {{title=x}}\nB.\n:: #footer\n::: each {{in=([1]) as=n}}\nN.\n:: #emp{CARET}ty\nNone.\n::: /each\n::: /card\n"
    ));
    assert_eq!(nested["subject"]["component"], "each", "in another's slot");
    let inline = hint(&format!("a :1 and :b and :c[x and more{CARET}\n"));
    assert_eq!(inline, Value::Null);
    let unknown = hint_with(
        &format!(
            "---\nmd:\n  import:\n    - from: ./missing.comp.md\n      names: \"*\"\n---\n\n:: thi{CARET}ng[T]\n"
        ),
        &[],
    );
    assert_eq!(
        unknown["subject"].get("component"),
        None,
        "a name from an import that failed"
    );
}
