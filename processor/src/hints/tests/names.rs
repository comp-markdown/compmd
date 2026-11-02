//! Hints on names: a component's, a code renderer's language, a built-in's, an unknown one's, a slot's.

use super::*;

#[test]
fn a_name_is_the_component_it_stands_for() {
    let json = hint(&format!(
        "{CARD}::: ca{CARET}rd {{title=\"Setup\"}}\nText.\n::: /card\n"
    ));
    assert_eq!(json["range"], range(12, 5, 9));
    let subject = &json["subject"];
    assert_eq!(
        (&subject["kind"], &subject["name"]),
        (&json!("component"), &json!("card"))
    );
    let card = &subject["component"];
    assert_eq!(
        (&card["kind"], &card["provided"]),
        (&json!("compmd"), &json!(false))
    );
    assert_eq!(card.get("file"), None, "defined in the document");
    assert_eq!(card["defined"]["start"], json!({ "line": 1, "column": 1 }));
    assert_eq!(card["signature"]["params"][1]["doc"], "The card's heading.");
    let completions = texts(&json);
    assert!(completions.contains(&"card") && completions.contains(&"box"));
    let closer = hint(&format!("{CARD}::: card {{title=x}}\n::: /c{CARET}ard\n"));
    assert_eq!(closer["range"], range(13, 6, 10));
    assert_eq!(closer["subject"]["name"], "card");
}

#[test]
fn a_rendered_code_blocks_language_and_params_are_its_renderers() {
    let shout = "```compmd component=shout\nrenders: [shout, yell]\nparams:\n  loud: boolean = false\n---\n{{code}}\n```\n\n";
    let language = hint(&format!("{shout}  ```ye{CARET}ll {{loud}}\nhi\n```\n"));
    assert_eq!(language["range"], range(9, 6, 10));
    assert_eq!(language["subject"]["name"], "shout");
    assert_eq!(language["subject"]["component"]["kind"], "compmd");
    assert_eq!(texts(&language), ["shout", "yell"]);
    let key = hint(&format!("{shout}```shout {{lo{CARET}ud}}\nhi\n```\n"));
    assert_eq!(key["subject"]["param"]["name"], "loud");
    assert_eq!(key["range"], range(9, 11, 15));
    let keys = texts(&key);
    assert!(keys.contains(&"alt") && keys.contains(&"width"));
    assert!(!keys.contains(&"code"), "the block's text is `code`");
    let alt = hint(&format!("{shout}```shout {{al{CARET}t=\"Hi\"}}\nhi\n```\n"));
    assert_eq!(alt["subject"]["param"]["origin"], "attribute");
    let before = hint(&format!("{shout}```shout {CARET}{{loud}}\nhi\n```\n"));
    assert_eq!(before.get("subject"), None, "before the list");
    assert_eq!(before["diagnostics"][0]["code"], "W101", "no `alt`");
    let body = hint(&format!("{shout}```shout\nh{CARET}i\n```\n"));
    assert_eq!(body, Value::Null, "the code itself");
    let plain = hint(&format!("```sh {{lo{CARET}ud}}\nhi\n```\n"));
    assert_eq!(plain, Value::Null, "no renderer");
    let no_list = hint(&format!("{shout}```shout {{alt=x}} {CARET}\nhi\n```\n"));
    assert_eq!(no_list, Value::Null, "past the list");
    let none = hint(&format!("{shout}```shout {CARET}\nhi\n```\n"));
    assert_eq!(none.get("subject"), None, "no list");
}

#[test]
fn a_built_in_is_cores_and_an_unknown_name_has_its_diagnostic() {
    let json = hint(&format!("::: bo{CARET}x\n::: /box\n"));
    let core = &json["subject"]["component"];
    assert_eq!(
        (&core["kind"], &core["file"], &core["provided"]),
        (
            &json!("processor"),
            &json!("@compmd/core@1.0.0/index.comp.md"),
            &json!(true)
        )
    );
    let unknown = hint(&format!("::: bxo{CARET}\n::: /bxo\n"));
    assert_eq!(unknown["subject"].get("component"), None);
    assert_eq!(unknown["diagnostics"][0]["code"], "F002");
    assert_eq!(unknown["diagnostics"][0]["fix"]["text"], "box");
    let typed = hint(&format!("::: car{CARET}"));
    assert_eq!(typed["subject"]["name"], "car", "a name being typed");
}

#[test]
fn a_host_language_component_without_a_signature_takes_the_universal_params() {
    let text = format!(
        "---\nmd:\n  import:\n    - from: ./Chart.jsx\n      names: [{{name: Chart, as: chart}}]\n---\n\n:: chart[C] {{{CARET}}}\n"
    );
    let json = hint_with(&text, &[("Chart.jsx", "export default () => null;")]);
    let params = names(&json);
    assert_eq!(params[0], "width");
    let name = hint_with(
        &text.replace("{‸}", "{}").replace(":: chart", ":: ch‸art"),
        &[("Chart.jsx", "export default () => null;")],
    );
    assert_eq!(name["subject"]["component"]["kind"], "jsx");
    assert_eq!(name["subject"]["component"]["file"], "Chart.jsx");
}

#[test]
fn a_slot_line_names_the_containers_slots() {
    let json = hint(&format!(
        "{CARD}::: card {{title=x}}\nBody.\n:: #foo{CARET}ter\nEnd.\n::: /card\n"
    ));
    assert_eq!(json["range"], range(14, 5, 11));
    let subject = &json["subject"];
    assert_eq!(
        (&subject["component"], &subject["slot"]),
        (&json!("card"), &json!("footer"))
    );
    assert_eq!(subject["slots"], json!(["default", "footer"]));
    assert_eq!(texts(&json), ["footer"]);
    let each = hint(&format!(
        "::: box\n::: each {{in=([1]) as=n}}\n{{{{n}}}}\n:: #emp{CARET}ty\nNone.\n::: /each\n::: /box\n"
    ));
    assert_eq!(
        each["subject"]["component"], "each",
        "the innermost container"
    );
    assert_eq!(texts(&each), ["before", "empty"]);
    let outside = hint(&format!(":: #foo{CARET}ter\n"));
    assert_eq!(outside["diagnostics"][0]["code"], "F015");
}
