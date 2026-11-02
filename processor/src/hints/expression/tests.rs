use super::super::tests::{CARD, CARET, hint, hint_with, range, texts};
use super::md;
use serde_json::{Value, json};

const BOOKS: &str = "---\ntitle: Reading\nshelf: {name: Home}\nmd:\n  data:\n    - from: ./books.csv\n      as: books\n      schema: {title: string, pages: number, read: boolean}\n  inputs:\n    since: number = 1970\n---\n\n";

fn books(body: &str) -> Value {
    hint_with(
        &format!("{BOOKS}{body}"),
        &[("books.csv", "title,pages,read\nDune,412,true\n")],
    )
}

#[test]
fn each_step_of_a_path_names_what_it_is() {
    let data = books(&format!("{{{{md.data.boo{CARET}ks}}}}\n"));
    assert_eq!(data["range"], range(13, 11, 16));
    let subject = &data["subject"];
    assert_eq!(
        (&subject["kind"], &subject["path"]),
        (&json!("path"), &json!("md.data.books"))
    );
    let name = &subject["name"];
    assert_eq!(name["about"], "data from `./books.csv`");
    assert_eq!(name["type"], "list(map(string, dyn))");
    assert_eq!(
        name["fields"],
        json!(["pages: double", "read: bool", "title: string"])
    );
    assert_eq!(texts(&data), ["books"]);
    let md = books(&format!("{{{{m{CARET}d.doc.title}}}}\n"));
    assert_eq!(md["subject"]["path"], "md");
    assert!(
        md["subject"]["name"]["about"]
            .as_str()
            .is_some_and(|about| about.contains("`doc`"))
    );
    let title = books(&format!("{{{{ md.doc.ti{CARET}tle }}}}\n"));
    assert_eq!(
        (
            &title["subject"]["name"]["about"],
            &title["subject"]["name"]["type"]
        ),
        (&json!("the frontmatter's `title`"), &json!("string"))
    );
    assert_eq!(texts(&title), ["shelf", "title"]);
    let field = books(&format!("{{{{md.doc.shelf.na{CARET}me}}}}\n"));
    assert_eq!(
        field["subject"]["name"]["about"],
        "the frontmatter's `shelf.name`"
    );
    let input = books(&format!("{{{{md.input.sin{CARET}ce}}}}\n"));
    assert_eq!(
        input["subject"]["name"]["about"],
        "the input `since`, a number"
    );
    let colon = hint(&format!(
        "---\nreading: {{page:120}}\n---\n\n{{{{md.doc.reading.pa{CARET}ge}}}}\n"
    ));
    assert_eq!(
        colon["subject"]["name"]["about"],
        "`reading` has a key `page:120`; a colon needs a space after it to give `page` a value"
    );
    let unknown = books(&format!("{{{{md.doc.nothin{CARET}g}}}}\n"));
    assert_eq!(unknown["subject"]["name"].get("type"), None);
    assert_eq!(unknown["diagnostics"][0]["code"], "F014");
}

#[test]
fn after_md_the_names_and_the_functions_are_offered() {
    let json = books(&format!("{{{{md.{CARET}"));
    let names = texts(&json);
    for name in ["data", "doc", "input", "version", "sortBy", "today"] {
        assert!(names.contains(&name), "{name} in {names:?}");
    }
    assert_eq!(json["range"], range(13, 6, 6));
    for (path, about) in [
        ("md.doc", "the frontmatter's own keys"),
        ("md.data", "the data the frontmatter imports"),
        ("md.input", "the inputs the reader sets"),
        (
            "md.version",
            "the CompMD version the document is written in",
        ),
        ("md.lang", "the document's language, a BCP 47 tag"),
        ("md.tz", "the render's time zone"),
    ] {
        let json = books(&format!("{{{{{path}{CARET}}}}}\n"));
        assert_eq!(json["subject"]["name"]["about"], about, "{path}");
    }
    assert_eq!(
        books(&format!("{{{{md.doc.title}}}} {CARET}b\n")),
        Value::Null,
        "past the interpolation"
    );
    assert_eq!(
        books(&format!("\\{{{{md.do{CARET}c}}}}\n")),
        Value::Null,
        "escaped"
    );
}

