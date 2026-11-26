use std::collections::BTreeMap;

use compmd_host::registry::Release;

use super::*;

fn pack(name: &str) -> PackName {
    name.parse().expect("a name")
}

/// What an ask gave: a release of each of `packs`, at 1.0.0.
fn resolved(packs: &[&str]) -> Resolved {
    let release = || Release {
        version: "1.0.0".parse().expect("a version"),
        integrity: None,
        files: BTreeMap::from([("package.json".to_owned(), "{}".to_owned())]),
        broken: None,
    };
    Resolved {
        releases: packs
            .iter()
            .map(|name| (pack(name), vec![release()]))
            .collect(),
        ..Resolved::default()
    }
}

fn known(suggested: &SuggestedPacks) -> Vec<String> {
    suggested
        .source()
        .known()
        .iter()
        .map(|known| known.release.to_string())
        .collect()
}

#[test]
fn the_scopes_are_compmds_then_the_projects_then_the_writers_each_once() {
    let project = ["@acme".to_owned(), "@comp-markdown".to_owned()];
    let writer = ["@zine".to_owned(), "@acme".to_owned()];
    assert_eq!(
        scopes_of(&project, &writer),
        ["@comp-markdown", "@acme", "@zine"]
    );
    assert_eq!(scopes_of(&[], &[]), ["@comp-markdown"]);
}

#[test]
fn scopes_are_asked_once_until_they_change_or_the_project_is_read_again() {
    let mut suggested = SuggestedPacks::default();
    let scopes = scopes_of(&[], &[]);
    assert!(suggested.ask(&scopes));
    assert!(!suggested.ask(&scopes));
    assert!(
        suggested.ask(&scopes_of(&["@acme".to_owned()], &[])),
        "another scope"
    );
    suggested.ask_again();
    assert!(suggested.ask(&scopes));
}

#[test]
fn what_an_ask_got_is_known_in_its_scopes_order_and_replaces_what_was_held() {
    let mut suggested = SuggestedPacks::default();
    assert_eq!(known(&suggested), Vec::<String>::new());
    suggested.ask(&scopes_of(&["@acme".to_owned()], &[]));
    suggested.add(resolved(&[
        "@acme/maps",
        "@comp-markdown/timeline",
        "@acme/charts",
    ]));
    assert_eq!(
        known(&suggested),
        [
            "@comp-markdown/timeline@1.0.0",
            "@acme/charts@1.0.0",
            "@acme/maps@1.0.0"
        ]
    );
    let source = suggested.source();
    assert_eq!(source.releases(&pack("@acme/maps")).len(), 1);
    let place = "@acme/maps@1.0.0/package.json".parse().expect("a place");
    assert_eq!(source.pack_file(&place).as_deref(), Some("{}"));
    suggested.add(resolved(&["@comp-markdown/badge"]));
    assert_eq!(known(&suggested), ["@comp-markdown/badge@1.0.0"]);
}
