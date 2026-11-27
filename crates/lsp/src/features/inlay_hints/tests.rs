use compmd::tokens::{SyntaxToken, TokenKind};
use lsp_types::Position;
use serde_json::{Value, json};

use super::*;
use crate::features::testing::{asking, processed};

/// Its title, a component whose body gives an expression's value, used once, a list's items, and a list written
/// alone, which F024 says can't be.
const NOTES: &str = "---\ntitle: Harbour notes\nteam: [Ana, Kenji, Priya]\n---\n\n# {{md.doc.title}}\n\n```compmd component=badge\nparams:\n  status: string\n---\n:value[s]{of=(status == \"done\" ? \"success\" : \"neutral\")}\n```\n\n:: badge[B] {status=done}\n\n::: each {in=(md.doc.team) as=p join=inline sep=\", \"}\n{{p}}\n::: /each\n\n{{md.doc.team}}\n";

fn lines(first: u32, last: u32) -> Range {
    Range {
        start: Position {
            line: first,
            character: 0,
        },
        end: Position {
            line: last,
            character: u32::MAX,
        },
    }
}

/// The hints of `text` in `range`, as JSON.
fn hints(text: &str, range: Range) -> Value {
    let processor = processed(text);
    let hints = asking(text, &processor, &[], |asked| inlay_hints(range, asked));
    serde_json::to_value(hints).expect("hints serialize")
}

#[test]
fn each_value_is_after_what_gives_it_padded_and_without_a_kind() {
    let team = r#"["Ana","Kenji","Priya"]"#;
    assert_eq!(
        hints(NOTES, lines(0, 30)),
        json!([
            { "position": { "line": 5, "character": 18 }, "label": "= Harbour notes", "paddingLeft": true },
            { "position": { "line": 11, "character": 55 }, "label": "= success", "paddingLeft": true },
            { "position": { "line": 16, "character": 26 }, "label": "= [3 items]", "tooltip": team, "paddingLeft": true },
            { "position": { "line": 17, "character": 5 }, "label": "= Ana, … (3)", "paddingLeft": true },
        ]),
        "the list written alone failed: no hint"
    );
}

#[test]
fn only_the_hints_in_the_range_are_given() {
    let positions = |range| {
        let hints = hints(NOTES, range);
        let lines: Vec<Value> = hints
            .as_array()
            .expect("a list")
            .iter()
            .map(|hint| hint["position"]["line"].clone())
            .collect();
        lines
    };
    assert_eq!(positions(lines(10, 16)), [json!(11), json!(16)]);
    let up_to_the_brace = Range {
        start: Position {
            line: 5,
            character: 18,
        },
        end: Position {
            line: 5,
            character: 18,
        },
    };
    assert_eq!(positions(up_to_the_brace), [json!(5)], "its ends are in it");
    let before = Range {
        end: Position {
            line: 5,
            character: 17,
        },
        ..up_to_the_brace
    };
    assert_eq!(positions(before), Vec::<Value>::new());
}

#[test]
fn a_document_that_couldnt_be_processed_has_none() {
    let processor = processed("# Notes\n");
    let none = asking("# Notes\n", &processor, &[], |asked| {
        inlay_hints(
            lines(0, 9),
            &Asked {
                processor: None,
                ..*asked
            },
        )
    });
    assert!(none.is_empty());
}

#[test]
fn what_the_hints_show_is_their_labels_in_written_order() {
    let labels: Vec<String> = values_shown(&processed(NOTES))
        .into_iter()
        .map(|shown| shown.label)
        .collect();
    assert_eq!(
        labels,
        [
            "= Harbour notes",
            "= success",
            "= [3 items]",
            "= Ana, … (3)"
        ]
    );
}

#[test]
fn an_interpolation_or_an_expression_ends_at_its_closing_token() {
    let token = |kind, line, column, end: Option<usize>| SyntaxToken {
        kind,
        range: compmd::diagnostic::Range {
            start: Location::new(line, column),
            end: end.map(|end| Location::new(line, end)),
        },
        unknown: false,
    };
    let tokens = [
        token(TokenKind::InterpolationBrace, 1, 1, Some(3)),
        token(TokenKind::InterpolationPath, 1, 3, Some(4)),
        token(TokenKind::InterpolationBrace, 1, 4, Some(6)),
        token(TokenKind::ExpressionParen, 2, 4, Some(5)),
        token(TokenKind::ExpressionParen, 2, 9, None),
        token(TokenKind::ComponentName, 3, 1, Some(4)),
    ];
    let ends = ends(&tokens);
    assert_eq!(ends.get(&Location::new(1, 3)), Some(&Location::new(1, 6)));
    assert_eq!(ends.len(), 1, "a closer without an end places nothing");
}

#[test]
fn a_client_takes_refreshes_only_when_it_says_so() {
    let capabilities = |workspace: Value| -> ClientCapabilities {
        serde_json::from_value(json!({ "workspace": workspace })).expect("capabilities")
    };
    assert!(takes_refreshes(&capabilities(
        json!({ "inlayHint": { "refreshSupport": true } })
    )));
    assert!(!takes_refreshes(&capabilities(
        json!({ "inlayHint": { "refreshSupport": false } })
    )));
    assert!(!takes_refreshes(&capabilities(json!({ "inlayHint": {} }))));
    assert!(!takes_refreshes(&capabilities(json!({}))));
    assert!(!takes_refreshes(&ClientCapabilities::default()));
}