#[test]
fn a_template_binds_its_name_and_md_loop() {
    let body = format!(
        "::: each {{in=(md.data.books) as=b}}\n{{{{b.ti{CARET}tle}}}}, {{{{md.loop.number}}}}\n::: /each\n"
    );
    let json = books(&body);
    assert_eq!(json["subject"]["name"]["type"], "string");
    assert_eq!(texts(&json), ["pages", "read", "title"]);
    let bound = books(&body.replacen("b.ti‸tle", "b‸.title", 1));
    assert_eq!(
        bound["subject"]["name"]["about"],
        "each element of `(md.data.books)`"
    );
    let number = books(
        &body
            .replacen('‸', "", 1)
            .replacen("md.loop.number", "md.loop.num‸ber", 1),
    );
    assert_eq!(
        (
            &number["subject"]["name"]["about"],
            &number["subject"]["name"]["type"]
        ),
        (
            &json!("the element's position, from 1, for display"),
            &json!("int")
        )
    );
    let one = books(&format!(
        "::: each {{in=[{{\"a\": 1}}] as=x}}\n{{{{x.{CARET}a}}}}\n::: /each\n"
    ));
    assert_eq!(one["subject"]["name"]["type"], "double", "a JSON list");
    let cel = books(&format!(
        "::: each {{in=([{{\"a\": 1}}]) as=x}}\n{{{{x.{CARET}a}}}}\n::: /each\n"
    ));
    assert_eq!(cel["subject"]["name"]["type"], "int", "a CEL list");
    let opener = books(&format!(
        "::: each {{in=(md.data.books) as=b}}\n{{{{b{CARET}}}}}\n:: #empty\n{{{{b}}}}\n::: /each\n"
    ));
    assert_eq!(
        opener["subject"]["name"]["about"],
        "each element of `(md.data.books)`"
    );
    let empty = books(&format!(
        "::: each {{in=(md.data.books) as=b}}\nB.\n:: #empty\n{{{{b{CARET}}}}}\n::: /each\n"
    ));
    assert_eq!(
        empty["subject"]["name"].get("about"),
        None,
        "`empty` binds nothing"
    );
}

#[test]
fn a_definitions_body_sees_its_params() {
    let json = hint(&CARD.replacen("{{title}}", "{{tit‸le}}", 1));
    let name = &json["subject"]["name"];
    assert_eq!(name["about"], "a param of `card`: The card's heading.");
    assert_eq!(name["type"], "string");
    let kind = hint(&CARD.replacen("{{title}}", "{{ki‸nd}}", 1));
    assert_eq!(kind["subject"]["name"]["about"], "a param of `card`");
}

