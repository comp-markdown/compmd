use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use semver::Version;

use super::*;
use crate::registry::{LocalRegistry, ServedPack, VersionInfo};

/// A folder of its own under the system's temporary folder, removed when dropped.
struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        Self(std::env::temp_dir().join(format!("compmd-host-resolve-{}-{n}", std::process::id())))
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        drop(fs::remove_dir_all(&self.0));
        drop(fs::remove_file(&self.0));
    }
}

const DOCUMENT: &str = "```compmd component=note\n{{md.slot}}\n```\n";

fn notes() -> ServedPack {
    ServedPack::new("@kit/notes")
        .version(
            "1.0.0",
            &[
                ("package.json", r#"{"name": "@kit/notes"}"#),
                ("index.comp.md", DOCUMENT),
            ],
        )
        .version("2.0.0", &[("package.json", "{}")])
}

fn pack(name: &str) -> PackName {
    name.parse().expect("a name")
}

fn options(registry: &LocalRegistry) -> Options {
    Options::new(BTreeMap::from([(
        "@kit".to_owned(),
        registry.url().to_owned(),
    )]))
}

fn client() -> Client {
    Client::new(Duration::from_secs(5))
}

/// The host's answer to what is missing: `packs` first, then nothing, as a request that imports only them.
fn only(
    packs: &[(&str, &str)],
) -> impl FnMut(Vec<(PackName, Release)>, BTreeMap<PackName, String>) -> Vec<Needed> {
    let mut first: Vec<Needed> = packs
        .iter()
        .map(|(name, range)| Needed {
            pack: pack(name),
            range: (*range).to_owned(),
            integrity: None,
            library: false,
        })
        .collect();
    move |_, _| std::mem::take(&mut first)
}

#[test]
fn a_pack_is_fetched_checked_and_kept() {
    let registry = LocalRegistry::serve(&[notes()]).expect("served");
    let temp = Temp::new();
    let cache = FolderCache::new(&temp.0);
    let resolved = resolve(
        options(&registry),
        Some(&client()),
        &cache,
        only(&[("@kit/notes", "1")]),
    );
    assert_eq!(resolved.problems, []);
    let [release] = resolved.releases[&pack("@kit/notes")].as_slice() else {
        panic!("one release");
    };
    assert_eq!(release.version, Version::new(1, 0, 0));
    assert_eq!(release.files["index.comp.md"], DOCUMENT);
    assert_eq!(
        registry.asked(),
        ["/@kit%2fnotes", "/@kit/notes/-/notes-1.0.0.tgz"]
    );
    assert!(
        cache
            .kept_versions(registry.url(), &pack("@kit/notes"))
            .is_some_and(|kept| kept.fresh)
    );
}

#[test]
fn what_was_kept_needs_no_network() {
    let temp = Temp::new();
    let cache = FolderCache::new(&temp.0);
    let options = {
        let registry = LocalRegistry::serve(&[notes()]).expect("served");
        resolve(
            options(&registry),
            Some(&client()),
            &cache,
            only(&[("@kit/notes", "1")]),
        );
        options(&registry)
    };
    let resolved = resolve(options.clone(), None, &cache, only(&[("@kit/notes", "1")]));
    assert_eq!(resolved.problems, []);
    assert_eq!(
        resolved.releases[&pack("@kit/notes")][0].version,
        Version::new(1, 0, 0)
    );
    let unkept = resolve(options, None, &cache, only(&[("@kit/notes", "2")]));
    assert_eq!(
        unkept.problems,
        [Problem {
            pack: pack("@kit/notes"),
            message: OFFLINE.to_owned()
        }],
        "its versions are kept, but not 2.0.0's files"
    );
}

#[test]
fn a_registry_that_cant_be_reached_or_lacks_the_pack_gives_nothing() {
    let temp = Temp::new();
    let cache = FolderCache::new(&temp.0);
    let gone = {
        let registry = LocalRegistry::serve(&[]).expect("served");
        options(&registry)
    };
    let unreached = resolve(gone, Some(&client()), &cache, only(&[("@kit/notes", "1")]));
    assert!(unreached.releases.is_empty());
    assert_eq!(unreached.problems.len(), 1);
    let registry = LocalRegistry::serve(&[]).expect("served");
    let unknown = resolve(
        options(&registry),
        Some(&client()),
        &cache,
        only(&[("@kit/notes", "1")]),
    );
    assert!(
        unknown.releases.is_empty() && unknown.problems.is_empty(),
        "F026, for the processor"
    );
}

#[test]
fn a_tarball_comes_only_from_its_registrys_origin_and_must_be_there() {
    let registry = LocalRegistry::serve(&[]).expect("served");
    let version = |tarball: String| VersionInfo {
        version: Version::new(1, 0, 0),
        tarball,
        integrity: None,
        unpacked_size: None,
    };
    let elsewhere = client().tarball(
        registry.url(),
        &version("https://elsewhere.example/x.tgz".into()),
        100,
    );
    assert!(
        matches!(elsewhere, Answer::Failed { message } if message.contains("isn't on its registry's origin"))
    );
    let missing = client().tarball(
        registry.url(),
        &version(format!("{}x.tgz", registry.url())),
        100,
    );
    assert!(matches!(missing, Answer::Failed { message } if message.ends_with("answered 404")));
}

#[test]
fn a_cache_that_cant_keep_is_a_problem_and_the_release_is_used() {
    let registry = LocalRegistry::serve(&[notes()]).expect("served");
    let temp = Temp::new();
    fs::write(&temp.0, "a file, not a folder").expect("written");
    let cache = FolderCache::new(&temp.0);
    let resolved = resolve(
        options(&registry),
        Some(&client()),
        &cache,
        only(&[("@kit/notes", "1")]),
    );
    assert_eq!(
        resolved.releases[&pack("@kit/notes")][0].version,
        Version::new(1, 0, 0)
    );
    let messages: Vec<&str> = resolved
        .problems
        .iter()
        .map(|problem| problem.message.as_str())
        .collect();
    assert_eq!(
        messages.len(),
        2,
        "its versions and its files: {messages:?}"
    );
    assert!(
        messages
            .iter()
            .all(|message| message.starts_with("the cache couldn't keep it"))
    );
}
