use crate::context::RenderContext;
use crate::diagnostic::{Code, Diagnostic};
use crate::host::{LoadedRelease, NeededPack};
use crate::policy::Policy;
use crate::processor::HostComponent;
use crate::{Ambient, ParseOptions, Processor, expand};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::rc::Rc;

/// A release of the pack `name` with one JS component, `component`, naming `peers`.
fn pack(name: &str, version: &str, component: &str, peers: Value) -> Value {
    let mut manifest = json!({ "name": name, "version": version });
    manifest["peerDependencies"] = peers;
    let index = format!(
        "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```js component={component}\nexport default () => {{}};\n```\n\n```compmd component={component} signature\nparams: {{}}\n```\n"
    );
    json!({ "version": version, "files": { "package.json": manifest.to_string(), "index.comp.md": index } })
}

/// A release of the library pack `name`, its `compmd` field `compmd`, naming `peers`.
fn library(name: &str, version: &str, compmd: Value, peers: Value) -> Value {
    let mut manifest = json!({ "name": name, "version": version });
    manifest["compmd"] = compmd;
    manifest["peerDependencies"] = peers;
    json!({ "version": version, "files": { "package.json": manifest.to_string(), "dist/index.js": "export {};" } })
}

struct Processed {
    components: Vec<HostComponent>,
    diagnostics: Vec<Diagnostic>,
    needed: Vec<NeededPack>,
}

/// A document importing every pack of `imports` (`@acme/charts@1`), with `packs` and `policy`.
fn processed(imports: &[&str], packs: Value, policy: Value) -> Processed {
    let imports: String = imports
        .iter()
        .flat_map(|from| ["    - from: \"", from, "\"\n      names: \"*\"\n"])
        .collect();
    let text = format!("---\nmd:\n  import:\n{imports}---\n");
    let packs: HashMap<String, Vec<LoadedRelease>> = serde_json::from_value(packs).expect("packs");
    let policy: Policy = serde_json::from_value(policy).expect("a policy");
    let context = RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    };
    let validated = Processor::new("a.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve_with((), packs, &Ambient::default(), &policy)
        .validate(&[]);
    let needed = validated.sources(&context).packs;
    let expanded = validated
        .load(context, (), ())
        .expand(&expand::Options::default());
    Processed {
        components: expanded.host_components(),
        diagnostics: expanded.diagnostics().to_vec(),
        needed,
    }
}

/// Each component's libraries as `name: pack@version`, nested ones after `>`.
fn chosen(processed: &Processed) -> Value {
    fn named(
        libraries: &std::collections::BTreeMap<String, crate::processor::LibraryModule>,
    ) -> Vec<String> {
        libraries
            .iter()
            .flat_map(|(name, module)| {
                let own = format!("{name}: {}", module.place);
                std::iter::once(own.clone()).chain(
                    named(&module.libraries)
                        .into_iter()
                        .map(move |inner| format!("{own} > {inner}")),
                )
            })
            .collect()
    }
    processed
        .components
        .iter()
        .map(|component| (component.name.clone(), json!(named(&component.libraries))))
        .collect::<serde_json::Map<_, _>>()
        .into()
}

fn messages(processed: &Processed) -> Vec<String> {
    processed
        .diagnostics
        .iter()
        .inspect(|diagnostic| assert_eq!(diagnostic.code, Code::W121, "{}", diagnostic.message))
        .map(|diagnostic| diagnostic.message.clone())
        .collect()
}

#[test]
fn packs_share_the_newest_version_every_range_allows() {
    let packs = json!({
        "@acme/charts": [pack("@acme/charts", "1.0.0", "chart", json!({ "@kit/d3": "^7.9", "react": "^19" }))],
        "@acme/maps": [pack("@acme/maps", "1.0.0", "map", json!({ "@kit/d3": "~7.9.0" }))],
        "@kit/d3": [
            library("@kit/d3", "7.9.5", json!({ "library": "./dist/index.js" }), json!({})),
            library("@kit/d3", "7.12.0", json!({ "library": "dist/index.js" }), json!({})),
        ],
    });
    let processed = processed(&["@acme/charts@1", "@acme/maps@1"], packs, json!({}));
    assert_eq!(messages(&processed), Vec::<String>::new());
    assert_eq!(
        chosen(&processed),
        json!({ "chart": ["@kit/d3: @kit/d3@7.9.5/dist/index.js"], "map": ["@kit/d3: @kit/d3@7.9.5/dist/index.js"] })
    );
    assert_eq!(processed.needed, Vec::<NeededPack>::new());
}

