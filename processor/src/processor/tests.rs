use super::*;
use crate::host::NeededPack;
use crate::parse::TextEdit;
use serde_json::{Map, json};
use std::collections::HashMap;

fn context(inputs: serde_json::Value) -> RenderContext {
    let inputs: Map<String, serde_json::Value> = serde_json::from_value(inputs).expect("an object");
    RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs,
    }
}

/// A document with an input `n`, a data import that can't load, and a use of `x`, which nothing defines.
const TEXT: &str = "---\nmd:\n  data:\n    - from: ./missing.json\n      as: m\n  inputs:\n    n: number = 1\n---\n\n{{md.input.n}} :x[y]\n";

/// The codes of the diagnostics so far.
fn codes<S>(processor: &Processor<S>) -> Vec<String> {
    processor
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code.to_string())
        .collect()
}

/// The values of a loaded processor's inputs.
fn values(loaded: &Processor<Loaded>) -> Vec<serde_json::Value> {
    loaded
        .inputs()
        .iter()
        .map(|input| input.value.clone())
        .collect()
}

/// The text of the expanded tree's first paragraph.
fn written(processor: &Processor<Expanded>) -> serde_json::Value {
    serde_json::to_value(processor.expanded()).expect("a tree")["children"][0]["children"][0]["value"]
        .clone()
}

fn validated() -> Processor<Validated> {
    let files: HashMap<String, String> = HashMap::new();
    Processor::new("a.comp.md", TEXT)
        .parse(&ParseOptions::default())
        .resolve(files, (), &Ambient::default())
        .validate(&[])
}

#[test]
fn each_state_gives_what_its_stage_made_with_the_diagnostics_so_far() {
    let parsed = Processor::new("a.comp.md", TEXT).parse(&ParseOptions::default());
    assert_eq!(
        parsed.tree().children.len(),
        2,
        "the frontmatter and a paragraph"
    );
    assert_eq!(codes(&parsed), Vec::<String>::new());
    let resolved = parsed.resolve(HashMap::<String, String>::new(), (), &Ambient::default());
    assert_eq!(codes(&resolved), ["F002"], "`x`");
    let validated = resolved.validate(&[]);
    assert_eq!(codes(&validated), ["F002"]);
    let loaded = validated.load(context(json!({})), (), ());
    assert_eq!(values(&loaded), [json!(1)], "`n`'s default");
    assert_eq!(
        codes(&loaded),
        ["F002", "F026"],
        "and the data that can't load"
    );
    let expanded = loaded.expand(&expand::Options::default());
    assert_eq!(written(&expanded), json!("1 y"));
    assert_eq!(expanded.sources(), Needed::default());
}

#[test]
fn a_loaded_processor_loads_again_for_another_render_context_keeping_stages_1_to_3() {
    let loaded = validated().load(context(json!({})), (), ());
    let reloaded = loaded.reload(context(json!({ "n": 2 })), (), ());
    assert_eq!(values(&reloaded), [json!(2)]);
    assert_eq!(
        codes(&reloaded),
        ["F002", "F026"],
        "stage 4's own found again, not twice"
    );
}

#[test]
fn an_expanded_processor_reloads_and_replaces_the_whole_tree() {
    let expanded = validated()
        .load(context(json!({ "n": 2 })), (), ())
        .expand(&expand::Options::default());
    assert_eq!(written(&expanded), json!("2 y"));
    let reloaded = expanded.reload(context(json!({ "n": 3 })), (), ());
    assert_eq!(written(&reloaded), json!("3 y"));
    let splice = reloaded.splice();
    assert_eq!((splice.from, splice.to, splice.nodes.len()), (0, 1, 1));
    assert_eq!(codes(&reloaded), ["F002", "F026"]);
}

#[test]
fn sources_list_the_packs_imported_that_the_host_has_no_release_of() {
    let import =
        |from: &str, extra: &str| format!("    - from: \"{from}\"\n{extra}      names: \"*\"\n");
    let text = [
        "---\nmd:\n  import:\n".to_string(),
        import("@acme/kit@1", ""),
        import("@acme/kit@1", ""),
        import("@acme/charts@^2", ""),
        import("@acme/base", ""),
        import("@acme/loose@^1", "      integrity: sha512-l\n"),
        import("@bad/pack", ""),
        "---\n".to_string(),
    ]
    .concat();
    let base = format!(
        "---\nmd:\n  import:\n{}---\n",
        import("@acme/theme@1.2.3", "      integrity: sha512-t\n")
    );
    let packs: HashMap<String, Vec<crate::host::LoadedRelease>> = serde_json::from_value(json!({
        "@acme/charts": [{ "version": "1.0.0", "files": {} }],
        "@acme/base": [{ "version": "1.0.0", "files": {
            "package.json": "{\"name\": \"@acme/base\", \"version\": \"1.0.0\"}",
            "index.comp.md": base,
        } }],
    }))
    .expect("packs");
    let policy: Policy = serde_json::from_value(json!({ "refused": { "@bad/pack": "too large" } }))
        .expect("a policy");
    let validated = Processor::new("a.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve_with((), packs, &Ambient::default(), &policy)
        .validate(&[]);
    let needed = |pack: &str, range: &str, integrity: Option<&str>| NeededPack {
        pack: pack.into(),
        range: range.into(),
        integrity: integrity.map(Into::into),
        library: false,
    };
    let expected = [
        needed("@acme/kit", "1", None),
        needed("@acme/charts", "^2", None),
        needed("@acme/theme", "1.2.3", Some("sha512-t")),
        needed("@acme/loose", "^1", None),
    ];
    assert_eq!(validated.sources(&context(json!({}))).packs, expected);
    let expanded = validated
        .load(context(json!({})), (), ())
        .expand(&expand::Options::default());
    assert_eq!(expanded.sources().packs, expected);
    let json = serde_json::to_value(expanded.sources()).expect("JSON");
    assert_eq!(
        json["packs"][0],
        json!({ "pack": "@acme/kit", "range": "1" })
    );
    assert_eq!(
        json["packs"][2],
        json!({ "pack": "@acme/theme", "range": "1.2.3", "integrity": "sha512-t" })
    );
}

#[test]
fn an_edit_after_text_of_two_blocks_was_joined_expands_the_whole_document() {
    let files: HashMap<String, String> = HashMap::new();
    let context = context(json!({}));
    let mut processor = Processor::new("a.comp.md", "One.\n\nTwo.\n")
        .parse(&ParseOptions::default())
        .resolve(files, (), &Ambient::default())
        .validate(&[])
        .load(context, (), ())
        .expand(&expand::Options::default());
    // As joining text across blocks leaves it.
    processor.state.result.blocks = None;
    let processor = processor.edit(&TextEdit {
        start: 9,
        end: 9,
        text: "!".into(),
    });
    let splice = processor.splice();
    assert_eq!((splice.from, splice.to, splice.nodes.len()), (0, 2, 2));
    let json = serde_json::to_value(processor.expanded()).expect("a tree");
    assert_eq!(json["children"][1]["children"][0]["value"], "Two!.");
}
