//! The host-language components a renderer gets: their code from fences, files and packs, a pack's built module by
//! its place, each with its pack's trust, what no binding runs, and a name two documents give.

use super::*;
use crate::common;
use compmd::policy::{Policy, Trust};
use compmd::processor::Module;
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
            integrity: Some("sha512-KIT".into()),
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
            pack: "@acme/maps".into(),
            version: Version::new(1, 2, 0),
            path: "map.js".into(),
            integrity: Some("sha512-KIT".into()),
        })
    );
    assert_eq!(
        (pin.code.as_str(), &pin.module, &pin.trust),
        ("export default () => null;", &None, &permissive),
        "a pack's source file comes as text"
    );
    assert_eq!(
        (&own.module, &own.trust),
        (&None, &Trust::Strict),
        "the document's own"
    );
    assert_eq!(
        serde_json::to_value([map, own]).expect("serializes"),
        json!([
            {
                "name": "map", "language": "js", "code": "",
                "module": { "pack": "@acme/maps", "version": "1.2.0", "path": "map.js", "integrity": "sha512-KIT" },
                "trust": { "level": "permissive", "network": ["https://tiles.example.com"], "host": true },
            },
            { "name": "own", "language": "jsx", "code": "export default () => null;", "trust": { "level": "strict" } },
        ]),
        "the JSON a renderer reads, leaving out what is empty"
    );
}

#[test]
fn host_components_leave_out_what_no_binding_runs() {
    let input = "---\nmd:\n  import:\n    - from: ./gone.jsx\n      names: [e]\n---\n\n```vue component=a\nx\n```\n\n```svelte component=b runtime=svelte@4\nx\n```\n\n```svelte component=c\n<p>c</p>\n```\n\n```compmd component=d\n---\nD\n```\n";
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
fn a_name_two_documents_give_is_the_processed_documents() {
    let rendered = render(
        "---\nmd:\n  import:\n    - from: ./kit.comp.md\n      names: [wrap]\n---\n\n```jsx component=chip\nexport default () => 'mine';\n```\n\n:: wrap[W]\n",
        Setup {
            files: vec![(
                "kit.comp.md",
                "---\nmd:\n  export:\n    names: [wrap]\n---\n\n```js component=chip\nexport default () => {};\n```\n\n```compmd component=wrap\n---\n:: chip[C]\n```\n",
            )],
            ..Setup::default()
        },
    );
    assert_eq!(rendered.components, [("chip".to_string(), Language::Jsx)]);
}
