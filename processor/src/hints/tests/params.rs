//! Hints on params: a key and what it takes, a value and its tokens, the params not yet written, a declared param,
//! an input a param binds, strings, classes and ids, and how defaults are written.

use super::*;
use crate::tree::Literal;

#[test]
fn a_key_says_what_its_param_takes() {
    let json = hint(&format!("::: box {{padd{CARET}ing=m}}\n::: /box\n"));
    assert_eq!(json["range"], range(1, 10, 17));
    let subject = &json["subject"];
    assert_eq!(
        (&subject["kind"], &subject["component"]),
        (&json!("param"), &json!("box"))
    );
    assert_eq!(subject.get("value"), None);
    let padding = &subject["param"];
    assert_eq!(
        (&padding["origin"], &padding["breakpoints"]),
        (&json!("common"), &json!(true))
    );
    assert_eq!(
        padding["takes"]["description"],
        "one or two of `none`, `s`, `m`, `l` or a length in `px`, `rem`, `em` or `%`, such as `s l`"
    );
    let keys = texts(&json);
    assert!(keys.contains(&"padding") && keys.contains(&"gap") && keys.contains(&"lang"));
}

#[test]
fn a_value_offers_the_tokens_its_param_takes() {
    let json = hint(&format!("::: box {{border=sub{CARET}tle}}\n::: /box\n"));
    assert_eq!(json["range"], range(1, 17, 23));
    assert_eq!(json["subject"]["value"], "subtle");
    assert_eq!(texts(&json), ["none", "subtle", "strong"]);
    let empty = hint(&format!("::: box {{border={CARET}}}\n::: /box\n"));
    assert_eq!(
        empty["subject"]["value"], "",
        "after an `=` with nothing yet"
    );
    assert_eq!(empty["range"], range(1, 17, 17));
    assert_eq!(empty["diagnostics"][0]["code"], "F011");
    let unclosed = hint(&format!("::: box {{padding=m border={CARET}"));
    assert_eq!(
        (
            &unclosed["subject"]["param"]["name"],
            &unclosed["subject"]["value"]
        ),
        (&json!("border"), &json!(""))
    );
}

#[test]
fn between_params_the_list_offers_those_not_written() {
    let json = hint(&format!("::: box {{padding=m {CARET}}}\n::: /box\n"));
    assert_eq!(json["subject"]["kind"], "params");
    assert_eq!(json["range"], range(1, 20, 20));
    let params = names(&json);
    assert!(params.contains(&"display") && params.contains(&"border"));
    assert!(!params.contains(&"padding"));
    assert_eq!(params[0], "display", "the declared params first");
    let typed = hint(&format!("::: box {{padding=m bord{CARET}"));
    assert_eq!(typed["subject"]["kind"], "params", "a key being typed");
    assert_eq!(typed["range"], range(1, 20, 24));
    assert!(texts(&typed).contains(&"border"));
    assert_eq!(
        hint(&format!("::: box {{padding=m}}{CARET}\n::: /box\n")),
        Value::Null
    );
}

#[test]
fn a_declared_param_has_its_declaration() {
    let json = hint(&format!(
        "{CARD}::: card {{kind=w{CARET} title=x}}\n::: /card\n"
    ));
    let kind = &json["subject"]["param"];
    assert_eq!(
        (&kind["origin"], &kind["default"], &kind["required"]),
        (&json!("declared"), &json!("note"), &json!(false))
    );
    assert_eq!(texts(&json), ["note", "warning"]);
    let title = hint(&format!("{CARD}::: card {{ti{CARET}tle=x}}\n::: /card\n"));
    let title = &title["subject"]["param"];
    assert_eq!(
        (&title["doc"], &title["required"]),
        (&json!("The card's heading."), &json!(true))
    );
    let gap = hint(&format!("::: box {{md:ga{CARET}p=l}}\n::: /box\n"));
    assert_eq!(gap["range"], range(1, 13, 16));
    assert_eq!(gap["subject"]["param"]["origin"], "declared");
    assert_eq!(gap["subject"]["param"]["breakpoints"], true);
}

#[test]
fn a_bound_input_offers_the_inputs_it_binds() {
    let text = format!(
        "---\nmd:\n  inputs:\n    since: number = 1970\n    name: string = \"\"\n---\n\n```compmd component=slider\nparams:\n  of: input<number>\n---\nS.\n```\n\n:: slider[] {{of={CARET}}}\n"
    );
    let json = hint(&text);
    let takes = &json["subject"]["param"]["takes"];
    assert_eq!(
        takes["description"],
        "the name of an input of a number under `md.inputs`"
    );
    assert_eq!(texts(&json), ["since"]);
}

#[test]
fn strings_classes_and_ids_are_values() {
    let quoted = hint(&format!("::: box {{title=\"a b{CARET}"));
    assert_eq!(quoted["subject"]["kind"], "params", "`box` has no `title`");
    assert_eq!(quoted["range"], range(1, 10, 15));
    let open = hint(&format!("{CARD}::: card {{title=\"a b{CARET}"));
    assert_eq!(open["subject"]["value"], "\"a b", "to the end of the line");
    let class = hint(&format!("::: box {{.wi{CARET}de}}\n::: /box\n"));
    assert_eq!(
        (
            &class["subject"]["param"]["name"],
            &class["subject"]["value"]
        ),
        (&json!("class"), &json!("wide"))
    );
    let id = hint(&format!("::: box {{#in{CARET}tro .x}}\n::: /box\n"));
    assert_eq!(id["subject"]["param"]["name"], "id");
    let typed = hint(&format!("::: box {{.x #b{CARET}"));
    assert_eq!(
        (
            &typed["subject"]["param"]["name"],
            &typed["subject"]["value"]
        ),
        (&json!("id"), &json!("b"))
    );
}

#[test]
fn defaults_are_written_as_params_write_them() {
    let text = |value: &str| {
        StringPart::Text(Literal {
            value: value.into(),
            position: None,
        })
    };
    let interpolation = StringPart::Interpolation(crate::tree::Interpolation {
        path: "n".into(),
        ..Default::default()
    });
    for (default, expected) in [
        (ParamValue::Bare("m".into()), "m"),
        (ParamValue::Json("[1]".into()), "[1]"),
        (ParamValue::Class("wide".into()), "wide"),
        (ParamValue::Expr("1 + 1".into()), "(1 + 1)"),
        (ParamValue::Flag, "true"),
        (
            ParamValue::String(vec![text("Step "), interpolation]),
            "\"Step {{n}}\"",
        ),
    ] {
        assert_eq!(written(&DefaultValue::Param(default)), expected);
    }
    assert_eq!(written(&DefaultValue::Yaml(json!("a b"))), "a b");
    assert_eq!(written(&DefaultValue::Yaml(json!([1]))), "[1]");
}

#[test]
fn every_kinds_tokens_are_completions_it_accepts() {
    for kind in Kind::ALL {
        let hint = KindHint::of(kind);
        assert!(
            hint.tokens.iter().all(|token| kind.accepts_text(token)),
            "{kind:?}"
        );
        assert_eq!(hint.description, kind.describe());
    }
}