#[test]
fn an_expression_names_its_paths_and_functions() {
    let exists = books(&format!(
        "::: if {{when=(md.data.books.exi{CARET}sts(b, !b.read))}}\nYes.\n::: /if\n"
    ));
    let subject = &exists["subject"];
    assert_eq!(
        (&subject["kind"], &subject["name"]),
        (&json!("function"), &json!("exists"))
    );
    assert_eq!(
        subject["hint"]["signatures"],
        json!(["list.exists(x, predicate) → bool"])
    );
    assert_eq!(exists["range"], range(13, 29, 35));
    assert!(texts(&exists).contains(&"filter") && !texts(&exists).contains(&"has"));
    let local = books(&format!(
        "::: if {{when=(md.data.books.exists(b, !b.re{CARET}ad))}}\nYes.\n::: /if\n"
    ));
    assert_eq!(local["subject"]["name"]["type"], "bool");
    let var = books(&format!(
        "::: if {{when=(md.data.books.exists(b, !b{CARET}.read))}}\nYes.\n::: /if\n"
    ));
    assert_eq!(
        var["subject"]["name"]["about"],
        "each element of `md.data.books`"
    );
    let md = books(&format!(
        ":value[x]{{of=(md.cur{CARET}rency(1, \"USD\"))}}\n"
    ));
    assert_eq!(md["subject"]["name"], "md.currency");
    assert_eq!(
        md["subject"]["hint"]["signatures"],
        json!(["md.currency(number, string) → string"])
    );
    let size = books(&format!(
        ":value[x]{{of=(string(si{CARET}ze(md.data.books)))}}\n"
    ));
    assert_eq!(
        size["subject"]["hint"]["doc"],
        "the length of a string, a list or a map"
    );
    assert!(texts(&size).contains(&"has") && !texts(&size).contains(&"trim"));
    let unknown = books(&format!(":value[x]{{of=(nothin{CARET}g(1))}}\n"));
    assert_eq!(unknown["subject"].get("hint"), None);
    let text = books(&format!(":value[x]{{of=(\"a b{CARET}\")}}\n"));
    assert_eq!(
        text["subject"]["kind"], "param",
        "inside a string, the param's hint"
    );
    let number = books(&format!(":value[x]{{of=(string(1{CARET}2))}}\n"));
    assert_eq!(number["subject"]["kind"], "param");
    let unclosed = books(&format!("::: if {{when=(md.input.si{CARET}nce"));
    assert_eq!(unclosed["subject"]["name"]["type"], "double");
    let quoted = books(&format!(
        "::: box {{title=\"In {{{{md.doc.tit{CARET}le}}}}\"}}\n::: /box\n"
    ));
    assert_eq!(
        quoted["subject"]["path"], "md.doc.title",
        "an interpolation in a string"
    );
}

#[test]
fn the_spec_lists_every_function() {
    let spec =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../compmd-spec-v1.md"))
            .expect("the spec");
    let section = spec
        .split("### 8.2 ")
        .nth(1)
        .and_then(|rest| rest.split("### 8.3 ").next())
        .expect("section 8.2");
    let named = |name: &str| {
        section.contains(&format!("`{name}`")) || section.contains(&format!("`{name}("))
    };
    let missing: Vec<String> = crate::cel::function_names(true)
        .into_iter()
        .chain(crate::cel::function_names(false))
        .map(ToString::to_string)
        .chain(super::md_names().iter().map(|name| format!("md.{name}")))
        .filter(|name| !named(name))
        .collect();
    assert!(missing.is_empty(), "spec §8.2 doesn't name {missing:?}");
}

#[test]
fn paths_through_fields_maps_and_lists() {
    let field = books(&format!("{{{{md.data.books.ti{CARET}tle}}}}\n"));
    assert_eq!(
        field["subject"]["name"]["about"],
        "the field `title` of `md.data.books`'s records"
    );
    assert_eq!(texts(&field), Vec::<&str>::new(), "a list has no fields");
    let tags = hint(&format!(
        "---\ntags: [a, b]\n---\n\n{{{{md.doc.ta{CARET}gs}}}}\n"
    ));
    assert_eq!(tags["subject"]["name"]["type"], "list(string)");
    assert_eq!(tags["subject"]["name"].get("fields"), None);
    let text = books(&format!("{{{{md.doc.title.x{CARET}}}}}\n"));
    assert_eq!(
        text["subject"]["name"].get("type"),
        None,
        "a string has no fields"
    );
    let raw = hint_with(
        &format!(
            "---\nmd:\n  data:\n    - from: ./raw.json\n      as: raw\n---\n\n{{{{md.data.raw.a.b{CARET}}}}}\n"
        ),
        &[("raw.json", "{}")],
    );
    assert_eq!(
        raw["subject"]["name"]["type"], "dyn",
        "data without a schema"
    );
    let meta = hint(&format!(
        "```compmd component=chip\nparams:\n  meta: object\n---\n{{{{meta.x{CARET}}}}}\n```\n"
    ));
    assert_eq!(meta["subject"]["name"]["type"], "dyn", "a map's values");
    let slot = hint(&CARD.replacen("{{title}}", "{{md.slot.foo‸ter}}", 1));
    assert_eq!(
        slot["subject"]["name"]["about"],
        "the content of the slot `footer`"
    );
    let default = hint(&CARD.replacen("{{title}}", "{{md.sl‸ot}}", 1));
    assert_eq!(
        default["subject"]["name"]["about"],
        "the default slot's content"
    );
    let each = |path: &str| {
        books(&format!(
            "::: each {{in=(md.data.books) as=b}}\n{{{{md.loop.{path}{CARET}}}}}\n::: /each\n"
        ))["subject"]["name"]["about"]
            .clone()
    };
    assert_eq!(each("index"), "the element's position, from 0");
    assert_eq!(each("count"), "how many elements there are");
    assert_eq!(each("first"), "whether it is the first element");
    assert_eq!(each("last"), "whether it is the last element");
    assert_eq!(each("other"), Value::Null);
    let lp = books(&format!(
        "::: each {{in=(md.data.books) as=b}}\n{{{{md.lo{CARET}op}}}}\n::: /each\n"
    ));
    assert!(
        lp["subject"]["name"]["about"]
            .as_str()
            .is_some_and(|about| about.contains("`index`"))
    );
}

