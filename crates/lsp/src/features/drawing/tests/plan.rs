//! The plan as a whole: the lines asked for, its version, the settings (decision 6), reveal modes (decision 5) and glyph
//! sets.

use lsp_types::Uri;

use super::*;
use crate::encoding::Encoding;
use crate::places::Places;

/// A document with an element of each kind decision 11 names.
const NOTES: &str = "# Notes\n\n- [ ] ropes\n\n> [!NOTE]\n> Tide.\n\n---\n\n| A |\n| - |\n\n```rust\nx\n```\n\n::: box {padding=s}\n:: box[A] {padding=s}\n::: /box\n";

#[test]
fn only_the_lines_asked_for_are_planned() {
    let lines = |first, last| Range::new(Position::new(first, 0), Position::new(last, 0));
    let planning = Planning {
        primitives: all(),
        render: RenderSettings::default(),
    };
    assert_eq!(
        described(&plan_of(NOTES, lines(4, 6), &planning)),
        [
            "replace 4:0-4:1 \"▋\" compmd.callout.note reveal 4",
            "replace 4:2-4:9 \"ℹ Note\" compmd.callout.note reveal 4",
            "replace 5:0-5:1 \"▋\" compmd.callout.note reveal 5",
        ]
    );
    let in_the_box = described(&plan_of(NOTES, lines(17, 18), &planning));
    assert_eq!(
        in_the_box,
        [
            "prefix 17 \"│ \" compmd.container.border",
            "style 17:3-17:6 compmd.component",
            "style 17:10-17:21 compmd.param",
        ],
        "a box's marks on those lines only"
    );
}

#[test]
fn the_plan_is_of_the_servers_version_and_empty_without_a_processor() {
    let uri: Uri = "file:///site/a.comp.md".parse().expect("a URI");
    let places = Places::default();
    let asked = Asked {
        processor: None,
        uri: &uri,
        plain_markdown_file: false,
        path: std::path::Path::new("/site/a.comp.md"),
        index: None,
        links: crate::settings::LinkSettings::default(),
        text: NOTES,
        version: 7,
        encoding: Encoding::Utf16,
        places: &places,
        pack_components: &[],
        unregistered: None,
    };
    let range = Range::new(Position::new(0, 0), Position::new(20, 0));
    assert_eq!(
        plan(range, &Planning::default(), &asked),
        Plan {
            version: 7,
            marks: Vec::new()
        }
    );
}

#[test]
fn render_enabled_false_draws_nothing() {
    assert_eq!(
        drawn_with(NOTES, json!({ "enabled": false })),
        Vec::<String>::new()
    );
}

#[test]
fn each_elements_switch_turns_it_off() {
    let off = json!({
        "heading": false, "code": false, "table": false, "list": false, "checkbox": false, "quote": false,
        "callout": false, "rule": false, "container": false, "component": false, "param": false,
    });
    assert_eq!(drawn_with(NOTES, off), Vec::<String>::new());
    let frontmatter = "---\nmd:\n  lang: en\n---\n";
    assert_eq!(
        drawn_with(frontmatter, json!({ "param": false, "frontmatter": false })),
        Vec::<String>::new()
    );
    assert_eq!(
        drawn_with(frontmatter, json!({ "frontmatter": false })),
        ["style 1:0-1:2 compmd.frontmatter.md"]
    );
}

#[test]
fn a_callout_switched_off_is_a_plain_quote_and_a_quote_switched_off_keeps_its_callouts_title() {
    let alert = "> [!NOTE]\n> Tide.\n";
    assert_eq!(
        drawn_with(alert, json!({ "callout": false })),
        [
            "replace 0:0-0:1 \"▋\" compmd.quote.1 reveal 0",
            "replace 1:0-1:1 \"▋\" compmd.quote.1 reveal 1",
        ]
    );
    assert_eq!(
        drawn_with(alert, json!({ "quote": false })),
        ["replace 0:2-0:9 \"ℹ Note\" compmd.callout.note reveal 0"]
    );
}

#[test]
fn an_elements_options_change_how_it_is_drawn() {
    let code = "```rust\nx\n```\n";
    assert_eq!(
        drawn_with(code, json!({ "code": { "language": "fence" } }))[0],
        "replace 0:0-0:7 \"rust\" compmd.code.language reveal 0"
    );
    assert_eq!(
        drawn_with(code, json!({ "code": { "language": "off" } }))[0],
        "hide 0:0-0:7 reveal 0"
    );
    assert_eq!(
        drawn_with("# A\n", json!({ "heading": { "fill": false } }))[1],
        "style 0:0-0:3 compmd.heading.1"
    );
    let table = "| A |\n| - |\n";
    assert!(
        drawn_with(table, json!({ "table": { "borders": false } }))
            .iter()
            .all(|mark| !mark.starts_with("line")),
        "no borders"
    );
    assert_eq!(
        drawn_with("# A\n", json!({ "heading": { "marker": false } }))[0],
        "style 0:0-0:1 compmd.heading.marker",
        "the # as written, set back"
    );
    assert_eq!(
        drawn_with("A\n=\n", json!({ "heading": { "marker": false } })),
        [
            "style 0:0-1:1 compmd.heading.1 whole",
            "style 1:0-1:1 compmd.heading.marker"
        ],
        "a setext heading's underline as written, set back"
    );
    assert_eq!(
        drawn_with("```rust\nx\n```\n", json!({ "code": { "fence": false } })),
        ["style 0:3-0:7 compmd.code.language"],
        "the fences as written"
    );
    let boxed = "::: box\nIn.\n::: /box\n";
    assert!(
        drawn_with(boxed, json!({ "container": { "gutter": false } }))
            .iter()
            .all(|mark| !mark.starts_with("prefix")),
        "no gutter"
    );
    let front = "---\ntitle: Notes\n---\n";
    assert_eq!(
        drawn_with(front, json!({ "frontmatter": { "rules": false } })),
        ["style 1:0-2:0 compmd.frontmatter whole reveal 0-2"],
        "set back, its fences as written"
    );
    assert!(
        drawn_with(front, json!({ "frontmatter": { "dim": false } }))
            .iter()
            .all(|mark| !mark.contains("compmd.frontmatter whole")),
        "its rules, its lines as written"
    );
    let container = "::: box {padding=s}\n::: /box\n";
    assert!(
        drawn_with(container, json!({ "param": false }))
            .iter()
            .all(|mark| !mark.contains("compmd.param")),
        "the params left as written"
    );
}

