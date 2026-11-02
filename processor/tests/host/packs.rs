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

#[test]
fn a_missing_pack_is_listed_for_the_host_to_get_and_the_document_processed_again_with_it() {
    let text = importing(&["@acme/kit@^1.1"], ":: card[Card]\n");
    let host = Host::default();
    let first = host.validate(&text);
    let needed = first.sources(&host.context).packs;
    assert_eq!(
        needed,
        [NeededPack {
            pack: "@acme/kit".into(),
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
    let text = "---\nmd:\n  import:\n    - from: \"@acme/kit@1.2.0\"\n      integrity: sha512-@acme/kit@1.2.0\n      names: [card]\n---\n\n:: card[Card]\n";
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
            pack: "@kit/d3".into(),
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
        pack: "@kit/d3".into(),
        version: Version::new(7, 9, 0),
        path: "dist/d3.js".into(),
        libraries: BTreeMap::new(),
    };
    assert_eq!(
        bars.libraries,
        BTreeMap::from([("@kit/d3".to_string(), module)])
    );
}
