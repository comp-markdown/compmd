use super::*;
use crate::features::testing::processed;

/// The symbols of `text` as an outline: each `name (kind)`, its children indented under it.
fn outline(text: &str) -> Vec<String> {
    fn lines(symbols: &[DocumentSymbol], depth: usize, out: &mut Vec<String>) {
        for symbol in symbols {
            out.push(format!(
                "{}{} ({:?})",
                "  ".repeat(depth),
                symbol.name,
                symbol.kind
            ));
            lines(
                symbol.children.as_deref().unwrap_or_default(),
                depth + 1,
                out,
            );
        }
    }
    let processor = processed(text);
    let mut out = Vec::new();
    lines(
        &symbols(&processor.tree().children, text, Encoding::Utf16),
        0,
        &mut out,
    );
    out
}

#[test]
fn headings_hold_what_follows_them_up_to_the_next_of_their_level() {
    let text = "# Notes `v2`\n\n:: toc\n\n## One\n\n::: box\nA.\n\n## Inside\n\n:: #footer\nB.\n::: /box\n\n## Two\n\n# Next\n";
    assert_eq!(
        outline(text),
        [
            "Notes v2 (String)",
            "  toc (Object)",
            "  One (String)",
            "    box (Object)",
            "      Inside (String)",
            "      #footer (Field)",
            "  Two (String)",
            "Next (String)",
        ]
    );
}

#[test]
fn definitions_hold_their_body_and_a_heading_runs_to_the_last_it_holds() {
    let text = "```compmd component=note\n---\n## Body\n```\n\n```js component=chart\nexport default () => null;\n```\n\n```compmd component=chart signature\n---\n```\n\n> - :: leaf[In a quote]\n";
    assert_eq!(
        outline(text),
        [
            "note (Class)",
            "  Body (String)",
            "chart (Class)",
            "chart (Interface)",
            "leaf (Object)"
        ]
    );
    let processor = processed("# A\n\n::: box\nText.\n::: /box\n");
    let found = symbols(
        &processor.tree().children,
        "# A\n\n::: box\nText.\n::: /box\n",
        Encoding::Utf16,
    );
    assert_eq!(
        found[0].range.end.line, 4,
        "the heading runs to the box's closer"
    );
    assert_eq!(found[0].selection_range.end.line, 0);
}
