use super::{Given, lines, read};
use syn::parse_quote;

fn doc(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

#[test]
fn doc_attributes_give_their_lines() {
    let attrs: Vec<syn::Attribute> = vec![
        parse_quote!(#[doc = " The length."]),
        parse_quote!(#[doc = ""]),
        parse_quote!(#[doc = " A\n second"]),
        parse_quote!(#[inline]),
        parse_quote!(#[doc(hidden)]),
        parse_quote!(#[doc = concat!("a", "b")]),
    ];
    assert_eq!(lines(&attrs), ["The length.", "", "A", "second"]);
}

#[test]
fn the_first_paragraph_is_the_description_and_the_cel_block_the_examples() {
    let read = read(&doc(
        "The length of a string,\nin characters.\n\nMore.\n\n```cel\nsize(\"héllo\") // 5\n\nlet price = 12\nlet md.data = SALES\nmd.currency(price * 10, \"USD\") // \"$120.00\"\n```",
    ))
    .expect("a doc");
    assert_eq!(read.description, "The length of a string, in characters.");
    assert_eq!(read.examples.len(), 2);
    assert_eq!(read.examples[0].expression, "size(\"héllo\")");
    assert_eq!(read.examples[0].result, "5");
    assert_eq!(read.examples[0].given.len(), 0);
    assert_eq!(
        read.examples[1].given,
        [
            ("price".to_string(), Given::Json("12".to_string())),
            ("md.data".to_string(), Given::Constant("SALES".to_string())),
        ]
    );
    let tokens = read.examples[1].tokens().to_string();
    assert!(tokens.contains(r#"given : & [("price" , "12") , ("md.data" , SALES)]"#));
    assert!(tokens.contains(r#"result : "\"$120.00\"""#));
}

#[test]
fn what_a_doc_lacks_is_said() {
    let error = |text: &str| read(&doc(text)).err();
    assert_eq!(
        error("").as_deref(),
        Some("has no description: its doc's first paragraph")
    );
    assert_eq!(
        error("```cel\nf() // 1\n```").as_deref(),
        Some("has no description: its doc's first paragraph")
    );
    assert_eq!(
        error("The length.").as_deref(),
        Some("has no example: a ```cel block in its doc")
    );
    assert_eq!(
        error("The length.\n```cel\nf() // 1").as_deref(),
        Some("has a ```cel block that doesn't end")
    );
    assert_eq!(
        error("The length.\n```cel\n```").as_deref(),
        Some("has no example in its ```cel block")
    );
    assert_eq!(
        error("The length.\n```cel\nf()\n```").as_deref(),
        Some("has `f()`, which isn't `expression // result`")
    );
    assert_eq!(
        error("The length.\n```cel\nlet x\nf() // 1\n```").as_deref(),
        Some("has `let x`, which isn't `let name = value`")
    );
    assert_eq!(
        error("The length.\n```cel\nf() // 1\nlet x = 1\n```").as_deref(),
        Some("has a `let` with no example after it")
    );
}

#[test]
fn a_constant_is_in_upper_case() {
    assert_eq!(Given::of("SALES_2"), Given::Constant("SALES_2".to_string()));
    assert_eq!(
        Given::of("{\"a\": 1}"),
        Given::Json("{\"a\": 1}".to_string())
    );
    assert_eq!(Given::of("Sales"), Given::Json("Sales".to_string()));
    assert_eq!(Given::of("null"), Given::Json("null".to_string()));
}
