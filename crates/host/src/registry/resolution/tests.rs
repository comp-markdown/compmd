//! The resolution driven by a host in memory: a registry's metadata and tarballs, a cache, and a stand-in for the
//! processor's `sources`, as `@compmd/registry`'s `resolve.test.ts` drives the TypeScript resolver.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::json;

use super::*;
use crate::registry::integrity::integrity_of;
use crate::registry::step::KeptVersions;
use crate::registry::tar::tests::tarball;
use crate::registry::{DEFAULT_MAX_PACK_SIZE, Denied};

const NPM: &str = "https://registry.npmjs.org/";

fn name(text: &str) -> PackName {
    text.parse().expect("a pack's name")
}

fn import(pack: &str, range: &str) -> Needed {
    Needed {
        pack: name(pack),
        range: range.into(),
        integrity: None,
        library: false,
    }
}

/// A release in the registry: its pack, its version, and its files by path.
type ReleaseFiles<'a> = (&'a str, &'a str, &'a [(&'a str, &'a [u8])]);

/// A host: what the request imports, the registry's metadata and tarballs, the cache, and what it did.
#[derive(Default)]
struct Host {
    imports: Vec<Needed>,
    /// What a pack's release brings in turn, once added: its library packs.
    brings: BTreeMap<String, Vec<Needed>>,
    metadata: BTreeMap<String, Answer>,
    tarballs: BTreeMap<String, Answer>,
    kept_versions: BTreeMap<String, KeptVersions>,
    kept_files: BTreeMap<String, Files>,
    have: BTreeSet<PackName>,
    refused: BTreeSet<PackName>,
    /// Each fetch and keep, in order.
    log: Vec<String>,
    /// Packs to give up on at the first step that touches them.
    give_up: Vec<PackName>,
}

