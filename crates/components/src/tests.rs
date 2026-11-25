//! The same cases as `@compmd/vite`'s `props.test.ts` and `analysis.test.ts` had, and the conformance cases that need
//! component analysis.

use std::collections::HashMap;
use std::path::Path;

use compmd::analysis::{Prop, PropType};
use compmd::{Ambient, ParseOptions, Processor};
use serde_json::Value;

use super::*;

fn react(code: &str, tsx: bool) -> Option<Props> {
    ComponentReader.props(if tsx { Language::Tsx } else { Language::Jsx }, code)
}

fn svelte(code: &str) -> Option<Props> {
    ComponentReader.props(Language::Svelte, code)
}

fn prop(name: &str) -> Prop {
    Prop {
        name: name.into(),
        kind: None,
    }
}

fn typed(name: &str, kind: PropType) -> Prop {
    Prop {
        name: name.into(),
        kind: Some(kind),
    }
}

fn names(props: Option<Props>) -> Option<Vec<String>> {
    props.map(|props| props.props.into_iter().map(|prop| prop.name).collect())
}

#[test]
fn reads_the_names_a_default_export_destructures_and_a_rest() {
    assert_eq!(
        react(
            "export default function Badge({ label, tone = 'info', ...more }) { return null; }",
            false
        ),
        Some(Props {
            props: vec![prop("label"), prop("tone")],
            rest: true
        })
    );
    assert_eq!(
        react("export default () => null;", false),
        Some(Props::default())
    );
    assert_eq!(
        names(react("export default (({ a, 'b-c': b }) => null);", false)),
        Some(vec!["a".into(), "b-c".into()])
    );
    assert_eq!(
        names(react(
            "export default function ({ a }) { return null; }",
            false
        )),
        Some(vec!["a".into()])
    );
}

#[test]
fn takes_the_types_a_type_literal_an_interface_or_an_alias_states() {
    let source = [
        "import type { ReactNode } from 'react';",
        "interface Props { title: string; count?: number; tone: 'info' | 'warn'; open: boolean | undefined; footer: ReactNode; meta: object }",
        "const Card = ({ title }: Props) => null;",
        "export default Card;",
    ]
    .join("\n");
    assert_eq!(
        react(&source, true),
        Some(Props {
            props: vec![
                typed("title", PropType::String),
                typed("count", PropType::Number),
                typed(
                    "tone",
                    PropType::Enum {
                        members: vec!["info".into(), "warn".into()]
                    }
                ),
                typed("open", PropType::Boolean),
                typed("footer", PropType::Slot),
                prop("meta"),
            ],
            rest: false,
        })
    );
    assert_eq!(
        names(react(
            "type P = { a: string } & { b: number };\nexport default function X(props: P) { return null; }",
            true
        )),
        Some(vec!["a".into(), "b".into()])
    );
}

#[test]
fn reads_the_other_types_it_knows_and_leaves_the_rest_untyped() {
    let source = "export interface Props { a: (string); b: 'x'; c: true | false; d: null | number; e: React.ReactNode; f: JSX.Element; g: 1 | 2; h: Array<string>; i: 'x' | 1 }\nexport default function Card(props: Props) { return null; }";
    assert_eq!(
        react(source, true).map(|props| props.props),
        Some(vec![
            typed("a", PropType::String),
            typed(
                "b",
                PropType::Enum {
                    members: vec!["x".into()]
                }
            ),
            typed("c", PropType::Boolean),
            typed("d", PropType::Number),
            typed("e", PropType::Slot),
            typed("f", PropType::Slot),
            prop("g"),
            prop("h"),
            prop("i"),
        ])
    );
    let alias = "export type Props = ({ a: string });\nexport const Card = (props: Props) => null;\nexport default Card;";
    assert_eq!(names(react(alias, true)), Some(vec!["a".into()]));
}

#[test]
fn cant_read_what_it_doesnt_see() {
    assert_eq!(
        react("export function Badge() { return null; }", false),
        None,
        "no default export"
    );
    assert_eq!(
        react(
            "export default memo(function Badge({ a }) { return null; });",
            false
        ),
        None,
        "a wrapped one"
    );
    assert_eq!(
        react(
            "export default function Badge(props) { return null; }",
            false
        ),
        None,
        "an untyped props object"
    );
    assert_eq!(
        react(
            "export default function Badge({ [key]: a }) { return null; }",
            false
        ),
        None,
        "a computed key"
    );
    assert_eq!(react("export default Missing;", false), None);
    let generic =
        "interface P<T> { a: T }\nexport default function X(props: P<string>) { return null; }";
    assert_eq!(react(generic, true), None, "a generic type");
    let extended = "interface Base { a: string }\ninterface P extends Base { b: string }\nexport default function X(props: P) { return null; }";
    assert_eq!(
        react(extended, true),
        None,
        "an interface that extends another"
    );
    assert_eq!(
        react(
            "interface P { [key: string]: string }\nexport default function X(props: P) { return null; }",
            true
        ),
        None
    );
    assert_eq!(
        react("export default function (", false),
        None,
        "can't be parsed"
    );
}

