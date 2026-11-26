use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use compmd::{Ambient, ParseOptions, Processor};
use serde_json::json;

use crate::disk_files::DiskFiles;
use crate::lock_file::integrities;
use crate::project_files::tests::Memory;
use crate::{config_of, read_config};

use super::*;

/// A site with three installed packs (one only for development), a library and a dependency it hasn't installed, and
/// a folder pack of its own.
fn site() -> Memory {
    Memory::of([
        (
            "/site/package.json",
            r#"{"dependencies": {"@kit/mermaid": "^1", "d3": "^7", "@kit/missing": "^1", "@kit/library": "^1"}, "devDependencies": {"@kit/notes": "^2", "@kit/mermaid": "^1"}}"#,
        ),
        (
            "/site/node_modules/@kit/mermaid/package.json",
            r#"{"name": "@kit/mermaid", "version": "1.2.0", "description": "Diagrams.", "compmd": "./docs/mermaid.comp.md"}"#,
        ),
        (
            "/site/node_modules/@kit/mermaid/docs/mermaid.comp.md",
            "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=diagram\n---\nA diagram.\n```\n",
        ),
        (
            "/site/node_modules/@kit/notes/package.json",
            r#"{"name": "@kit/notes", "version": "2.0.0"}"#,
        ),
        ("/site/node_modules/@kit/notes/index.comp.md", "# Notes"),
        (
            "/site/node_modules/@kit/library/package.json",
            r#"{"name": "@kit/library", "version": "1.0.0"}"#,
        ),
        (
            "/site/node_modules/d3/package.json",
            r#"{"name": "d3", "version": "7.9.0"}"#,
        ),
        (
            "/site/package-lock.json",
            r#"{"packages": {"node_modules/@kit/mermaid": {"version": "1.2.0", "integrity": "sha512-mermaid"}}}"#,
        ),
        (
            "/site/components/package.json",
            r#"{"name": "@site/components", "version": "1.0.0"}"#,
        ),
        ("/site/components/index.comp.md", "# Components"),
        ("/shared/kit/package.json", r#"{"name": "@shared/kit"}"#),
    ])
}

fn name(text: &str) -> PackName {
    text.parse().expect("a pack's name")
}

fn installed(files: &Memory) -> PackFolders<&Memory> {
    PackFolders::installed(
        Path::new("/site"),
        files,
        &integrities(Path::new("/site"), files),
    )
    .expect("the packs")
}

#[test]
fn the_installed_packs_are_the_scoped_dependencies_that_are_packs() {
    let files = site();
    let packs = installed(&files);
    let found: Vec<String> = packs
        .packs()
        .iter()
        .map(|pack| pack.release.to_string())
        .collect();
    assert_eq!(found, ["@kit/mermaid@1.2.0", "@kit/notes@2.0.0"]);
    assert_eq!(
        packs.packs()[0].dir,
        Path::new("/site/node_modules/@kit/mermaid")
    );
    assert_eq!(
        packs.packs()[0].integrity,
        Some("sha512-mermaid".parse().expect("an integrity"))
    );
    assert_eq!(packs.packs()[1].integrity, None, "not in the lock file");
    assert_eq!(
        PackFolders::installed(Path::new("/empty"), &files, &Integrities::new())
            .expect("none")
            .packs(),
        []
    );
}

#[test]
fn the_packs_found_are_known_with_their_descriptions() {
    let files = site();
    let known: Vec<(String, Option<String>)> = installed(&files)
        .known()
        .into_iter()
        .map(|known| (known.release.to_string(), known.description))
        .collect();
    assert_eq!(
        known,
        [
            (
                "@kit/mermaid@1.2.0".to_owned(),
                Some("Diagrams.".to_owned())
            ),
            ("@kit/notes@2.0.0".to_owned(), None),
        ]
    );
}

#[test]
fn an_installed_pack_named_otherwise_is_an_error() {
    let files = Memory::of([
        (
            "/site/package.json",
            r#"{"dependencies": {"@kit/mermaid": "^1"}}"#,
        ),
        (
            "/site/node_modules/@kit/mermaid/package.json",
            r#"{"name": "@kit/other", "compmd": "./a.comp.md"}"#,
        ),
    ]);
    assert_eq!(
        PackFolders::installed(Path::new("/site"), &files, &Integrities::new())
            .map(|packs| packs.packs().len()),
        Err(ProjectError::Renamed {
            dependency: "@kit/mermaid".into(),
            name: "@kit/other".into()
        })
    );
}

#[test]
fn the_folder_packs_are_the_configurations_relative_to_its_folder() {
    let files = site();
    let config = read_config(
        &json!({ "packs": { "./components": { "trust": "trusted" }, "../shared/kit": {}, "@kit/mermaid": {} } }),
        "compmd.config.json",
    )
    .expect("a configuration");
    let packs = PackFolders::listed(&config, Path::new("/site"), &files).expect("the packs");
    let found: Vec<(String, Option<&str>)> = packs
        .packs()
        .iter()
        .map(|pack| (pack.release.to_string(), pack.key.as_deref()))
        .collect();
    assert_eq!(
        found,
        [
            ("@site/components@1.0.0".to_owned(), Some("./components")),
            ("@shared/kit@0.0.0".to_owned(), Some("../shared/kit"))
        ]
    );
    assert_eq!(packs.packs()[1].dir, Path::new("/shared/kit"));
    assert_eq!(
        packs.name_of("./components"),
        Some(name("@site/components"))
    );
    assert_eq!(packs.name_of("@kit/mermaid"), None);
}

#[test]
fn a_listed_folder_that_isnt_one_or_isnt_a_pack_is_an_error() {
    let files = Memory::of([("/site/kit/readme.md", "# Kit")]);
    let listing = |key: &str| {
        read_config(&json!({ "packs": { key: {} } }), "compmd.config.json")
            .expect("a configuration")
    };
    assert_eq!(
        PackFolders::listed(&listing("./components"), Path::new("/site"), &files)
            .map(|packs| packs.packs().len()),
        Err(ProjectError::NoFolder {
            folder: "./components".into()
        })
    );
    assert_eq!(
        PackFolders::listed(&listing("./kit"), Path::new("/site"), &files)
            .map(|packs| packs.packs().len()),
        Err(ProjectError::NotAPack {
            dir: "/site/kit".into()
        })
    );
}

#[test]
fn gives_the_processor_each_packs_release_and_its_files() {
    let files = site();
    let packs = installed(&files);
    assert_eq!(
        packs.releases(&name("@kit/mermaid")),
        [Release {
            version: "1.2.0".parse().expect("a version"),
            integrity: Some("sha512-mermaid".parse().expect("an integrity")),
            broken: None
        }]
    );
    assert_eq!(packs.releases(&name("@kit/missing")), []);
    let place = |text: &str| text.parse::<Place>().expect("a place");
    assert_eq!(
        packs
            .pack_file(&place("@kit/notes@2.0.0/index.comp.md"))
            .as_deref(),
        Some("# Notes")
    );
    assert_eq!(
        packs.pack_file(&place("@kit/notes@1.0.0/index.comp.md")),
        None,
        "another version"
    );
    assert_eq!(
        packs.pack_file(&place("@kit/notes@2.0.0/none.comp.md")),
        None
    );
}

#[test]
fn a_document_imports_an_installed_pack_as_the_site_does() {
    let files = Arc::new(site());
    let packs = PackFolders::installed(
        Path::new("/site"),
        Arc::clone(&files),
        &integrities(Path::new("/site"), &files),
    )
    .expect("the packs");
    let translated = Config::default().translated(|key| key.parse().ok(), &BTreeMap::new());
    let diagnostics = |text: &str| -> Vec<String> {
        Processor::new("a.comp.md", text)
            .parse(&ParseOptions::default())
            .resolve_with((), packs.clone(), &Ambient::default(), &translated.policy)
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code.to_string())
            .collect()
    };
    let import = |from: &str| {
        format!(
            "---\nmd:\n  import:\n    - from: \"{from}\"\n      names: [diagram]\n---\n\n:: diagram[A diagram]\n"
        )
    };
    assert_eq!(diagnostics(&import("@kit/mermaid@1")), Vec::<String>::new());
    assert_eq!(diagnostics(&import("@kit/mermaid@2")), ["F026"]);
}

