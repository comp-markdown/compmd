use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::*;
use crate::features::testing::processed;

/// `compmd-grammars`, the submodule with the shared token names and test set.
fn grammars() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../grammars/compmd-grammars")
}

fn json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).expect("the file")).expect("JSON")
}

/// The tokens of `text`, processed as the server processes a document without a project.
fn tokens_of(text: &str) -> Vec<SyntaxToken> {
    processed(text).tokens()
}

/// Each encoded token with where it is: its line, its first character, its length, its type and its modifiers.
fn decoded(tokens: &SemanticTokens) -> Vec<(u32, u32, u32, SemanticTokenType, u32)> {
    let (mut line, mut character) = (0, 0);
    tokens
        .data
        .iter()
        .map(|token| {
            if token.delta_line > 0 {
                character = 0;
            }
            line += token.delta_line;
            character += token.delta_start;
            (
                line,
                character,
                token.length,
                TYPES[token.token_type as usize].clone(),
                token.token_modifiers_bitset,
            )
        })
        .collect()
}

#[test]
fn each_kind_has_the_type_the_shared_names_give_it() {
    let names = json(&grammars().join("syntax/tokens.json"));
    let names = names["tokens"].as_object().expect("the tokens");
    for kind in TokenKind::ALL {
        assert_eq!(
            names[kind.name()]["semantic"].as_str(),
            Some(type_of(kind).as_str()),
            "{}",
            kind.name()
        );
    }
    // An escape is in its string's token, which has its type.
    assert_eq!(
        names["string-escape"]["semantic"],
        names["param-string"]["semantic"]
    );
    for (name, token) in names.iter().filter(|(name, _)| *name != "string-escape") {
        let typed = token["semantic"].is_string();
        assert_eq!(
            typed,
            TokenKind::ALL.iter().any(|kind| kind.name() == name),
            "`{name}` is a kind when it has a type"
        );
    }
}

/// Where the UTF-16 index `index` of `units` is: its line and its character.
fn position_of(units: &[u16], index: usize) -> (u32, u32) {
    let before = &units[..index];
    let line = before
        .iter()
        .filter(|unit| **unit == u16::from(b'\n'))
        .count();
    let character = before
        .iter()
        .rev()
        .take_while(|unit| **unit != u16::from(b'\n'))
        .count();
    (
        u32::try_from(line).expect("a line"),
        u32::try_from(character).expect("a character"),
    )
}

/// Where each span of a case starts, as `compmd-grammars`' `tests/cases.js` finds it: the next occurrence of its text
/// after the previous span's start, in UTF-16 units.
fn starts_of(units: &[u16], spans: &[Value]) -> Vec<usize> {
    let mut from = 0;
    spans
        .iter()
        .map(|span| {
            let needle: Vec<u16> = span[0].as_str().expect("its text").encode_utf16().collect();
            let start = (from..=units.len() - needle.len())
                .find(|&at| units[at..at + needle.len()] == needle[..])
                .expect("the span");
            from = start + 1;
            start
        })
        .collect()
}

#[test]
fn the_shared_test_set_holds_for_the_servers_tokens() {
    let names = json(&grammars().join("syntax/tokens.json"));
    let folder = grammars().join("tests/cases");
    let mut files: Vec<PathBuf> = fs::read_dir(&folder)
        .expect("the cases")
        .map(|entry| entry.expect("a case").path())
        .collect();
    files.sort();
    let mut checked = 0;
    for file in files {
        for case in json(&file).as_array().expect("cases") {
            let (name, document) = (
                &case["name"],
                case["document"].as_str().expect("a document"),
            );
            let encoded = decoded(&encode(
                &tokens_of(document),
                document,
                Encoding::Utf16,
                None,
            ));
            let units: Vec<u16> = document.encode_utf16().collect();
            let spans = case["tokens"].as_array().expect("spans");
            for (span, start) in spans.iter().zip(starts_of(&units, spans)) {
                let (text, token) = (
                    span[0].as_str().expect("its text"),
                    span[1].as_str().expect("its token"),
                );
                let Some(expected) = names["tokens"][token]["semantic"].as_str() else {
                    continue;
                };
                let (line, character) = position_of(&units, start);
                let covering = encoded.iter().find(|(at, first, length, ..)| {
                    *at == line && *first <= character && character < first + length
                });
                assert_eq!(
                    covering.map(|token| token.3.as_str()),
                    Some(expected),
                    "{name}: {text} ({token})"
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 50, "the set was read: {checked} spans");
}

#[test]
fn tokens_are_relative_in_the_encodings_units_and_a_range_takes_those_starting_in_it() {
    let text = "Été :x[a] {{ md.doc.name }}\n::: box\n";
    let tokens = tokens_of(text);
    let all = decoded(&encode(&tokens, text, Encoding::Utf16, None));
    assert_eq!(all[0], (0, 4, 1, SemanticTokenType::OPERATOR, 0));
    assert_eq!(
        all[1],
        (0, 5, 1, SemanticTokenType::TYPE, 1),
        "`x` stands for no component"
    );
    let utf8 = decoded(&encode(&tokens, text, Encoding::Utf8, None));
    assert_eq!(utf8[0].1, 6, "`É` and `é` are two bytes each");
    let second = Range {
        start: Position {
            line: 1,
            character: 0,
        },
        end: Position {
            line: 2,
            character: 0,
        },
    };
    let ranged = decoded(&encode(&tokens, text, Encoding::Utf16, Some(second)));
    assert_eq!(
        ranged,
        [
            (1, 0, 3, SemanticTokenType::OPERATOR, 0),
            (1, 4, 3, SemanticTokenType::TYPE, 0)
        ]
    );
    assert_eq!(
        legend().token_modifiers,
        [SemanticTokenModifier::new("unknown")]
    );
}
