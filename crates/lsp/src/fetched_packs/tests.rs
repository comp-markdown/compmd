use compmd_host::pack::Place;

use super::*;

fn pack(name: &str) -> PackName {
    name.parse().expect("a name")
}

fn import(name: &str, range: &str) -> NeededPack {
    NeededPack {
        pack: pack(name),
        range: range.into(),
        integrity: None,
        library: false,
    }
}

/// What a fetch of `@kit/notes` gave: its 1.0.0 release, `@kit/huge` refused, `@kit/gone` not got.
fn resolved() -> Resolved {
    let release = compmd_host::registry::Release {
        version: "1.0.0".parse().expect("a version"),
        integrity: Some("sha512-abc".into()),
        files: BTreeMap::from([("index.comp.md".to_owned(), "# Notes".to_owned())]),
        broken: None,
    };
    Resolved {
        releases: BTreeMap::from([(pack("@kit/notes"), vec![release])]),
        refused: BTreeMap::from([(pack("@kit/huge"), "too large".to_owned())]),
        problems: vec![Problem {
            pack: pack("@kit/gone"),
            message: "no answer".into(),
        }],
        ..Resolved::default()
    }
}

#[test]
fn an_import_is_asked_once_and_pending_until_its_answer() {
    let mut fetched = FetchedPacks::default();
    let asked = fetched.ask(vec![import("@kit/notes", "1"), import("@kit/notes", "1")]);
    assert_eq!(asked.len(), 1, "the same import, once");
    assert_eq!(
        (asked[0].pack.as_str(), asked[0].range.as_str()),
        ("@kit/notes", "1")
    );
    assert!(fetched.source().pending(&pack("@kit/notes")));
    assert_eq!(fetched.ask(vec![import("@kit/notes", "1")]), []);
    let problems = fetched.add(&[pack("@kit/notes")], resolved());
    assert_eq!(problems.len(), 1);
    let source = fetched.source();
    assert!(!source.pending(&pack("@kit/notes")));
    assert_eq!(
        source.releases(&pack("@kit/notes"))[0].version.to_string(),
        "1.0.0"
    );
    let place: Place = "@kit/notes@1.0.0/index.comp.md".parse().expect("a place");
    assert_eq!(source.pack_file(&place).as_deref(), Some("# Notes"));
    assert_eq!(fetched.refused()[&pack("@kit/huge")], "too large");
}

#[test]
fn a_pack_asked_twice_is_pending_until_both_answers_and_held_once() {
    let mut fetched = FetchedPacks::default();
    fetched.ask(vec![import("@kit/notes", "1")]);
    fetched.ask(vec![import("@kit/notes", "^1.0.0")]);
    fetched.add(&[pack("@kit/notes")], resolved());
    assert!(
        fetched.source().pending(&pack("@kit/notes")),
        "one answer of two"
    );
    fetched.add(&[pack("@kit/notes")], resolved());
    assert!(!fetched.source().pending(&pack("@kit/notes")));
    assert_eq!(fetched.source().releases(&pack("@kit/notes")).len(), 1);
    fetched.add(&[pack("@kit/notes")], Resolved::default());
    assert!(
        !fetched.source().pending(&pack("@kit/notes")),
        "an answer no ask waits for"
    );
}

#[test]
fn after_the_project_is_read_again_an_import_is_asked_again() {
    let mut fetched = FetchedPacks::default();
    fetched.ask(vec![import("@kit/notes", "1")]);
    fetched.ask_again();
    assert_eq!(fetched.ask(vec![import("@kit/notes", "1")]).len(), 1);
}
