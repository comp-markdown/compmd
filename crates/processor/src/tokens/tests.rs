use std::rc::Rc;

use super::*;
use crate::context::RenderContext;
use crate::{Ambient, ParseOptions, Processor};

/// The tokens of `text`, each as its text and its kind's name, with `?` after an unknown name.
fn tokens(text: &str) -> Vec<String> {
    let context = RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    };
    let processor = Processor::new("a.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve((), (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&crate::expand::Options::default());
    let lines: Vec<Vec<char>> = text.lines().map(|line| line.chars().collect()).collect();
    processor
        .tokens()
        .iter()
        .map(|token| {
            let line = &lines[token.range.start.line - 1];
            let end = token.range.end.map_or(line.len() + 1, |end| end.column);
            let spanned: String = line[token.range.start.column - 1..end - 1].iter().collect();
            format!(
                "{spanned} {}{}",
                token.kind.name(),
                if token.unknown { "?" } else { "" }
            )
        })
        .collect()
}

#[test]
fn a_container_its_params_and_its_closer() {
    assert_eq!(
        tokens(
            "::: box {md:gap=s .wide #top title=\"A {{md.doc.name}} b\" n=(1 + 2) open data=[1]}\nText.\n::: /box\n"
        ),
        [
            "::: component-marker",
            "box component-name",
            "md breakpoint",
            "gap param-key",
            "s param-bare",
            ".wide param-class",
            "#top param-id",
            "title param-key",
            "\"A  param-string",
            "{{ interpolation-brace",
            "md.doc.name interpolation-path",
            "}} interpolation-brace",
            " b\" param-string",
            "n param-key",
            "( expression-paren",
            ") expression-paren",
            "open param-key",
            "data param-key",
            "::: component-marker",
            "/ component-marker",
            "box component-name",
        ]
    );
}

#[test]
fn a_name_that_stands_for_no_component_is_unknown() {
    assert_eq!(
        tokens(":: carx[A card]\n"),
        [":: component-marker", "carx component-name?"]
    );
}

#[test]
fn continuations_slots_and_a_broken_line() {
    assert_eq!(
        tokens("::: grid {title=\"x}\n::: + lg:gap=l\n\n:: #footer\nEnd.\n::: /grid\n")[..6],
        [
            "::: component-marker",
            "grid component-name",
            "::: component-marker",
            "+ component-marker",
            "lg breakpoint",
            "gap param-key",
        ]
    );
    assert_eq!(
        tokens("::: 1box\n"),
        ["::: component-marker"],
        "a line with no form and no name"
    );
    assert_eq!(
        tokens("::: box x\n"),
        ["::: component-marker", "box component-name"],
        "a malformed line keeps its name"
    );
}

#[test]
fn inline_components_interpolations_and_what_isnt_text() {
    assert_eq!(
        tokens(
            "A :badge[new :mark[x]]{tone=info}{{md.doc.name}}, `:not[this]` and \\:no[t] {{ a.b }}.\n"
        ),
        [
            ": component-marker",
            "badge component-name?",
            ": component-marker",
            "mark component-name?",
            "tone param-key",
            "info param-bare",
            "{{ interpolation-brace",
            "md.doc.name interpolation-path",
            "}} interpolation-brace",
            "{{ interpolation-brace",
            "a.b interpolation-path",
            "}} interpolation-brace",
        ]
    );
    assert_eq!(tokens("a::b[c] x:y[z]\n"), Vec::<String>::new());
    assert_eq!(
        tokens(":: figure[A :mark[sky]] {src=a.jpg}\n")[2..4],
        [": component-marker", "mark component-name?"]
    );
}

#[test]
fn the_frontmatters_md_key_in_yaml_and_toml() {
    assert_eq!(
        tokens("---\nmd:\n  lang: en\ntitle: x\n---\n"),
        ["md frontmatter-key"]
    );
    assert_eq!(
        tokens(
            "+++\nmd.version = 1\n[md]\nlang = \"en\"\n[[md.data]]\nfrom = \"a.csv\"\nmdx = 1\n+++\n"
        ),
        [
            "md frontmatter-key",
            "md frontmatter-key",
            "md frontmatter-key"
        ]
    );
}

#[test]
fn a_definitions_fence_and_body_and_a_signature_fence() {
    assert_eq!(
        tokens(
            "```compmd component=card {gap=s}\nparams:\n  title: string\n---\n# {{title}}\n```\n\n```compmd component=card signature\n---\n```\n\n```js component=chart runtime=react\nexport default () => null;\n```\n"
        ),
        [
            "component param-key",
            "card component-name",
            "gap param-key",
            "s param-bare",
            "{{ interpolation-brace",
            "title interpolation-path",
            "}} interpolation-brace",
            "component param-key",
            "card component-name",
            "signature param-key",
            "component param-key",
            "chart component-name",
            "runtime param-key",
            "react param-bare",
        ]
    );
    assert_eq!(
        tokens("```js\n:: box\n```\n$$\n:: box\n$$\n"),
        Vec::<String>::new(),
        "code and math"
    );
}

#[test]
fn kinds_have_the_shared_names() {
    let names: Vec<&str> = TokenKind::ALL.iter().map(|kind| kind.name()).collect();
    assert_eq!(names.len(), 13);
    assert_eq!(
        serde_json::to_value(TokenKind::ParamKey).expect("a name"),
        "param-key"
    );
}