#[test]
fn reads_the_vite_plugins_example_app_as_the_plugin_does() {
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../npm-packages/vite/example");
    let config = config_of(&example, &DiskFiles).expect("its configuration");
    let installed = PackFolders::installed(&example, DiskFiles, &integrities(&example, &DiskFiles))
        .expect("its installed packs");
    let found: Vec<String> = installed
        .packs()
        .iter()
        .map(|pack| pack.release.to_string())
        .collect();
    assert_eq!(found, ["@example/charts@1.0.0", "@example/old@1.0.0"]);
    let listed = PackFolders::listed(&config, &example, DiskFiles).expect("its folder packs");
    assert_eq!(listed.name_of("./components"), Some(name("@example/app")));
    let translated = config.translated(
        |key| listed.name_of(key).or_else(|| key.parse().ok()),
        &BTreeMap::new(),
    );
    assert_eq!(translated.ambient, [name("@example/app")]);
    assert_eq!(
        translated.policy.trust(&name("@example/app")),
        compmd::policy::Trust::Trusted
    );
    assert_eq!(translated.policy.deny[0].pack, name("@example/old"));
}

#[test]
fn a_folder_path_resolves_its_dots_without_the_disk() {
    assert_eq!(
        joined(Path::new("/site/readers"), "../components"),
        Path::new("/site/components")
    );
    assert_eq!(
        joined(Path::new("/site"), "./notes/./kit"),
        Path::new("/site/notes/kit")
    );
}