#[test]
fn reads_what_props_is_given_typed_or_not() {
    assert_eq!(
        svelte(
            "<script>\n  let { title, icon = 'info', children, footer } = $props();\n</script>\n<h3>{title}</h3>"
        ),
        Some(Props {
            props: vec![
                prop("title"),
                prop("icon"),
                prop("children"),
                prop("footer")
            ],
            rest: false
        })
    );
    let typed_script = "<script lang=\"ts\">\n  import type { Snippet } from 'svelte';\n  interface Props { value: number; children?: Snippet }\n  let { value, children }: Props = $props();\n</script>\n";
    assert_eq!(
        svelte(typed_script),
        Some(Props {
            props: vec![
                typed("value", PropType::Number),
                typed("children", PropType::Slot)
            ],
            rest: false
        })
    );
}

#[test]
fn gives_no_props_for_a_component_without_props_and_none_for_one_it_cant_read() {
    assert_eq!(svelte("<p>static</p>"), Some(Props::default()));
    assert_eq!(
        svelte("<script>let x = 1;\nlet y = other();</script>"),
        Some(Props::default())
    );
    assert_eq!(
        svelte("<script>let { a } = $props();"),
        None,
        "never closed"
    );
    assert_eq!(
        svelte("<script>let props = $props();</script>"),
        None,
        "an untyped props object"
    );
    assert_eq!(
        ComponentReader.props(Language::Js, "export default () => null;"),
        None
    );
    assert_eq!(
        ComponentReader.props(Language::Ts, "export default () => null;"),
        None
    );
}

/// The W104s of the document `a.comp.md` among `files`, processed with the reader, each `line:column message`.
fn analysed(files: &[(&str, &str)]) -> Vec<String> {
    let files: HashMap<String, String> = files
        .iter()
        .map(|(path, text)| ((*path).to_owned(), (*text).to_owned()))
        .collect();
    let text = files["a.comp.md"].clone();
    let resolved = Processor::new("a.comp.md", text)
        .parse(&ParseOptions::default())
        .analysing(ComponentReader)
        .resolve(files, (), &Ambient::default());
    resolved
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.code.to_string() == "W104")
        .map(|diagnostic| {
            format!(
                "{}:{} {}",
                diagnostic.range.start.line, diagnostic.range.start.column, diagnostic.message
            )
        })
        .collect()
}

#[test]
fn checks_an_imported_component_file_by_the_name_its_import_gives_it_and_leaves_one_it_cant_read() {
    let document = "---\nmd:\n  import:\n    - from: ./Card.svelte\n      names: [{name: Card, as: card}]\n    - from: ./Gauge.jsx\n      names: [{name: Gauge, as: gauge}]\n---\n\n```compmd component=card signature\nparams:\n  title: string\nslots: [default]\n```\n\n```compmd component=gauge signature\nparams:\n  value: number\n```\n";
    let found = analysed(&[
        ("a.comp.md", document),
        (
            "Card.svelte",
            "<script>\n  let { title } = $props();\n</script>\n",
        ),
        (
            "Gauge.jsx",
            "export default function Gauge(props) { return null; }",
        ),
    ]);
    assert_eq!(
        found,
        [
            "10:1 the signature declares the slot `default`, which Card.svelte has no prop for (`children`)"
        ]
    );
}

#[test]
fn leaves_a_component_whose_code_isnt_there() {
    let document = "```svelte component=empty\n```\n\n```compmd component=empty signature\nparams:\n  value: number\n```\n";
    assert_eq!(analysed(&[("a.comp.md", document)]), Vec::<String>::new());
}

/// Each conformance case that needs component analysis gives its diagnostics, processed with the reader.
#[test]
fn passes_the_conformance_cases_that_need_component_analysis() {
    let cases = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../conformance/resolve");
    let mut ran = 0;
    for entry in std::fs::read_dir(&cases).expect("the cases") {
        let case = entry.expect("a case").path();
        let Ok(meta) = std::fs::read_to_string(case.join("case.json")) else {
            continue;
        };
        if !meta.contains("component-analysis") {
            continue;
        }
        let text = std::fs::read_to_string(case.join("input.comp.md")).expect("its document");
        let expected: Value = serde_json::from_str(
            &std::fs::read_to_string(case.join("diagnostics.json")).expect("its diagnostics"),
        )
        .expect("JSON");
        let resolved = Processor::new("input.comp.md", text)
            .parse(&ParseOptions::default())
            .analysing(ComponentReader)
            .resolve((), (), &Ambient::default());
        let found: Vec<(String, usize, usize)> = resolved
            .diagnostics()
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.code.to_string(),
                    diagnostic.range.start.line,
                    diagnostic.range.start.column,
                )
            })
            .collect();
        let wanted: Vec<(String, usize, usize)> = expected
            .as_array()
            .expect("a list")
            .iter()
            .map(|diagnostic| {
                let number = |key: &str| {
                    usize::try_from(diagnostic[key].as_u64().expect("a number")).expect("small")
                };
                (
                    diagnostic["code"].as_str().expect("a code").to_owned(),
                    number("line"),
                    number("column"),
                )
            })
            .collect();
        let mut found = found;
        found.sort();
        let mut wanted = wanted;
        wanted.sort();
        assert_eq!(found, wanted, "{}", case.display());
        ran += 1;
    }
    assert_eq!(ran, 1, "the cases that need component analysis");
}
