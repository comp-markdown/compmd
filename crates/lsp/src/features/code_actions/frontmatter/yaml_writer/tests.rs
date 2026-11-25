use proptest::prelude::*;
use serde_json::{Value, json};

use super::write;
use crate::features::code_actions::frontmatter::comments::Comments;
use crate::features::code_actions::frontmatter::testing::{mappings, read_back};

fn written(value: &Value) -> String {
    write(value.as_object().expect("a mapping"), &Comments::default())
}

#[test]
fn mappings_are_blocks_and_lists_of_plain_values_flow() {
    let value = json!({
        "title": "Release notes",
        "md": {
            "version": 1,
            "import": [
                { "from": "@myorg/ui@1", "names": ["card", "callout"] },
                { "from": "./chart.jsx", "names": [{ "name": "chart", "as": "trend" }] },
            ],
            "inputs": {},
        },
        "tags": [],
        "draft": null,
    });
    assert_eq!(
        written(&value),
        "title: Release notes\nmd:\n  version: 1\n  import:\n    - from: \"@myorg/ui@1\"\n      names: [card, callout]\n    - from: ./chart.jsx\n      names:\n        - name: chart\n          as: trend\n  inputs: {}\ntags: []\ndraft: null\n"
    );
}

#[test]
fn a_string_is_quoted_when_plain_it_would_read_otherwise() {
    let value =
        json!({ "a": "no", "b": "1.10", "c": "true", "d": "a: b", "e": " x", "f": "\u{7f}" });
    assert_eq!(
        written(&value),
        "a: no\nb: \"1.10\"\nc: \"true\"\nd: \"a: b\"\ne: \" x\"\nf: \"\\u007F\"\n"
    );
}

#[test]
fn lists_in_lists_and_comments_above_their_keys() {
    let value = json!({ "grid": [[1, [2]], { "a": [] }], "b": 1 });
    let comments = Comments {
        above: [("/b".to_owned(), vec!["the last".to_owned(), String::new()])].into(),
        end: vec!["done".into()],
    };
    assert_eq!(
        write(value.as_object().expect("a mapping"), &comments),
        "grid:\n  -\n    - 1\n    - [2]\n  - a: []\n# the last\n#\nb: 1\n# done\n"
    );
}

proptest! {
    #[test]
    fn what_is_written_reads_back_the_same(mapping in mappings(true)) {
        // An empty frontmatter reads as no keys, which the processor takes for an empty mapping.
        prop_assume!(!mapping.is_empty());
        let text = write(&mapping, &Comments::default());
        prop_assert_eq!(read_back("---", &text), Value::Object(mapping), "{}", text);
    }
}
