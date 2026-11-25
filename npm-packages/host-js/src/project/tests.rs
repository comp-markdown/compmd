use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Value, json};

use super::*;

/// A project in memory: its files by path; every folder above a file is a folder.
struct Memory(BTreeMap<String, String>);

impl Memory {
    fn of(files: &[(&str, &str)]) -> Self {
        Self(
            files
                .iter()
                .map(|(path, text)| ((*path).to_owned(), (*text).to_owned()))
                .collect(),
        )
    }
}

impl ProjectFiles for Memory {
    fn text(&self, path: &Path) -> Option<String> {
        self.0.get(path.to_str()?).cloned()
    }

    fn is_dir(&self, path: &Path) -> bool {
        self.0
            .keys()
            .any(|file| Path::new(file) != path && Path::new(file).starts_with(path))
    }
}

fn parsed(text: &str) -> Value {
    serde_json::from_str(text).expect("JSON")
}

/// A site with an installed pack, a folder pack of its own and its lock file.
fn site() -> Memory {
    Memory::of(&[
        (
            "/site/compmd.config.json",
            r#"{"packs": {"./components": {"trust": "trusted", "ambient": true}, "@kit/mermaid": {"version": "^1"}}, "deny": [{"pack": "@kit/old", "version": "<2", "reason": "old"}], "maxPackSize": "5 MB", "libraries": ["d3"], "registries": {"*": "https://registry.npmjs.org"}, "fetch": "direct", "origins": "any", "editor": {"suggest": ["@acme"]}}"#,
        ),
        (
            "/site/package.json",
            r#"{"dependencies": {"@kit/mermaid": "^1"}}"#,
        ),
        (
            "/site/node_modules/@kit/mermaid/package.json",
            r#"{"name": "@kit/mermaid", "version": "1.2.0", "compmd": "./a.comp.md"}"#,
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
    ])
}

#[test]
fn a_configuration_is_written_as_the_plugin_reads_it() {
    assert_eq!(
        parsed(&config_of("/site", &site())),
        json!({ "config": {
            "packs": [
                ["./components", { "trust": { "level": "trusted" }, "ambient": true }],
                ["@kit/mermaid", { "trust": { "level": "strict" }, "ambient": false, "version": "^1" }],
            ],
            "imports": "any",
            "deny": [{ "pack": "@kit/old", "version": "<2", "reason": "old" }],
            "maxPackSize": 5_000_000.0,
            "libraries": [{ "name": "d3", "imports": ["d3"] }],
            "registries": { "*": "https://registry.npmjs.org/" },
            "fetch": "direct",
            "origins": "any",
            "editor": { "suggest": ["@acme"] },
        } })
    );
    let defaults = parsed(&read_config("{}", "compmd.config.json"));
    assert_eq!(
        (
            &defaults["config"]["imports"],
            &defaults["config"]["fetch"],
            &defaults["config"]["origins"]
        ),
        (&json!("any"), &json!("host"), &json!([]))
    );
    assert_eq!(
        parsed(&read_config(r#"{"imports": "listed"}"#, "x"))["config"]["imports"],
        json!("listed")
    );
}

#[test]
fn a_wrong_configuration_says_every_problem_and_a_project_that_cant_be_read_why() {
    assert_eq!(
        parsed(&read_config(
            r#"{"fetch": "proxy"}"#,
            "package.json (`compmd`)"
        )),
        json!({ "problems": ["fetch: `host` or `direct`"], "source": "package.json (`compmd`)", "message": "package.json (`compmd`):\n- fetch: `host` or `direct`" })
    );
    assert_eq!(
        parsed(&read_config("not json", "x"))["problems"],
        json!(["the configuration is an object"])
    );
    let broken = Memory::of(&[
        ("/b/compmd.config.json", "{"),
        ("/w/compmd.config.json", r#"{"fetch": 1}"#),
    ]);
    assert!(
        parsed(&config_of("/b", &broken))["error"]
            .as_str()
            .is_some_and(|error| error.contains("isn't JSON"))
    );
    assert_eq!(
        parsed(&config_of("/w", &broken))["problems"],
        json!(["fetch: `host` or `direct`"])
    );
}

#[test]
fn the_packs_found_have_their_folder_key_and_integrity() {
    assert_eq!(
        parsed(&installed_packs("/site", &site())),
        json!({ "packs": [{ "name": "@kit/mermaid", "version": "1.2.0", "dir": "/site/node_modules/@kit/mermaid", "integrity": "sha512-mermaid" }] })
    );
    let keys = ["./components".to_owned(), "@kit/mermaid".to_owned()];
    assert_eq!(
        parsed(&folder_packs(&keys, "/site", &site())),
        json!({ "packs": [{ "name": "@site/components", "version": "1.0.0", "dir": "/site/components", "key": "./components" }] })
    );
    assert_eq!(
        parsed(&folder_packs(&["./none".to_owned()], "/site", &site())),
        json!({ "error": "./none is listed in the configuration, but it isn't a folder of the app" })
    );
}

#[test]
fn the_lock_file_and_the_root_are_found_as_the_crate_finds_them() {
    assert_eq!(
        parsed(&integrities("/site/posts", &site())),
        json!({ "@kit/mermaid@1.2.0": "sha512-mermaid" })
    );
    assert_eq!(
        lock_file_of("/site", &site()).as_deref(),
        Some("/site/package-lock.json")
    );
    assert_eq!(lock_file_of("/elsewhere", &site()), None);
    assert_eq!(root_of("/site/posts/a.comp.md", &[], &site()), "/site");
    assert_eq!(
        root_of("/notes/a.comp.md", &["/notes".to_owned()], &site()),
        "/notes"
    );
}
