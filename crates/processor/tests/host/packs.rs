//! Packs: importing one the host gives, the host's round trip for one it hasn't, integrity, broken releases, and
//! library packs.

use super::*;
use compmd::host::{NeededPack, newest_matching};
use compmd::processor::LibraryModule;
use std::collections::BTreeMap;

#[test]
fn a_document_imports_a_pack_the_host_gives_and_uses_its_components() {
    let host = Host::default().with(
        "@acme/kit",
        release("@acme/kit", "1.2.0", &card_pack("A card."), json!({})),
    );
    let processed = host.process(&importing(&["@acme/kit@1"], ":: card[Card]\n"));
    assert_eq!(diagnostics(&processed), Vec::<String>::new());
    assert_eq!(shown(&processed), "A card.");
}

/// A release of `@acme/kit` at `version` holding `icons.json`, whose `name` is `name`.
fn kit(version: &str, name: &str) -> LoadedRelease {
    let mut kit = release(
        "@acme/kit",
        version,
        "---\nmd:\n  export:\n    names: \"*\"\n---\n",
        json!({}),
    );
    kit.files.insert(
        "icons.json".to_string(),
        json!({ "name": name }).to_string(),
    );
    kit
}

#[test]
fn a_document_reads_a_packs_file_only_through_its_own_import_of_the_pack() {
    // `@acme/board` imports the kit; the document imports the board, not the kit.
    let board = "---\nmd:\n  import:\n    - from: \"@acme/kit@1\"\n      names: \"*\"\n  export:\n    names: \"*\"\n---\n\n```compmd component=board\n---\nA board.\n```\n";
    let host = Host::default().with("@acme/kit", kit("1.0.0", "v1")).with(
        "@acme/board",
        release("@acme/board", "1.0.0", board, json!({})),
    );
    let text = "---\nmd:\n  import:\n    - from: \"@acme/board@1\"\n      names: \"*\"\n  data:\n    - from: \"@acme/kit/icons.json\"\n      as: icons\n---\n\n:: board[B]\n";
    let processed = host.process(text);
    assert!(
        diagnostics(&processed)
            .iter()
            .any(|diagnostic| diagnostic.contains("`@acme/kit` isn't imported")),
        "{:?}",
        diagnostics(&processed)
    );
}

#[test]
fn each_document_reads_a_packs_file_in_the_release_its_own_import_took() {
    // The document imports kit 2; `@acme/old`, which it imports too, imports kit 1, and shows its own icons' name.
    let old = "---\nmd:\n  import:\n    - from: \"@acme/kit@1\"\n      names: \"*\"\n  data:\n    - from: \"@acme/kit/icons.json\"\n      as: icons\n  export:\n    names: \"*\"\n---\n\n```compmd component=old\n---\nold {{md.data.icons.name}}\n```\n";
    let host = Host::default()
        .with("@acme/kit", kit("1.0.0", "v1"))
        .with("@acme/kit", kit("2.0.0", "v2"))
        .with("@acme/old", release("@acme/old", "1.0.0", old, json!({})));
    let text = "---\nmd:\n  import:\n    - from: \"@acme/old@1\"\n      names: \"*\"\n    - from: \"@acme/kit@2\"\n      names: \"*\"\n  data:\n    - from: \"@acme/kit/icons.json\"\n      as: icons\n---\n\nmain {{md.data.icons.name}}\n\n:: old[O]\n";
    let processed = host.process(text);
    let shown = shown(&processed);
    assert!(shown.contains("main v2"), "{shown}");
    assert!(shown.contains("old v1"), "{shown}");
}

