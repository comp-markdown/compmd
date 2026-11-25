use proptest::prelude::*;
use serde_json::{Value, json};

use super::{Unwritable, write};
use crate::features::code_actions::frontmatter::comments::Comments;
use crate::features::code_actions::frontmatter::testing::{mappings, read_back};

fn written(value: &Value) -> Result<String, Unwritable> {
    write(value.as_object().expect("a mapping"), &Comments::default())
}

#[test]
fn values_come_first_then_tables_and_lists_of_tables() {
    let value = json!({
        "md": {
            "version": 1,
            "import": [
                { "from": "@myorg/ui@1", "names": ["card", { "name": "chart", "as": "trend" }] },
                { "from": "./a.comp.md", "names": "*" },
            ],
        },
        "title": "Release notes",
        "tags": [],
        "seo": { "image": { "alt": "A harbour" } },
    });
    assert_eq!(
        written(&value),
        Ok("title = \"Release notes\"\ntags = []\n\n[md]\nversion = 1\n\n[[md.import]]\nfrom = \"@myorg/ui@1\"\nnames = [\"card\", { name = \"chart\", as = \"trend\" }]\n\n[[md.import]]\nfrom = \"./a.comp.md\"\nnames = \"*\"\n\n[seo.image]\nalt = \"A harbour\"\n".into())
    );
}

#[test]
fn keys_and_strings_are_quoted_and_escaped_as_toml_needs() {
    let value = json!({ "a b": "say \"hi\"\n\\", "x": {}, "c": "\u{1}" });
    assert_eq!(
        written(&value),
        Ok("\"a b\" = \"say \\\"hi\\\"\\n\\\\\"\nx = {}\nc = \"\\u0001\"\n".into())
    );
}

#[test]
fn an_empty_value_or_a_large_integer_is_unwritable_where_it_is() {
    assert_eq!(
        written(&json!({ "md": { "inputs": { "x": null } } })),
        Err(Unwritable("/md/inputs/x".into()))
    );
    assert_eq!(
        written(&json!({ "n": [u64::MAX] })),
        Err(Unwritable("/n/0".into()))
    );
}

#[test]
fn comments_go_above_keys_and_headers() {
    let value = json!({ "a": 1, "md": { "lang": "de" } });
    let comments = Comments {
        above: [
            ("/a".to_owned(), vec!["first".to_owned()]),
            ("/md".to_owned(), vec!["CompMD's".to_owned()]),
        ]
        .into(),
        end: Vec::new(),
    };
    assert_eq!(
        write(value.as_object().expect("a mapping"), &comments),
        Ok("# first\na = 1\n\n# CompMD's\n[md]\nlang = \"de\"\n".into())
    );
}

proptest! {
    #[test]
    fn what_is_written_reads_back_the_same(mapping in mappings(false)) {
        let text = write(&mapping, &Comments::default()).expect("TOML holds it");
        prop_assert_eq!(read_back("+++", &text), Value::Object(mapping), "{}", text);
    }
}
