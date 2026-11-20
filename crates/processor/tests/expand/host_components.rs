//! The host-language components a renderer gets: their code from fences, files and packs, a pack's built module by
//! its place, each with its pack's trust, what no binding runs, and a name two documents give.

use super::*;
use crate::common;
use compmd::policy::{Policy, Trust};
use compmd::processor::Module;
use compmd::tree::Print;
use serde_json::json;

#[test]
fn host_components_come_with_their_code_from_fences_files_and_packs() {
    let dir = common::conformance().join("examples/complete-example-expanded");
    let setup = common::Setup::of(&dir, &Value::Null);
    let processor = setup.process("input.comp.md", &setup.files["input.comp.md"]);
    let components = processor.host_components();
    let found: Vec<(&str, Language, Option<&str>)> = components
        .iter()
        .map(|c| (c.name.as_str(), c.language, c.runtime.as_deref()))
        .collect();
    assert_eq!(
        found,
        [
            ("board", Language::Jsx, None),
            ("chart", Language::Jsx, None),
            ("sparkline", Language::Jsx, Some("react@19")),
        ],
        "an imported file, a pack's file, a fence"
    );
    assert_eq!(components[0].code, setup.files["board.jsx"]);
    let pack = dir.join("registry/acme-charts-1.4.2/chart.jsx");
    assert_eq!(
        components[1].code,
        std::fs::read_to_string(pack).expect("chart.jsx")
    );
    assert!(
        components[2]
            .code
            .starts_with("export default function Sparkline")
    );
}

#[test]
fn a_packs_built_module_comes_by_its_place_and_components_have_their_packs_trust() {
    let kit = LoadedRelease {
        release: Release {
            version: Version::new(1, 2, 0),
            integrity: Some("sha512-KIT".parse().expect("an integrity")),
            broken: None,
        },
        files: HashMap::from([
            (
                "package.json".to_string(),
                r#"{"name": "@acme/maps", "version": "1.2.0"}"#.to_string(),
            ),
            (
                "index.comp.md".to_string(),
                "---\nmd:\n  import:\n    - from: ./map.js\n      names: [map]\n    - from: ./Pin.jsx\n      names: [{name: Pin, as: pin}]\n  export:\n    names: [map, pin]\n---\n\n```compmd component=map signature\nparams: {}\n```\n\n```compmd component=pin signature\nparams: {}\n```\n".to_string(),
            ),
            ("map.js".to_string(), "export default () => {};".to_string()),
            ("Pin.jsx".to_string(), "export default () => null;".to_string()),
        ]),
    };
    let packs = HashMap::from([("@acme/maps".to_string(), vec![kit])]);
    let input = "---\nmd:\n  import:\n    - from: \"@acme/maps@1\"\n      names: [map, pin]\n---\n\n```jsx component=own\nexport default () => null;\n```\n\n```compmd component=own signature\nparams: {}\n```\n\n:: map[Map]\n\n:: pin[Pin]\n\n:: own[Own]\n";
    let policy: Policy = serde_json::from_value(json!({ "packs": {
        "@acme/maps": { "level": "permissive", "network": ["https://tiles.example.com"], "host": true }
    } }))
    .expect("a policy");
    let expanded = Processor::new("input.comp.md", input)
        .parse(&ParseOptions::default())
        .resolve_with((), packs, &Ambient::default(), &policy)
        .validate(&[])
        .load(context(), (), ())
        .expand(&expand::Options::default());
    assert_eq!(expanded.diagnostics(), []);
    let components = expanded.host_components();
    let permissive = Trust::Permissive {
        network: vec!["https://tiles.example.com".into()],
        host: true,
    };
    let [map, own, pin] = &components[..] else {
        panic!("three components: {components:?}");
    };
    assert_eq!(
        (map.name.as_str(), map.code.as_str(), &map.trust),
        ("map", "", &permissive)
    );
    assert_eq!(
        map.module,
        Some(Module {
            place: "@acme/maps@1.2.0/map.js".parse().expect("a place"),
            integrity: Some("sha512-KIT".parse().expect("an integrity")),
        })
    );
    assert_eq!(
        (pin.code.as_str(), &pin.module, &pin.trust),
        ("export default () => null;", &None, &permissive),
        "a pack's source file comes as text"
    );
    assert_eq!(
        pin.place,
        Some("@acme/maps@1.2.0/Pin.jsx".parse().expect("a place")),
        "and by its place, for a module compiled ahead"
    );
    assert_eq!(
        (&own.module, &own.place, &own.trust),
        (&None, &None, &Trust::Strict),
        "the document's own"
    );
    assert_eq!(map.place, None, "a built module's place is its module's");
    assert_eq!(
        serde_json::to_value([map, own, pin]).expect("serializes"),
        json!([
            {
                "name": "map", "language": "js", "code": "",
                "module": { "pack": "@acme/maps", "version": "1.2.0", "path": "map.js", "integrity": "sha512-KIT" },
                "trust": { "level": "permissive", "network": ["https://tiles.example.com"], "host": true },
            },
            { "name": "own", "language": "jsx", "code": "export default () => null;", "trust": { "level": "strict" } },
            {
                "name": "pin", "language": "jsx", "code": "export default () => null;",
                "place": { "pack": "@acme/maps", "version": "1.2.0", "path": "Pin.jsx" },
                "trust": { "level": "permissive", "network": ["https://tiles.example.com"], "host": true },
            },
        ]),
        "the JSON a renderer reads, leaving out what is empty"
    );
}