#[test]
fn reveal_names_the_parts_lines_the_elements_or_none() {
    let quote = "> One\n> Two\n";
    assert_eq!(
        drawn_with(quote, json!({ "reveal": "element" })),
        [
            "replace 0:0-0:1 \"▋\" compmd.quote.1 reveal 0-1",
            "replace 1:0-1:1 \"▋\" compmd.quote.1 reveal 0-1",
        ]
    );
    assert_eq!(
        drawn_with(quote, json!({ "reveal": "off" })),
        [
            "replace 0:0-0:1 \"▋\" compmd.quote.1",
            "replace 1:0-1:1 \"▋\" compmd.quote.1",
        ]
    );
}

#[test]
fn glyphs_come_from_the_set_and_the_writers_own() {
    let text = "## A\n\n```rust\nx\n```\n\n> [!TIP]\n> Early.\n";
    // A mark's text is written as Rust writes a string, a Nerd Font's icons escaped.
    let nerd = drawn_with(
        text,
        json!({ "glyphs": "nerd", "code": { "language": "above" } }),
    );
    assert_eq!(
        nerd[0],
        "replace 0:0-0:2 \"\\u{f0ca3}\" compmd.heading.marker reveal 0"
    );
    assert_eq!(
        nerd[3],
        "line above 2 [\"\\u{e7a8} rust\" compmd.code.language] reveal 2"
    );
    assert_eq!(
        nerd[6],
        "replace 6:2-6:8 \"\\u{f0336} Tip\" compmd.callout.tip reveal 6"
    );
    let ascii = drawn_with(
        text,
        json!({ "glyphs": "ascii", "glyph": { "heading.2": "§" }, "code": { "language": "above" } }),
    );
    assert_eq!(
        ascii[0],
        "replace 0:0-0:2 \"§\" compmd.heading.marker reveal 0"
    );
    assert_eq!(
        ascii[5],
        "replace 6:0-6:1 \"|\" compmd.callout.tip reveal 6"
    );
}

#[test]
fn positions_are_in_the_encoding_agreed() {
    let text = "# Été 😀\n\n`a`\n";
    let processor = processed(text);
    let uri: Uri = "file:///site/a.comp.md".parse().expect("a URI");
    let places = Places::default();
    let asked = Asked {
        processor: Some(&processor),
        uri: &uri,
        plain_markdown_file: false,
        path: std::path::Path::new("/site/a.comp.md"),
        index: None,
        links: crate::settings::LinkSettings::default(),
        text,
        version: 1,
        encoding: Encoding::Utf8,
        places: &places,
        pack_components: &[],
        unregistered: None,
    };
    let planning = Planning {
        primitives: all(),
        render: RenderSettings::default(),
    };
    let range = Range::new(Position::new(0, 0), Position::new(3, 0));
    let utf8 = described(&plan(range, &planning, &asked));
    assert_eq!(utf8[1], "style 0:0-0:12 compmd.heading.1 whole", "bytes");
}

#[test]
fn a_plain_markdown_file_is_drawn_as_markdown_unless_markdown_files_is_off() {
    let text = "# Notes\n\n::: box\nText.\n::: /box\n";
    let range = Range::new(Position::new(0, 0), Position::new(5, 0));
    let drawn_in = |name: &str, render: RenderSettings| {
        let processor = crate::features::testing::processed_file(name, text);
        let planning = Planning {
            primitives: all(),
            render,
        };
        crate::features::testing::asking_file(name, text, &processor, &[], |asked| {
            described(&plan(range, &planning, asked))
        })
    };
    let markdown = drawn_in("notes.md", RenderSettings::default());
    assert!(markdown[1].contains("compmd.heading.1"), "{markdown:?}");
    assert!(
        markdown
            .iter()
            .all(|mark| !mark.contains("compmd.container")),
        "`:::` is text: {markdown:?}"
    );
    let compmd = drawn_in("notes.comp.md", RenderSettings::default());
    assert!(compmd.iter().any(|mark| mark.contains("compmd.container")));
    let off = RenderSettings {
        markdown_files: false,
        ..RenderSettings::default()
    };
    assert_eq!(drawn_in("notes.md", off.clone()), Vec::<String>::new());
    assert_eq!(drawn_in("notes.comp.md", off), compmd, "only `.md` files");
}