#[test]
fn templates_without_a_list_bind_dyn() {
    let unbound = books(&format!(
        "::: each {{in=(md.data.books)}}\n{{{{b{CARET}}}}}\n::: /each\n"
    ));
    assert_eq!(unbound["subject"]["name"].get("about"), None, "no `as`");
    let bare = books(&format!(
        "::: each {{in=rows as=r}}\n{{{{r{CARET}}}}}\n::: /each\n"
    ));
    assert_eq!(bare["subject"]["name"]["about"], "each element of `rows`");
    let missing = books(&format!("::: each {{as=r}}\n{{{{r{CARET}}}}}\n::: /each\n"));
    assert_eq!(missing["subject"]["name"]["about"], "each element of `in`");
    let number = books(&format!(
        "::: each {{in=(5) as=r}}\n{{{{r{CARET}}}}}\n::: /each\n"
    ));
    assert_eq!(number["subject"]["name"]["type"], "dyn");
    let after = books(&format!(
        "::: each {{in=(md.data.books) as=b}}\nB.\n::: /each\n\n{{{{b{CARET}}}}}\n"
    ));
    assert_eq!(
        after["subject"]["name"].get("about"),
        None,
        "past the closer"
    );
}

#[test]
fn macros_bind_their_names_inside_them_only() {
    let closed = books(&format!(
        ":value[x]{{of=(string(md.data.books.exists(b, b.read) && b{CARET}))}}\n"
    ));
    assert_eq!(closed["subject"]["name"].get("about"), None);
    let unnamed = books(&format!(":value[x]{{of=(md.data.books.exists(x{CARET}"));
    assert_eq!(unnamed["subject"]["name"].get("about"), None, "no `,` yet");
    let nested = books(&format!(
        ":value[x]{{of=(string(md.data.books.exists(b, md.data.books.exists(c, c.re{CARET}ad))))}}\n"
    ));
    assert_eq!(nested["subject"]["name"]["type"], "bool");
    let opening = books(&format!(":value[x]{{of={CARET}(md.today())}}\n"));
    assert_eq!(opening["subject"]["kind"], "param", "on the `(`");
}

#[test]
fn every_function_is_described() {
    let names = crate::cel::function_names(true)
        .into_iter()
        .chain(crate::cel::function_names(false));
    for name in names {
        let described = crate::cel::described(name).expect("described");
        assert!(
            !described.doc.is_empty() && !described.signatures.is_empty(),
            "{name}"
        );
    }
    for name in super::md_names() {
        let described = md::described(name).expect("described");
        assert!(described.signatures[0].starts_with(&format!("md.{name}(")));
    }
    assert_eq!(md::described("nothing"), None);
    assert_eq!(
        crate::cel::described("indexOf").map(|described| described.signatures),
        Some(vec![
            "string.indexOf(string) → int".to_string(),
            "string.indexOf(string, int) → int".to_string()
        ])
    );
}