#[test]
fn an_ambient_packs_components_are_given_only_to_a_document_that_uses_them() {
    // Two components in one pack every document gets without importing it (spec §6.3).
    let two = format!(
        "{}\n```js component=map\nexport default () => {{}};\n```\n\n```compmd component=map signature\nparams: {{}}\n```\n",
        js_pack("chart")
    );
    let host = Host {
        ambient: Ambient {
            packs: vec!["@site/kit".parse().expect("a name")],
            report: false,
        },
        ..Host::default().with("@site/kit", release("@site/kit", "1.0.0", &two, json!({})))
    };
    let names = |text: &str| -> Vec<String> {
        host.process(text)
            .host_components()
            .into_iter()
            .map(|component| component.name)
            .collect()
    };
    assert_eq!(names(":: chart[A chart]\n"), ["chart"]);
    assert_eq!(names("Plain text, no component.\n"), Vec::<String>::new());
    assert_eq!(
        names("````compmd {render=output}\n:: map[A map]\n````\n"),
        ["map"]
    );
}

#[test]
fn a_missing_pack_is_listed_for_the_host_to_get_and_the_document_processed_again_with_it() {
    let text = importing(&["@acme/kit@^1.1"], ":: card[Card]\n");
    let host = Host::default();
    let first = host.validate(&text);
    let needed = first.sources(&host.context).packs;
    assert_eq!(
        needed,
        [NeededPack {
            pack: "@acme/kit".parse().expect("a name"),
            range: "^1.1".into(),
            integrity: None,
            library: false
        }]
    );
    // The host's registry has these versions; it takes the one the processor would.
    let versions = ["1.0.0", "1.1.4", "2.0.0"].map(|v| Version::parse(v).expect("a version"));
    let chosen = newest_matching(&needed[0].range, &versions).expect("a version");
    assert_eq!(chosen.to_string(), "1.1.4");
    let host = host.with(
        "@acme/kit",
        release("@acme/kit", "1.1.4", &card_pack("A card."), json!({})),
    );
    let again = host.process(&text);
    assert_eq!(again.sources().packs, Vec::<NeededPack>::new());
    assert_eq!(shown(&again), "A card.");
}

#[test]
fn a_pack_on_its_way_renders_its_uses_pending_then_draws_them_when_it_arrives() {
    let text = importing(&["@acme/kit@1"], ":: card[Card]\n");
    let mut host = Host::default();
    host.pending.insert("@acme/kit".into());
    let waiting = host.process(&text);
    assert_eq!(diagnostics(&waiting), Vec::<String>::new());
    assert_eq!(
        waiting.pending().packs,
        ["@acme/kit"
            .parse::<compmd::pack::PackName>()
            .expect("a name")]
    );
    // Still listed, for the host that is getting it.
    assert_eq!(waiting.sources().packs.len(), 1);
    let tree = serde_json::to_value(waiting.expanded()).expect("a tree");
    assert_eq!(tree["children"][0]["type"], "componentPending");
    assert_eq!(shown(&waiting), "Card");
    // It arrived: the release the host now holds is used, whatever it said before.
    let arrived = host.with(
        "@acme/kit",
        release("@acme/kit", "1.2.0", &card_pack("A card."), json!({})),
    );
    let drawn = arrived.process(&text);
    assert!(drawn.pending().packs.is_empty(), "nothing pending");
    assert_eq!(shown(&drawn), "A card.");
}

#[test]
fn a_pack_the_host_gave_up_on_is_f026_as_any_pack_that_cant_be_had() {
    let text = importing(&["@acme/kit@1"], ":: card[Card]\n");
    let host = Host::default();
    assert_eq!(codes(&host.process(&text)), ["F026"]);
}

#[test]
fn a_release_the_range_doesnt_take_leaves_the_pack_needed_and_the_import_f026() {
    let host = Host::default().with(
        "@acme/kit",
        release("@acme/kit", "2.0.0", &card_pack("A card."), json!({})),
    );
    let processed = host.process(&importing(&["@acme/kit@1"], ":: card[Card]\n"));
    // A wildcard import that loads nothing may hold any name, so `card` isn't F002 as well.
    assert_eq!(codes(&processed), ["F026"]);
    assert_eq!(processed.sources().packs.len(), 1);
}

