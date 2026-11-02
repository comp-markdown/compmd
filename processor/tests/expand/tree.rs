//! The expanded tree: every mdast node, origins, URLs, paths that fail, and the expansion limit.

use super::*;
use crate::common::at;
use serde_json::json;

#[test]
fn every_mdast_node_keeps_its_shape() {
    let input = "---\nk: guide\nmd:\n  syntax: [wiki-links]\n---\n\n> [!NOTE]\n> A *quote* with ~~old~~ **text**,\\\n> `code`, $x$ and <b>html</b>.\n\n```rust\nfn main() {}\n```\n\n$$\nx^2\n$$\n\n| A | B |\n| - | - |\n| [[Page]] | [ref][r] |\n\n---\n\n![Logo][r] and a note.[^n]\n\n[^n]: The note.\n\n[r]: https://example.com/{{md.doc.k}}\n";
    let rendered = render(input, Setup::default());
    assert_eq!(rendered.found, []);
    assert_eq!(
        types(&rendered.tree),
        [
            "blockquote",
            "code",
            "math",
            "table",
            "thematicBreak",
            "paragraph",
            "footnoteDefinition",
            "definition"
        ]
    );
    let quote = &rendered.tree["children"][0];
    assert_eq!(quote["alert"], "note");
    let line = &quote["children"][0]["children"];
    let kinds: Vec<&str> = line
        .as_array()
        .expect("inline nodes")
        .iter()
        .filter_map(|node| node["type"].as_str())
        .collect();
    assert_eq!(
        kinds,
        [
            "text",
            "emphasis",
            "text",
            "delete",
            "text",
            "strong",
            "text",
            "break",
            "inlineCode",
            "text",
            "inlineMath",
            "text",
            "html",
            "text",
            "html",
            "text"
        ]
    );
    let row = &rendered.tree["children"][3]["children"][1]["children"];
    assert_eq!(row[0]["children"][0]["type"], "wikiLink");
    assert_eq!(row[1]["children"][0]["type"], "linkReference");
    let paragraph = &rendered.tree["children"][5]["children"];
    assert_eq!(paragraph[0]["type"], "imageReference");
    assert_eq!(paragraph[2]["type"], "footnoteReference");
    assert_eq!(
        rendered.tree["children"][7]["url"],
        "https://example.com/guide"
    );
    assert_eq!(rendered.tree["children"][7].get("urlParts"), None);
}

#[test]
fn origins_cover_every_node_and_what_recovery_closed() {
    let input = "```compmd component=card\nslots: [default, foot]\n---\n{{md.slot}}\n\n{{md.slot.foot}}\n```\n\n# Title\n\n- item with `code`\n\n> quote\n\n::: card\nBody.\n:: #foot\nFoot.\n";
    let options = expand::Options {
        origins: true,
        ..expand::Options::default()
    };
    let rendered = render(
        input,
        Setup {
            options,
            ..Setup::default()
        },
    );
    assert_eq!(rendered.found, [at("F004", 15, 1)]);
    let card = &rendered.tree["children"][3];
    assert_eq!(card["name"], "box");
    assert_eq!(card["origin"]["start"]["line"], 15);
    assert_eq!(card["origin"]["end"]["line"], 18);
    assert_eq!(card["children"][1]["origin"]["start"]["line"], 18);
    let item = &rendered.tree["children"][1]["children"][0];
    assert_eq!(item["origin"]["start"]["line"], 11);
    assert_eq!(
        item["children"][0]["children"][1]["origin"]["start"]["column"],
        13
    );
}

#[test]
fn urls_take_their_values_and_never_their_scheme() {
    let input = "---\nlogo: ./logo.png\nbad: \"javascript:alert(1)\"\nlist: [a]\nsite: https://a.org\n---\n\n![Logo]({{md.doc.logo}}) ![Trap]({{md.doc.bad}})\n\n[a]({{nope}}) [b](https://a.org/{{md.doc.list}}) [c]({{md.doc.site}}/x)\n\n[t]: {{md.doc.bad}}\n";
    let rendered = render(input, Setup::default());
    assert_eq!(
        rendered.found,
        [at("F014", 10, 7), at("W109", 8, 26), at("W109", 12, 1)]
    );
    let images = &rendered.tree["children"][0]["children"];
    assert_eq!(images[0]["url"], "./logo.png");
    assert_eq!(images[1], json!({ "type": "text", "value": " Trap" }));
    let links = &rendered.tree["children"][1]["children"];
    assert_eq!(links[0]["url"], "{{nope}}");
    assert_eq!(links[2]["url"], "https://a.org/{{md.doc.list}}");
    assert_eq!(links[4]["url"], "https://a.org/x");
    assert_eq!(rendered.tree["children"][2]["url"], "");
}

#[test]
fn paths_that_fail_when_evaluated_are_shown_as_written() {
    let input = "---\nmd:\n  data:\n    - from: ./team.yaml\n      as: team\n---\n\n{{md.data.team.missing}} and {{md.slot}}.\n\n{{md.data.team.lead}}\n";
    let rendered = render(
        input,
        Setup {
            files: vec![("team.yaml", "lead: Ana\n")],
            ..Setup::default()
        },
    );
    assert_eq!(rendered.found, [at("F014", 8, 32), at("F024", 8, 3)]);
    assert_eq!(
        text(&rendered.tree),
        "{{md.data.team.missing}} and {{md.slot}}.|Ana"
    );
}

#[test]
fn the_limit_stops_templates_and_invocations() {
    let input = "```compmd component=pair\n---\nA\n\nB\n```\n\n::: each {in=([1, 2, 3]) as=i}\n:: pair[P]\n::: /each\n\n```compmd component=wide\nparams:\n  rows: array\n  as: name\nslots:\n  default: {each: rows, as: as}\n---\n{{md.slot}}\n```\n\n::: wide {rows=([1, 2, 3]) as=r}\nRow {{r}}\n::: /wide\n";
    let limited = |limit| {
        render(
            input,
            Setup {
                options: expand::Options {
                    limit,
                    ..expand::Options::default()
                },
                ..Setup::default()
            },
        )
    };
    let rendered = limited(4);
    assert_eq!(rendered.found, [at("F034", 9, 1)]);
    assert_eq!(text(&rendered.tree), "P|Row {{r}}");
    let unlimited = limited(1000);
    assert_eq!(unlimited.found, []);
    assert_eq!(text(&unlimited.tree), "AB|AB|AB|Row 1Row 2Row 3");
}
