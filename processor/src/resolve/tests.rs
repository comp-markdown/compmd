use super::*;
use crate::host::LoadedRelease;
use crate::parse::{ParseOptions, parse};
use serde_json::{Value, json};
use std::collections::HashMap;

/// The packs of a registry, by name, from JSON.
fn packs_of(packs: Value) -> HashMap<String, Vec<LoadedRelease>> {
    serde_json::from_value(packs).expect("packs")
}

/// `input.comp.md` among `files` resolved, with `packs` (as JSON) and the `ambient` packs.
fn resolved(files: &[(&str, &str)], packs: Value, ambient: &[&str]) -> Resolved {
    let files: HashMap<String, String> = files
        .iter()
        .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
        .collect();
    let packs = packs_of(packs);
    let text = &files["input.comp.md"];
    let ambient = Ambient {
        packs: ambient
            .iter()
            .map(|pack| pack.parse().expect("a pack's name"))
            .collect(),
        report: false,
    };
    let parsed = parse(text, &ParseOptions::default());
    let store = Store {
        files: &files,
        packs: &packs,
    };
    let host = Host {
        ambient: &ambient,
        policy: &Policy::default(),
    };
    resolve(
        "input.comp.md",
        text.clone(),
        parsed,
        store,
        host,
        &mut Parses::default(),
    )
}

/// The component `name` stands for in the processed document.
fn bound<'r>(resolved: &'r Resolved, name: &str) -> &'r Component {
    let main = resolved.document(resolved.main);
    let Some(Binding::Component(id, _)) = main.names.get(name) else {
        panic!("`{name}` is bound");
    };
    resolved.component(*id)
}

#[test]
fn a_document_is_parsed_again_only_when_its_text_changed() {
    let text =
        "---\nmd:\n  import:\n    - from: ./kit.comp.md\n      names: [card]\n---\n\n:: card[x]\n";
    let kit = |body: &str| format!("```compmd component=card\n---\n{body}\n```\n");
    let mut known = Parses::default();
    let mut resolve_with = |kit: &str| {
        let files = HashMap::from([("kit.comp.md".to_string(), kit.to_string())]);
        let store = Store {
            files: &files,
            packs: &(),
        };
        let parsed = parse(text, &ParseOptions::default());
        let resolved = resolve(
            "input.comp.md",
            text.to_string(),
            parsed,
            store,
            Host {
                ambient: &Ambient::default(),
                policy: &Policy::default(),
            },
            &mut known,
        );
        resolved
            .ids()
            .map(|id| Rc::clone(&resolved.document(id).root))
            .collect::<Vec<_>>()
    };
    let first = resolve_with(&kit("One."));
    let same = resolve_with(&kit("One."));
    let changed = resolve_with(&kit("Two."));
    assert_eq!(first.len(), 3, "core, the processed document and the kit");
    let shared = |one: &[Rc<Root>], other: &[Rc<Root>]| {
        one.iter()
            .zip(other)
            .filter(|(one, other)| Rc::ptr_eq(one, other))
            .count()
    };
    assert_eq!(shared(&first, &same), 2, "core and the kit are kept");
    assert_eq!(
        shared(&first, &changed),
        1,
        "the changed kit is parsed again"
    );
}

#[test]
fn core_is_built_in_even_when_the_host_has_a_pack_of_its_name() {
    let impostor = json!({ "@compmd/core": [{ "version": "1.0.0", "files": {
        "package.json": "{\"name\": \"@compmd/core\", \"version\": \"1.0.0\"}",
        "index.comp.md": "Not core.\n",
    } }] });
    let packs = packs_of(impostor.clone());
    let store = Store {
        files: &(),
        packs: &packs,
    };
    let manifest = Origin::Pack("@compmd/core@1.0.0/package.json".parse().expect("a place"));
    assert_eq!(store.read(&manifest), None, "the host isn't asked");
    let resolved = resolved(
        &[("input.comp.md", ":: box[B]\n")],
        impostor,
        &["@compmd/core"],
    );
    assert_eq!(resolved.documents.len(), 2, "the document and core, once");
    let core = resolved.document(bound(&resolved, "box").document);
    assert!(core.origin.is_core());
    assert_eq!(core.text, CORE_TEXT);
}