#[test]
fn ranges_that_share_no_version_get_one_each_unless_the_library_is_single() {
    let packs = |single: bool| {
        json!({
            "@acme/charts": [pack("@acme/charts", "1.0.0", "chart", json!({ "@kit/three": "^0.170" }))],
            "@acme/maps": [pack("@acme/maps", "1.0.0", "map", json!({ "@kit/three": "^0.171" }))],
            "@kit/three": [
                library("@kit/three", "0.170.2", json!({ "library": "dist/index.js", "single": single }), json!({})),
                library("@kit/three", "0.171.0", json!({ "library": "dist/index.js", "single": single }), json!({})),
            ],
        })
    };
    let two = processed(&["@acme/charts@1", "@acme/maps@1"], packs(false), json!({}));
    assert_eq!(messages(&two), Vec::<String>::new());
    assert_eq!(
        chosen(&two),
        json!({ "chart": ["@kit/three: @kit/three@0.170.2/dist/index.js"], "map": ["@kit/three: @kit/three@0.171.0/dist/index.js"] })
    );
    let one = processed(&["@acme/charts@1", "@acme/maps@1"], packs(true), json!({}));
    assert_eq!(
        messages(&one),
        [
            "`@acme/maps@1.0.0` needs `@kit/three` `^0.171`, which can't have two copies in one sandbox, and this document has 0.170.2, for `^0.170`, so its components that use it render their fallbacks"
        ]
    );
    assert_eq!(
        chosen(&one),
        json!({ "chart": ["@kit/three: @kit/three@0.170.2/dist/index.js"], "map": [] })
    );
}

#[test]
fn a_single_librarys_version_serves_every_range_it_matches() {
    let packs = json!({
        "@acme/charts": [pack("@acme/charts", "1.0.0", "chart", json!({ "@kit/three": "^0.170" }))],
        "@acme/maps": [pack("@acme/maps", "1.0.0", "map", json!({ "@kit/three": "^0.171" }))],
        "@acme/globe": [pack("@acme/globe", "1.0.0", "globe", json!({ "@kit/three": ">=0.170" }))],
        "@kit/three": [
            library("@kit/three", "0.170.2", json!({ "library": "dist/index.js", "single": true }), json!({})),
            library("@kit/three", "0.171.0", json!({ "library": "dist/index.js", "single": true }), json!({})),
        ],
    });
    let processed = processed(
        &["@acme/charts@1", "@acme/maps@1", "@acme/globe@1"],
        packs,
        json!({}),
    );
    assert_eq!(messages(&processed).len(), 1, "`@acme/maps`");
    assert_eq!(
        chosen(&processed)["globe"],
        json!(["@kit/three: @kit/three@0.170.2/dist/index.js"])
    );
}

#[test]
fn a_librarys_own_libraries_are_chosen_with_the_document() {
    let packs = json!({
        "@acme/charts": [pack("@acme/charts", "1.0.0", "chart", json!({ "@kit/three": "^0.170", "@kit/math": "^1.2" }))],
        "@kit/three": [library("@kit/three", "0.170.2", json!({ "library": "dist/index.js" }), json!({ "@kit/math": "^1", "react": "^19" }))],
        "@kit/math": [
            library("@kit/math", "1.1.0", json!({ "library": "dist/index.js" }), json!({})),
            library("@kit/math", "1.3.0", json!({ "library": "dist/index.js" }), json!({})),
        ],
    });
    let processed = processed(
        &["@acme/charts@1"],
        packs,
        json!({ "runtimes": [{ "name": "react", "version": "18.3.0" }] }),
    );
    assert_eq!(
        messages(&processed),
        [
            "`@kit/three@0.170.2` needs `react` `^19`, and this host has 18.3.0, so its components that use it render their fallbacks"
        ]
    );
    assert_eq!(
        chosen(&processed)["chart"],
        json!([
            "@kit/math: @kit/math@1.3.0/dist/index.js",
            "@kit/three: @kit/three@0.170.2/dist/index.js",
            "@kit/three: @kit/three@0.170.2/dist/index.js > @kit/math: @kit/math@1.3.0/dist/index.js",
        ])
    );
}