impl Host {
    /// A registry with each of `releases`: a pack, its version, its files, and the metadata to change.
    fn with_releases(mut self, releases: &[ReleaseFiles<'_>]) -> Self {
        let mut by_pack: BTreeMap<String, serde_json::Map<String, serde_json::Value>> =
            BTreeMap::new();
        for (pack, version, files) in releases {
            let bytes = tarball(files);
            let url = format!("{NPM}{pack}/-/{version}.tgz");
            let size: usize = files.iter().map(|(_, data)| data.len()).sum();
            by_pack.entry((*pack).to_owned()).or_default().insert(
                (*version).to_owned(),
                json!({ "dist": { "tarball": url, "integrity": integrity_of(&bytes), "unpackedSize": size } }),
            );
            self.tarballs.insert(url, Answer::Tarball { bytes });
        }
        for (pack, versions) in by_pack {
            let body = json!({ "name": pack, "versions": versions }).to_string();
            self.metadata.insert(
                metadata_url(NPM, &name(&pack)),
                Answer::Metadata {
                    status: 200,
                    body,
                    max_age: 300,
                },
            );
        }
        self
    }

    /// The metadata of `pack` as `edit` changes it.
    fn edit_metadata(&mut self, pack: &str, edit: impl FnOnce(&mut serde_json::Value)) {
        let url = metadata_url(NPM, &name(pack));
        let Some(Answer::Metadata { body, .. }) = self.metadata.get_mut(&url) else {
            panic!("no metadata for {pack}");
        };
        let mut value: serde_json::Value = serde_json::from_str(body).expect("JSON");
        edit(&mut value);
        *body = value.to_string();
    }

    fn perform(&mut self, step: &Step, resolution: &mut Resolution) -> Answer {
        match step {
            Step::Missing {
                releases, refused, ..
            } => {
                self.have
                    .extend(releases.iter().map(|(pack, _)| pack.clone()));
                self.refused.extend(refused.keys().cloned());
                let brought = self
                    .have
                    .iter()
                    .filter_map(|pack| self.brings.get(pack.as_str()))
                    .flatten()
                    .cloned();
                let packs = self
                    .imports
                    .iter()
                    .cloned()
                    .chain(brought)
                    .filter(|needed| {
                        !self.have.contains(&needed.pack) && !self.refused.contains(&needed.pack)
                    })
                    .collect();
                Answer::Missing { packs }
            }
            Step::KeptVersions { registry, pack, .. } => {
                if let Some(late) = self.give_up.iter().position(|late| late == pack) {
                    resolution.give_up(&self.give_up.remove(late), 30);
                }
                Answer::KeptVersions {
                    kept: self
                        .kept_versions
                        .get(&format!("{registry}{pack}"))
                        .cloned(),
                }
            }
            Step::FetchVersions { url, accept, .. } => {
                assert_eq!(accept, ABBREVIATED);
                self.log.push(format!("versions {url}"));
                self.metadata.get(url).cloned().unwrap_or(Answer::Metadata {
                    status: 404,
                    body: String::new(),
                    max_age: 0,
                })
            }
            Step::KeepVersions {
                pack,
                versions,
                max_age,
                ..
            } => {
                self.log
                    .push(format!("keep versions {pack} {} {max_age}", versions.len()));
                Answer::Done
            }
            Step::Kept { release, .. } => Answer::Kept {
                files: self.kept_files.get(&release.key()).cloned(),
            },
            Step::FetchTarball { url, max_bytes, .. } => {
                assert_eq!(*max_bytes, DEFAULT_MAX_PACK_SIZE);
                self.log.push(format!("tarball {url}"));
                self.tarballs.get(url).cloned().unwrap_or(Answer::Failed {
                    message: format!("{url} answered 404"),
                })
            }
            Step::Keep { release, .. } => {
                self.log
                    .push(format!("keep {}@{}", release.pack, release.version));
                Answer::Done
            }
        }
    }

    /// Resolves with `options`, as a host drives it.
    fn resolve(&mut self, options: Options) -> Resolved {
        let mut resolution = Resolution::new(options);
        loop {
            match resolution.next_steps() {
                Next::Steps(steps) => {
                    assert!(!steps.is_empty(), "a resolution that isn't done has steps");
                    for step in steps {
                        let answer = self.perform(&step, &mut resolution);
                        resolution.give(step.id(), answer);
                    }
                }
                Next::Done(resolved) => return resolved,
            }
        }
    }
}

fn npm() -> Options {
    Options::new(BTreeMap::from([("*".to_owned(), NPM.to_owned())]))
}

/// A pack with a document and a built module, and an icon.
const MERMAID: &[(&str, &[u8])] = &[
    (
        "package.json",
        br#"{"name": "@kit/mermaid", "version": "1.2.0"}"#,
    ),
    ("index.comp.md", b"# Mermaid"),
    ("dist/mermaid.js", b"export default 1;"),
    ("icons/logo.png", &[0x89, 0x50, 0xff, 0x00]),
];

#[test]
fn resolves_an_import_to_its_newest_matching_release_with_its_files_checked() {
    let mut host = Host {
        imports: vec![import("@kit/mermaid", "1")],
        ..Host::default()
    }
    .with_releases(&[
        ("@kit/mermaid", "1.2.0", MERMAID),
        ("@kit/mermaid", "1.1.0", MERMAID),
        ("@kit/mermaid", "2.0.0", MERMAID),
    ]);
    let resolved = host.resolve(npm());
    let release = &resolved.releases[&name("@kit/mermaid")][0];
    assert_eq!(release.version, Version::new(1, 2, 0));
    assert!(
        release
            .integrity
            .as_deref()
            .is_some_and(|integrity| integrity.starts_with("sha512-"))
    );
    assert_eq!(
        release.files.keys().collect::<Vec<_>>(),
        ["dist/mermaid.js", "index.comp.md", "package.json"]
    );
    assert_eq!(
        resolved.modules["@kit/mermaid@1.2.0/dist/mermaid.js"],
        "export default 1;"
    );
    assert_eq!(
        resolved.files["@kit/mermaid@1.2.0/icons/logo.png"].media_type,
        "image/png"
    );
    assert_eq!(resolved.problems, []);
    assert_eq!(
        host.log,
        [
            "versions https://registry.npmjs.org/@kit%2fmermaid",
            "keep versions @kit/mermaid 3 300",
            "tarball https://registry.npmjs.org/@kit/mermaid/-/1.2.0.tgz",
            "keep @kit/mermaid@1.2.0",
        ]
    );
}

#[test]
fn takes_kept_versions_and_files_without_the_registry() {
    let mut host = Host {
        imports: vec![import("@kit/mermaid", "1")],
        ..Host::default()
    }
    .with_releases(&[("@kit/mermaid", "1.2.0", MERMAID)]);
    let Some(Answer::Metadata { body, .. }) = host.metadata.values().next().cloned() else {
        panic!("metadata");
    };
    let versions = versions_of(&body, NPM).expect("versions");
    let release = KeptRelease {
        pack: name("@kit/mermaid"),
        version: Version::new(1, 2, 0),
        integrity: versions[0].integrity.clone().expect("an integrity"),
    };
    host.kept_versions.insert(
        format!("{NPM}@kit/mermaid"),
        KeptVersions {
            versions,
            fresh: true,
        },
    );
    host.kept_files.insert(
        release.key(),
        Files {
            text: BTreeMap::from([("index.comp.md".into(), "# Kept".into())]),
            binary: BTreeMap::new(),
        },
    );
    let resolved = host.resolve(npm());
    assert_eq!(
        resolved.releases[&name("@kit/mermaid")][0].files["index.comp.md"],
        "# Kept"
    );
    assert_eq!(host.log, Vec::<String>::new(), "nothing fetched");
}

#[test]
fn a_registry_that_fails_gives_the_stale_list_or_a_problem() {
    let mut host = Host {
        imports: vec![import("@kit/mermaid", "1")],
        ..Host::default()
    }
    .with_releases(&[("@kit/mermaid", "1.2.0", MERMAID)]);
    let Some(Answer::Metadata { body, .. }) = host.metadata.values().next().cloned() else {
        panic!("metadata");
    };
    host.kept_versions.insert(
        format!("{NPM}@kit/mermaid"),
        KeptVersions {
            versions: versions_of(&body, NPM).expect("versions"),
            fresh: false,
        },
    );
    host.metadata.insert(
        metadata_url(NPM, &name("@kit/mermaid")),
        Answer::Failed {
            message: "offline".into(),
        },
    );
    assert_eq!(
        host.resolve(npm()).releases[&name("@kit/mermaid")].len(),
        1,
        "the stale list"
    );
    let mut failing = Host {
        imports: vec![import("@kit/mermaid", "1")],
        ..Host::default()
    };
    failing.metadata.insert(
        metadata_url(NPM, &name("@kit/mermaid")),
        Answer::Metadata {
            status: 500,
            body: String::new(),
            max_age: 0,
        },
    );
    assert_eq!(
        failing.resolve(npm()).problems,
        [Problem {
            pack: name("@kit/mermaid"),
            message: "https://registry.npmjs.org/ answered 500 for `@kit/mermaid`".into()
        }]
    );
    let mut broken = Host {
        imports: vec![import("@kit/mermaid", "1")],
        ..Host::default()
    };
    broken.metadata.insert(
        metadata_url(NPM, &name("@kit/mermaid")),
        Answer::Metadata {
            status: 200,
            body: "{".into(),
            max_age: 0,
        },
    );
    assert!(
        broken.resolve(npm()).problems[0]
            .message
            .contains("isn't JSON")
    );
}

#[test]
fn a_pack_the_registry_doesnt_have_or_no_release_matches_stays_missing() {
    let mut host = Host {
        imports: vec![import("@kit/none", "1"), import("@kit/mermaid", "^3")],
        ..Host::default()
    }
    .with_releases(&[("@kit/mermaid", "1.2.0", MERMAID)]);
    let resolved = host.resolve(npm());
    assert_eq!(
        (resolved.releases.len(), resolved.problems.len()),
        (0, 0),
        "the processor's F026"
    );
    let mut unscoped = Host {
        imports: vec![import("@kit/mermaid", "1")],
        ..Host::default()
    };
    let options = Options::new(BTreeMap::from([("@other".to_owned(), NPM.to_owned())]));
    assert_eq!(
        unscoped.resolve(options).problems,
        [Problem {
            pack: name("@kit/mermaid"),
            message: "no registry is configured for `@kit/mermaid`'s scope".into()
        }]
    );
}

#[test]
fn the_policy_holds_before_any_download() {
    let mut host = Host {
        imports: vec![
            import("@kit/big", "1"),
            import("@kit/unsized", "1"),
            import("@kit/sha1", "1"),
            import("@kit/old", "1"),
        ],
        ..Host::default()
    }
    .with_releases(&[
        ("@kit/big", "1.0.0", MERMAID),
        ("@kit/unsized", "1.0.0", MERMAID),
        ("@kit/sha1", "1.0.0", MERMAID),
        ("@kit/old", "1.0.0", MERMAID),
    ]);
    host.edit_metadata("@kit/big", |metadata| {
        metadata["versions"]["1.0.0"]["dist"]["unpackedSize"] = json!(25_000_000);
    });
    host.edit_metadata("@kit/unsized", |metadata| {
        metadata["versions"]["1.0.0"]["dist"]
            .as_object_mut()
            .expect("dist")
            .remove("unpackedSize");
    });
    host.edit_metadata("@kit/sha1", |metadata| {
        metadata["versions"]["1.0.0"]["dist"]["integrity"] = json!("sha1-abc");
    });
    let mut options = npm();
    options.deny.push(Denied {
        pack: name("@kit/old"),
        version: Some("<2".into()),
    });
    let resolved = host.resolve(options);
    assert_eq!(
        resolved.refused,
        BTreeMap::from([
            (name("@kit/big"), "`@kit/big@1.0.0` is 25.0 MB unpacked, more than this host's 10 MB".to_owned()),
            (name("@kit/sha1"), "its registry publishes no SHA-512 integrity for `@kit/sha1@1.0.0`, so it can't be checked".to_owned()),
            (
                name("@kit/unsized"),
                "its registry doesn't say how large `@kit/unsized@1.0.0` is, and this host takes packs up to 10 MB".to_owned()
            ),
        ])
    );
    assert_eq!(
        resolved.releases[&name("@kit/old")][0].files,
        BTreeMap::new(),
        "denied: the processor's F026, no files"
    );
    assert!(
        host.log.iter().all(|line| !line.starts_with("tarball")),
        "nothing downloaded: {:?}",
        host.log
    );
}

#[test]
fn a_tarball_is_checked_against_its_integrity_and_its_registry() {
    let mut host = Host {
        imports: vec![import("@kit/mermaid", "1")],
        ..Host::default()
    }
    .with_releases(&[("@kit/mermaid", "1.2.0", MERMAID)]);
    host.tarballs.insert(
        format!("{NPM}@kit/mermaid/-/1.2.0.tgz"),
        Answer::Tarball {
            bytes: tarball(&[("index.comp.md", b"# Changed")]),
        },
    );
    let release = &host.resolve(npm()).releases[&name("@kit/mermaid")][0];
    assert_eq!(
        release.broken.as_deref(),
        Some("its tarball doesn't match the integrity its registry publishes")
    );
    assert_eq!(release.files, BTreeMap::new());
    let mut elsewhere = Host {
        imports: vec![import("@kit/mermaid", "1")],
        ..Host::default()
    }
    .with_releases(&[("@kit/mermaid", "1.2.0", MERMAID)]);
    elsewhere.edit_metadata("@kit/mermaid", |metadata| {
        metadata["versions"]["1.2.0"]["dist"]["tarball"] = json!("https://evil.example/x.tgz");
    });
    assert!(
        elsewhere.resolve(npm()).problems[0]
            .message
            .contains("isn't on its registry's origin")
    );
}

#[test]
fn a_tarball_past_the_cap_or_unreadable_is_a_problem() {
    let mut large = Host {
        imports: vec![import("@kit/mermaid", "1")],
        ..Host::default()
    }
    .with_releases(&[("@kit/mermaid", "1.2.0", MERMAID)]);
    let url = format!("{NPM}@kit/mermaid/-/1.2.0.tgz");
    large.tarballs.insert(
        url.clone(),
        Answer::Tarball {
            bytes: vec![0; 10_000_001],
        },
    );
    assert_eq!(
        large.resolve(npm()).problems[0].message,
        format!("{url} is larger than 10000000 bytes")
    );
    let mut failed = Host {
        imports: vec![import("@kit/mermaid", "1")],
        ..Host::default()
    }
    .with_releases(&[("@kit/mermaid", "1.2.0", MERMAID)]);
    failed.tarballs.insert(
        url.clone(),
        Answer::Failed {
            message: "reset".into(),
        },
    );
    assert_eq!(failed.resolve(npm()).problems[0].message, "reset");
    let mut not_gzip = Host {
        imports: vec![import("@kit/mermaid", "1")],
        ..Host::default()
    }
    .with_releases(&[("@kit/mermaid", "1.2.0", MERMAID)]);
    let bytes = b"not gzip".to_vec();
    not_gzip.edit_metadata("@kit/mermaid", |metadata| {
        metadata["versions"]["1.2.0"]["dist"]["integrity"] = json!(integrity_of(&bytes));
    });
    not_gzip.tarballs.insert(url, Answer::Tarball { bytes });
    assert_eq!(
        not_gzip.resolve(npm()).problems[0].message,
        "the tarball isn't gzip"
    );
}

#[test]
fn a_pinned_integrity_is_checked_and_a_kept_pinned_release_needs_no_registry() {
    let mut host = Host::default().with_releases(&[("@kit/mermaid", "1.2.0", MERMAID)]);
    let pinned = Needed {
        integrity: Some("sha512-other".parse().expect("an integrity")),
        ..import("@kit/mermaid", "1.2.0")
    };
    host.imports = vec![pinned.clone()];
    let release = &host.resolve(npm()).releases[&name("@kit/mermaid")][0];
    assert_eq!(release.files, BTreeMap::new(), "the processor's F035");
    let mut kept = Host {
        imports: vec![pinned],
        ..Host::default()
    };
    let key = KeptRelease {
        pack: name("@kit/mermaid"),
        version: Version::new(1, 2, 0),
        integrity: "sha512-other".into(),
    }
    .key();
    kept.kept_files.insert(
        key,
        Files {
            text: BTreeMap::from([("index.comp.md".into(), "# Pinned".into())]),
            binary: BTreeMap::new(),
        },
    );
    let resolved = kept.resolve(npm());
    assert_eq!(
        resolved.releases[&name("@kit/mermaid")][0].files["index.comp.md"],
        "# Pinned"
    );
    assert_eq!(kept.log, Vec::<String>::new());
}

#[test]
fn a_library_resolves_to_the_version_every_range_matches_and_a_release_is_fetched_once() {
    let library = |range: &str| Needed {
        library: true,
        ..import("@kit/d3", range)
    };
    let mut host = Host {
        imports: vec![import("@kit/charts", "1"), import("@kit/maps", "1")],
        ..Host::default()
    }
    .with_releases(&[
        ("@kit/charts", "1.0.0", MERMAID),
        ("@kit/maps", "1.0.0", MERMAID),
        ("@kit/d3", "7.8.0", MERMAID),
        ("@kit/d3", "7.9.0", MERMAID),
        ("@kit/d3", "8.0.0", MERMAID),
    ]);
    host.brings
        .insert("@kit/charts".into(), vec![library("^7")]);
    host.brings
        .insert("@kit/maps".into(), vec![library(">=7.8")]);
    let resolved = host.resolve(npm());
    assert_eq!(
        resolved.releases[&name("@kit/d3")]
            .iter()
            .map(|release| release.version.to_string())
            .collect::<Vec<_>>(),
        ["7.9.0"]
    );
    assert_eq!(
        host.log
            .iter()
            .filter(|line| line.contains("@kit/d3/-/"))
            .count(),
        1,
        "fetched once: {:?}",
        host.log
    );
}

#[test]
fn a_pack_given_up_on_is_a_problem_and_answers_after_change_nothing() {
    let mut host = Host {
        imports: vec![import("@kit/mermaid", "1")],
        ..Host::default()
    }
    .with_releases(&[("@kit/mermaid", "1.2.0", MERMAID)]);
    host.give_up = vec![name("@kit/mermaid")];
    let resolved = host.resolve(npm());
    assert_eq!(
        resolved.problems,
        [Problem {
            pack: name("@kit/mermaid"),
            message: "`@kit/mermaid` didn't come within 30 s".into()
        }]
    );
    assert_eq!(resolved.releases, BTreeMap::new());
    let mut resolution = Resolution::new(npm());
    resolution.give(99, Answer::Done);
    let Next::Steps(steps) = resolution.next_steps() else {
        panic!("steps");
    };
    resolution.give(steps[0].id(), Answer::Done);
    assert!(
        matches!(resolution.next_steps(), Next::Steps(steps) if steps.is_empty()),
        "the wrong answer, ignored"
    );
}

#[test]
fn steps_and_answers_are_json_a_host_in_javascript_passes() {
    let step = Step::FetchTarball {
        id: 3,
        url: "https://registry.npmjs.org/x.tgz".into(),
        max_bytes: 10,
    };
    assert_eq!(
        serde_json::to_value(&step).expect("JSON"),
        json!({ "kind": "fetchTarball", "id": 3, "url": "https://registry.npmjs.org/x.tgz", "maxBytes": 10 })
    );
    let answer: Answer = serde_json::from_value(
        json!({ "kind": "metadata", "status": 200, "body": "{}", "maxAge": 60 }),
    )
    .expect("an answer");
    assert_eq!(
        answer,
        Answer::Metadata {
            status: 200,
            body: "{}".into(),
            max_age: 60
        }
    );
    let missing: Answer = serde_json::from_value(json!({ "kind": "missing", "packs": [{ "pack": "@kit/d3", "range": "^7", "library": true }] })).expect("an answer");
    assert_eq!(
        missing,
        Answer::Missing {
            packs: vec![Needed {
                library: true,
                ..import("@kit/d3", "^7")
            }]
        }
    );
}