#[test]
fn a_component_file_is_bound_with_its_signature_fence() {
    let input = "---\nmd:\n  import:\n    - from: ./charts/Chart.jsx\n      names: [{name: Chart, as: plot}]\n---\n\n```compmd component=plot signature\nparams:\n  rows: array\n```\n";
    let resolved = resolved(
        &[
            ("input.comp.md", input),
            ("charts/Chart.jsx", "export default () => null;"),
        ],
        json!({}),
        &[],
    );
    let plot = bound(&resolved, "plot");
    assert!(
        matches!(&plot.body, Body::File { origin: Origin::File(path), .. } if path == "charts/Chart.jsx")
    );
    assert!(
        plot.signature.is_some() && plot.body.supported(&Policy::default()) && plot.body.is_host()
    );
}

#[test]
fn a_packs_own_file_is_loaded_once() {
    let kit = json!({ "@acme/kit": [{ "version": "2.0.0", "files": {
        "package.json": "{\"name\": \"@acme/kit\", \"version\": \"2.0.0\"}",
        "index.comp.md": "---\nmd:\n  import:\n    - from: ./parts.comp.md\n      names: [part]\n  export:\n    names: [part]\n---\n",
        "parts.comp.md": "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=part\n---\nPart.\n```\n",
    } }] });
    let input = "---\nmd:\n  import:\n    - from: \"@acme/kit@2\"\n      names: [part]\n    - from: \"@acme/kit\"\n      names: \"*\"\n---\n";
    let resolved = resolved(&[("input.comp.md", input)], kit, &[]);
    let parts = resolved
        .documents
        .iter()
        .filter(|document| document.origin.to_string() == "@acme/kit@2.0.0/parts.comp.md");
    assert_eq!(parts.count(), 1);
}

#[test]
fn a_broken_release_is_f035_and_loads_nothing() {
    let kit = json!({ "@acme/kit": [{ "version": "1.0.0", "broken": "its tarball doesn't match `dist.integrity`", "files": {
        "package.json": "{\"name\": \"@acme/kit\", \"version\": \"1.0.0\"}",
        "index.comp.md": "Kit.\n",
    } }] });
    let input = "---\nmd:\n  import:\n    - from: \"@acme/kit@1\"\n      names: \"*\"\n---\n";
    let resolved = resolved(&[("input.comp.md", input)], kit, &[]);
    let [found] = resolved.diagnostics().collect::<Vec<_>>()[..] else {
        panic!("one diagnostic");
    };
    assert_eq!(found.code, crate::diagnostic::Code::F035);
    assert!(
        found
            .message
            .contains("`@acme/kit@1.0.0`: its tarball doesn't match")
    );
    assert_eq!(resolved.documents.len(), 2, "core and the document");
    assert!(resolved.missing.is_empty(), "a release was given");
}

#[test]
fn an_ambient_packs_newest_release_gives_the_processors_components() {
    let release = |version: &str, component: &str| {
        json!({ "version": version, "files": {
            "package.json": format!("{{\"name\": \"@host/c\", \"version\": \"{version}\"}}"),
            "index.comp.md": format!("---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component={component}\n---\nX.\n```\n\n```compmd component=tag signature\nparams: {{}}\n```\n"),
        } })
    };
    let packs = json!({ "@host/c": [release("1.0.0", "old"), release("1.1.0", "new")] });
    let resolved = resolved(&[("input.comp.md", ":: tag[T]\n")], packs, &["@host/c"]);
    assert_eq!(bound(&resolved, "tag").body, Body::Processor);
    assert_eq!(bound(&resolved, "new").name, "new");
    assert!(resolved.document(resolved.main).names.get("old").is_none());
}