#[test]
fn host_components_are_those_the_document_uses_in_any_branch() {
    let components = ["chart", "map", "rating", "stars", "unused"]
        .map(|name| format!("```js component={name}\nexport default () => {{}};\n```\n\n"))
        .concat();
    let input = format!(
        "{components}```compmd component=wrap\n---\n:: map[M]\n```\n\n:: chart[C]\n\n::: if {{when=(false)}}\n:: rating[R]\n::: /if\n\n````compmd {{render=output}}\n```js component=stars\nexport default () => {{}};\n```\n\n```js component=unused\nexport default () => {{}};\n```\n\n:: stars[S]\n````\n"
    );
    let rendered = Processor::new("input.comp.md", input.as_str())
        .parse(&ParseOptions::default())
        .resolve((), (), &Ambient::default())
        .validate(&[])
        .load(context(), (), ())
        .expand(&expand::Options::default());
    let listed: Vec<(String, Option<usize>)> = rendered
        .host_components()
        .into_iter()
        .map(|c| (c.name, c.example))
        .collect();
    // `rating` is in a branch this render doesn't show, which another may: it is listed. `map` only through `wrap`,
    // which no one uses, the document's own `stars` and `unused`, and the example's `unused`: none is.
    assert_eq!(
        listed,
        [
            ("chart".into(), None),
            ("rating".into(), None),
            ("stars".into(), Some(1))
        ]
    );
}

#[test]
fn host_components_leave_out_what_no_binding_runs() {
    let input = "---\nmd:\n  import:\n    - from: ./gone.jsx\n      names: [e]\n---\n\n```vue component=a\nx\n```\n\n```svelte component=b runtime=svelte@4\nx\n```\n\n```svelte component=c\n<p>c</p>\n```\n\n```compmd component=d\n---\nD\n```\n\n:: a[A]\n\n:: b[B]\n\n:: c[C]\n\n:: d[D]\n\n:: e[E]\n";
    let rendered = Processor::new("input.comp.md", input)
        .parse(&ParseOptions::default())
        .resolve((), (), &Ambient::default())
        .validate(&[])
        .load(context(), (), ())
        .expand(&expand::Options::default());
    let names: Vec<(String, Language)> = rendered
        .host_components()
        .into_iter()
        .map(|c| (c.name, c.language))
        .collect();
    assert_eq!(names, [("c".into(), Language::Svelte)]);
}

#[test]
fn host_components_say_when_their_uses_print_their_fallback() {
    let input = "```js component=map\nexport default () => {};\n```\n\n```compmd component=map signature\nprint: fallback\n```\n\n```js component=chart\nexport default () => {};\n```\n\n```compmd component=chart signature\nprint: drawn\n```\n\n:: map[Map]\n\n:: chart[Chart]\n";
    let rendered = Processor::new("input.comp.md", input)
        .parse(&ParseOptions::default())
        .resolve((), (), &Ambient::default())
        .validate(&[])
        .load(context(), (), ())
        .expand(&expand::Options::default());
    let printing: Vec<(String, Print)> = rendered
        .host_components()
        .into_iter()
        .map(|c| (c.name, c.print))
        .collect();
    assert_eq!(
        printing,
        [
            ("chart".into(), Print::Drawn),
            ("map".into(), Print::Fallback)
        ]
    );
    let json = serde_json::to_value(rendered.host_components()).expect("JSON");
    assert!(json[0].get("print").is_none(), "drawn is left out");
    assert_eq!(json[1]["print"], json!("fallback"));
}

#[test]
fn a_name_two_documents_give_is_the_processed_documents() {
    let kit = || Setup {
        files: vec![(
            "kit.comp.md",
            "---\nmd:\n  export:\n    names: [wrap]\n---\n\n```js component=chip\nexport default () => {};\n```\n\n```compmd component=wrap\n---\n:: chip[C]\n```\n",
        )],
        ..Setup::default()
    };
    let document = |uses_its_own: &str| {
        format!(
            "---\nmd:\n  import:\n    - from: ./kit.comp.md\n      names: [wrap]\n---\n\n```jsx component=chip\nexport default () => 'mine';\n```\n\n:: wrap[W]\n{uses_its_own}"
        )
    };
    // Both are used: the processed document's.
    let both = render(&document("\n:: chip[Mine]\n"), kit());
    assert_eq!(both.components, [("chip".to_string(), Language::Jsx)]);
    // Only the kit's, through `wrap`: the kit's.
    let theirs = render(&document(""), kit());
    assert_eq!(theirs.components, [("chip".to_string(), Language::Js)]);
}