#[test]
fn an_import_pinned_to_the_releases_integrity_loads() {
    let host = Host::default().with(
        "@acme/kit",
        release("@acme/kit", "1.2.0", &card_pack("A card."), json!({})),
    );
    let text = "---\nmd:\n  import:\n    - from: \"@acme/kit@1.2.0\"\n      integrity: sha512-acme/kit120\n      names: [card]\n---\n\n:: card[Card]\n";
    assert_eq!(diagnostics(&host.process(text)), Vec::<String>::new());
}

#[test]
fn an_import_pinned_to_another_integrity_is_f035_and_loads_nothing() {
    let host = Host::default().with(
        "@acme/kit",
        release("@acme/kit", "1.2.0", &card_pack("A card."), json!({})),
    );
    let text = "---\nmd:\n  import:\n    - from: \"@acme/kit@1.2.0\"\n      integrity: sha512-other\n      names: [card]\n---\n\n:: card[Card]\n";
    assert_eq!(
        diagnostics(&host.process(text))[0],
        "F035 `@acme/kit@1.2.0` doesn't match this `integrity`; the import is dropped"
    );
}

#[test]
fn an_import_pinned_to_an_integrity_a_release_has_none_of_is_f035_and_loads_nothing() {
    // As when a host can't find the release's integrity in its lock file: nothing to match is no match.
    let mut kit = release("@acme/kit", "1.2.0", &card_pack("A card."), json!({}));
    kit.release.integrity = None;
    let host = Host::default().with("@acme/kit", kit);
    let text = "---\nmd:\n  import:\n    - from: \"@acme/kit@1.2.0\"\n      integrity: sha512-acme/kit120\n      names: [card]\n---\n\n:: card[Card]\n";
    assert_eq!(
        diagnostics(&host.process(text))[0],
        "F035 `@acme/kit@1.2.0` doesn't match this `integrity`; the import is dropped"
    );
}

#[test]
fn a_release_the_host_found_broken_is_f035_with_its_reason() {
    let mut broken = release("@acme/kit", "1.2.0", &card_pack("A card."), json!({}));
    broken.release.broken = Some("its tarball doesn't match `dist.integrity`".into());
    let processed = Host::default()
        .with("@acme/kit", broken)
        .process(&importing(&["@acme/kit@1"], ""));
    assert_eq!(codes(&processed), ["F035"]);
    assert!(diagnostics(&processed)[0].contains("its tarball doesn't match `dist.integrity`"));
}

#[test]
fn a_library_pack_a_pack_names_is_listed_then_given_to_its_components() {
    let bars = release(
        "@acme/bars",
        "1.0.0",
        &js_pack("bars"),
        json!({ "peerDependencies": { "@kit/d3": "^7" } }),
    );
    let host = Host::default().with("@acme/bars", bars);
    let text = importing(&["@acme/bars@1"], ":: bars[Bars]\n");
    let first = host.validate(&text);
    assert_eq!(
        first.sources(&host.context).packs,
        [NeededPack {
            pack: "@kit/d3".parse().expect("a name"),
            range: "^7".into(),
            integrity: None,
            library: true
        }]
    );
    assert_eq!(codes(&first), ["W121"], "until the host gets it");
    let mut d3 = release(
        "@kit/d3",
        "7.9.0",
        "",
        json!({ "compmd": { "library": "dist/d3.js" } }),
    );
    d3.files.insert(
        "dist/d3.js".into(),
        "export const version = '7.9.0';".into(),
    );
    let again = host.with("@kit/d3", d3).process(&text);
    assert_eq!(diagnostics(&again), Vec::<String>::new());
    let [bars] = again.host_components().try_into().expect("one component");
    let module = LibraryModule {
        place: "@kit/d3@7.9.0/dist/d3.js".parse().expect("a place"),
        libraries: BTreeMap::new(),
    };
    assert_eq!(
        bars.libraries,
        BTreeMap::from([("@kit/d3".to_string(), module)])
    );
}
