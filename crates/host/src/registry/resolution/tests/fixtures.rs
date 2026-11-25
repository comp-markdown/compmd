//! The shared cases (`fixtures/registry/`, the registry rules' decision 7), which `@compmd/registry` runs through the
//! WebAssembly build too: each case's registry served by the test host, as an npm registry.

use std::fs;
use std::path::PathBuf;

use serde_json::{Value, json};

use super::{Host, NPM, npm};
use crate::registry::integrity::integrity_of;
use crate::registry::tar::tests::tarball;
use crate::registry::{Answer, Denied, Needed, metadata_url};

/// Each case, with its file's name.
fn cases() -> Vec<(String, Value)> {
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/registry");
    let mut files: Vec<PathBuf> = fs::read_dir(&folder)
        .expect("the cases")
        .map(|entry| entry.expect("a case").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();
    files
        .into_iter()
        .map(|path| {
            let text = fs::read_to_string(&path).expect("a case");
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned());
            (
                name.unwrap_or_default(),
                serde_json::from_str(&text).expect("JSON"),
            )
        })
        .collect()
}

fn needed(value: &Value) -> Vec<Needed> {
    serde_json::from_value(value.clone()).unwrap_or_default()
}

/// The host serving `case`'s registry.
fn host_of(case: &Value) -> Host {
    let mut host = Host {
        imports: needed(&case["imports"]),
        ..Host::default()
    };
    for (pack, brought) in case["brings"].as_object().into_iter().flatten() {
        host.brings.insert(pack.clone(), needed(brought));
    }
    let mut metadata: Vec<(String, serde_json::Map<String, Value>)> = Vec::new();
    for release in case["registry"].as_array().expect("releases") {
        let (pack, version) = (
            release["pack"].as_str().expect("a pack"),
            release["version"].as_str().expect("a version"),
        );
        let files: Vec<(String, Vec<u8>)> = release["files"]
            .as_object()
            .into_iter()
            .flatten()
            .map(|(path, text)| {
                (
                    path.clone(),
                    text.as_str().unwrap_or_default().as_bytes().to_vec(),
                )
            })
            .collect();
        let entries: Vec<(&str, &[u8])> = files
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.as_slice()))
            .collect();
        let bytes = tarball(&entries);
        let url = format!("{NPM}{pack}/-/{version}.tgz");
        let size: usize = files.iter().map(|(_, bytes)| bytes.len()).sum();
        let mut dist =
            json!({ "tarball": url, "integrity": integrity_of(&bytes), "unpackedSize": size });
        for (key, value) in release["dist"].as_object().into_iter().flatten() {
            if value.is_null() {
                dist.as_object_mut().map(|dist| dist.remove(key));
            } else {
                dist[key] = value.clone();
            }
        }
        let served = if release["corrupt"] == json!(true) {
            tarball(&[("index.comp.md", b"# Changed")])
        } else {
            bytes
        };
        host.tarballs.insert(url, Answer::Tarball { bytes: served });
        match metadata.iter_mut().find(|(known, _)| known == pack) {
            Some((_, versions)) => {
                versions.insert(version.to_owned(), json!({ "dist": dist }));
            }
            None => metadata.push((
                pack.to_owned(),
                serde_json::Map::from_iter([(version.to_owned(), json!({ "dist": dist }))]),
            )),
        }
    }
    for (pack, versions) in metadata {
        let body = json!({ "name": pack, "versions": versions }).to_string();
        host.metadata
            .insert(metadata_url(NPM, &super::name(&pack)), Ok((200, body)));
    }
    host
}

#[test]
fn the_shared_cases_hold() {
    let cases = cases();
    assert!(cases.len() >= 4, "the cases were read");
    for (file, case) in cases {
        let mut host = host_of(&case);
        let mut options = npm();
        if let Some(size) = case["options"]["maxPackSize"].as_u64() {
            options.max_pack_size = size;
        }
        options.deny = serde_json::from_value::<Vec<Denied>>(case["options"]["deny"].clone())
            .unwrap_or_default();
        let resolved = host.resolve(options);
        let releases: serde_json::Map<String, Value> = resolved
            .releases
            .iter()
            .map(|(pack, releases)| {
                let listed: Vec<Value> = releases
                    .iter()
                    .map(|release| json!({ "version": release.version.to_string(), "files": release.files.keys().collect::<Vec<_>>() }))
                    .collect();
                (pack.to_string(), json!(listed))
            })
            .collect();
        let broken: Vec<String> = resolved
            .releases
            .iter()
            .flat_map(|(pack, releases)| {
                releases
                    .iter()
                    .filter(|release| release.broken.is_some())
                    .map(move |release| format!("{pack}@{}", release.version))
            })
            .collect();
        let downloaded: Vec<String> = host
            .log
            .iter()
            .filter_map(|line| line.strip_prefix(&format!("tarball {NPM}")))
            .map(|rest| rest.replace("/-/", "@").trim_end_matches(".tgz").to_owned())
            .collect();
        let got = json!({
            "releases": releases,
            "broken": broken,
            "refused": resolved.refused,
            "problems": resolved.problems,
            "downloaded": downloaded,
        });
        assert_eq!(got, case["resolved"], "{file}: {}", case["name"]);
    }
}