#[test]
fn libraries_that_name_each_other_are_w121_and_the_cycle_is_cut() {
    let packs = json!({
        "@acme/charts": [pack("@acme/charts", "1.0.0", "chart", json!({ "@kit/a": "^1" }))],
        "@kit/a": [library("@kit/a", "1.0.0", json!({ "library": "dist/index.js" }), json!({ "@kit/b": "^1" }))],
        "@kit/b": [library("@kit/b", "1.0.0", json!({ "library": "dist/index.js" }), json!({ "@kit/a": "^1" }))],
    });
    let processed = processed(&["@acme/charts@1"], packs, json!({}));
    assert_eq!(
        messages(&processed),
        [
            "`@kit/b@1.0.0` needs `@kit/a` `^1`, but libraries name each other: `@kit/a` → `@kit/b` → `@kit/a`, so its components that use it render their fallbacks"
        ]
    );
    assert_eq!(
        chosen(&processed)["chart"],
        json!([
            "@kit/a: @kit/a@1.0.0/dist/index.js",
            "@kit/a: @kit/a@1.0.0/dist/index.js > @kit/b: @kit/b@1.0.0/dist/index.js"
        ])
    );
}

#[test]
fn a_library_the_host_hasnt_is_listed_for_it_to_fetch() {
    let packs = json!({
        "@acme/charts": [pack("@acme/charts", "1.0.0", "chart", json!({ "@kit/three": "^0.170", "@kit/plot": "1.0.0", "d3": "^7", "@kit/odd": "not a range" }))],
        "@kit/three": [library("@kit/three", "0.169.0", json!({ "library": "dist/index.js" }), json!({}))],
    });
    let processed = processed(&["@acme/charts@1"], packs, json!({}));
    assert_eq!(
        messages(&processed),
        [
            "`@acme/charts@1.0.0` needs `@kit/three` `^0.170`, which this host couldn't get, so its components that use it render their fallbacks",
            "`@acme/charts@1.0.0` needs `@kit/plot` `1.0.0`, which this host couldn't get, so its components that use it render their fallbacks",
        ]
    );
    let needed = |pack: &str, range: &str| NeededPack {
        pack: pack.parse().expect("a pack's name"),
        range: range.into(),
        integrity: None,
        library: true,
    };
    assert_eq!(
        processed.needed,
        [needed("@kit/three", "^0.170"), needed("@kit/plot", "1.0.0")]
    );
    assert_eq!(
        serde_json::to_value(&processed.needed[0]).expect("JSON"),
        json!({ "pack": "@kit/three", "range": "^0.170", "library": true })
    );
}

#[test]
fn a_library_that_cant_be_used_says_why() {
    let packs = json!({
        "@acme/charts": [pack("@acme/charts", "1.0.0", "chart", json!({
            "@compmd/three": "^1", "@kit/refused": "^1", "@kit/denied": "^1", "@kit/broken": "^1", "@kit/plain": "^1", "@kit/empty": "^1"
        }))],
        "@kit/refused": [library("@kit/refused", "1.0.0", json!({ "library": "dist/index.js" }), json!({}))],
        "@kit/denied": [library("@kit/denied", "1.0.0", json!({ "library": "dist/index.js" }), json!({}))],
        "@kit/broken": [{ "version": "1.0.0", "broken": "its tarball doesn't match", "files": {} }],
        "@kit/plain": [{ "version": "1.0.0", "files": { "package.json": "{\"name\": \"@kit/plain\"}" } }],
        "@kit/empty": [{ "version": "1.0.0", "files": { "package.json": "{\"compmd\": {\"library\": \"dist/index.js\"}}" } }],
    });
    let policy = json!({ "refused": { "@kit/refused": "larger than this host serves" }, "deny": [{ "pack": "@kit/denied", "reason": "misuse" }] });
    let processed = processed(&["@acme/charts@1"], packs, policy);
    let why: Vec<String> = messages(&processed)
        .iter()
        .map(|message| {
            message
                .trim_start_matches("`@acme/charts@1.0.0` needs ")
                .trim_end_matches(", so its components that use it render their fallbacks")
                .to_string()
        })
        .collect();
    assert_eq!(
        why,
        [
            "`@compmd/three` `^1`, which is reserved, so no host fetches it",
            "`@kit/refused` `^1`, but `@kit/refused` isn't available here: larger than this host serves",
            "`@kit/denied` `^1`, but `@kit/denied@1.0.0` is denied here: misuse",
            "`@kit/broken` `^1`, but `@kit/broken@1.0.0`: its tarball doesn't match",
            "`@kit/plain` `^1`, but `@kit/plain@1.0.0` isn't a library pack",
            "`@kit/empty` `^1`, but `@kit/empty@1.0.0` doesn't have its module `dist/index.js`",
        ]
    );
    assert_eq!(processed.needed, Vec::<NeededPack>::new());
}
