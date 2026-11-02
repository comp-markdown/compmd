//! The host's policy, option by option: which packs a document may import, which it denies or left out, each
//! pack's trust, and the runtimes it runs.

use super::*;
use compmd::host::NeededPack;
use compmd::policy::Trust;
use std::collections::BTreeMap;

fn kit() -> Host {
    Host::default().with(
        "@acme/kit",
        release("@acme/kit", "1.2.0", &card_pack("A card."), json!({})),
    )
}

fn with_policy(host: Host, policy: Value) -> Host {
    Host {
        policy: serde_json::from_value(policy).expect("a policy"),
        ..host
    }
}

#[test]
fn under_imports_listed_a_pack_the_configuration_lists_is_imported() {
    let host = with_policy(
        kit(),
        json!({ "imports": "listed", "packs": { "@acme/kit": { "level": "strict" } } }),
    );
    assert_eq!(
        diagnostics(&host.process(&importing(&["@acme/kit@1"], ""))),
        Vec::<String>::new()
    );
}

#[test]
fn under_imports_listed_any_other_pack_is_f026_and_never_asked_for() {
    let host = with_policy(Host::default(), json!({ "imports": "listed" }));
    let validated = host.validate(&importing(&["@acme/kit@1"], ""));
    assert_eq!(
        diagnostics(&validated),
        [
            "F026 this host allows only the packs its configuration lists, and `@acme/kit` isn't one; the import is dropped"
        ]
    );
    assert_eq!(
        validated.sources(&host.context).packs,
        Vec::<NeededPack>::new()
    );
}

#[test]
fn a_pack_the_host_denies_is_f026_with_its_reason() {
    let host = with_policy(
        kit(),
        json!({ "deny": [{ "pack": "@acme/kit", "reason": "draws fake prompts" }] }),
    );
    assert_eq!(
        diagnostics(&host.process(&importing(&["@acme/kit@1"], "")))[0],
        "F026 `@acme/kit@1.2.0` is denied here: draws fake prompts; the import is dropped"
    );
}

#[test]
fn a_denied_range_leaves_the_other_versions_allowed() {
    let host = with_policy(
        kit(),
        json!({ "deny": [{ "pack": "@acme/kit", "version": "<1.2", "reason": "a flaw found in 1.1" }] }),
    );
    assert_eq!(
        diagnostics(&host.process(&importing(&["@acme/kit@1"], ""))),
        Vec::<String>::new()
    );
}

#[test]
fn a_pack_the_host_left_out_is_f026_with_why_and_never_asked_for() {
    let host = with_policy(
        Host::default(),
        json!({ "refused": { "@acme/kit": "larger than this host serves" } }),
    );
    let validated = host.validate(&importing(&["@acme/kit@1"], ""));
    assert_eq!(
        diagnostics(&validated),
        [
            "F026 `@acme/kit` isn't available here: larger than this host serves; the import is dropped"
        ]
    );
    assert_eq!(
        validated.sources(&host.context).packs,
        Vec::<NeededPack>::new()
    );
}

#[test]
fn each_packs_components_have_its_trust_level_strict_unless_listed() {
    let host = Host::default()
        .with(
            "@acme/charts",
            release("@acme/charts", "1.0.0", &js_pack("chart"), json!({})),
        )
        .with(
            "@acme/maps",
            release("@acme/maps", "1.0.0", &js_pack("map"), json!({})),
        )
        .with(
            "@acme/stars",
            release("@acme/stars", "1.0.0", &js_pack("stars"), json!({})),
        );
    let host = with_policy(
        host,
        json!({ "packs": {
            "@acme/maps": { "level": "permissive", "network": ["https://tiles.example.com"], "host": true },
            "@acme/stars": { "level": "trusted" }
        } }),
    );
    let processed = host.process(&importing(
        &["@acme/charts@1", "@acme/maps@1", "@acme/stars@1"],
        "",
    ));
    let trust: BTreeMap<String, Trust> = processed
        .host_components()
        .into_iter()
        .map(|component| (component.name, component.trust))
        .collect();
    assert_eq!(
        trust,
        BTreeMap::from([
            ("chart".to_string(), Trust::Strict),
            (
                "map".to_string(),
                Trust::Permissive {
                    network: vec!["https://tiles.example.com".into()],
                    host: true
                }
            ),
            ("stars".to_string(), Trust::Trusted),
        ])
    );
}

/// A Svelte component, with its signature, and a use of it.
const BADGE: &str = "```svelte component=badge\n<b>hi</b>\n```\n\n```compmd component=badge signature\nparams: {}\n```\n\n:: badge[Badge]\n";

#[test]
fn a_component_on_a_runtime_the_host_runs_is_given_to_the_renderer() {
    let host = with_policy(
        Host::default(),
        json!({ "runtimes": [{ "name": "svelte", "version": "5.57.2" }] }),
    );
    let processed = host.process(BADGE);
    assert_eq!(codes(&processed), Vec::<String>::new());
    assert_eq!(processed.host_components().len(), 1);
}

#[test]
fn a_component_on_a_runtime_the_host_doesnt_run_is_w102_and_left_out() {
    let host = with_policy(
        Host::default(),
        json!({ "runtimes": [{ "name": "react", "version": "19.3.0" }] }),
    );
    let processed = host.process(BADGE);
    assert_eq!(codes(&processed), ["W102"]);
    assert_eq!(processed.host_components(), []);
    assert_eq!(shown(&processed), "Badge", "its fallback");
}
