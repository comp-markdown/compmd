//! Packs from a registry served on the machine, as an editor opens documents that import them.

use std::path::Path;

use compmd_host::registry::{LocalRegistry, ServedPack};

use super::*;

/// A document importing `@kit/notes@1`'s `note`, and `absent`, which the pack doesn't export (F021 once it loads).
const DOCUMENT: &str = "---\nmd:\n  import:\n    - from: \"@kit/notes@1\"\n      names: [note, absent]\n---\n\n:: note[Hi]\n";

fn notes() -> ServedPack {
    let package = r#"{"name": "@kit/notes", "version": "1.0.0", "compmd": "index.comp.md"}"#;
    let document = "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=note\n---\nA note.\n```\n";
    ServedPack::new("@kit/notes").version(
        "1.0.0",
        &[("package.json", package), ("index.comp.md", document)],
    )
}

/// The project in `dir`, its packs from `registry`, and its document's URI.
fn project(dir: &Path, registry: &str) -> String {
    let config = json!({ "registries": { "@kit": registry } });
    fs::write(dir.join("compmd.config.json"), config.to_string()).expect("written");
    uri_of(&dir.join("a.comp.md"))
}

fn open(client: &Client, uri: &str) {
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": DOCUMENT } }));
}

#[test]
fn a_registry_pack_is_pending_then_fetched_and_kept_for_the_next_session() {
    let dir = folder("registry-fetched");
    let cache = dir.join("cache");
    let uri = {
        let registry = LocalRegistry::serve(&[notes()]).expect("served");
        let uri = project(&dir, registry.url());
        let (client, _) = Client::start_with(json!({ "capabilities": {} }), cache.clone());
        open(&client, &uri);
        assert_eq!(
            client.diagnostics(),
            Vec::<String>::new(),
            "pending, not F026"
        );
        assert_eq!(
            client.diagnostics(),
            ["F021"],
            "loaded: `absent` isn't a name of the pack's"
        );
        client.stop();
        uri
    };
    let (client, _) = Client::start_with(json!({ "capabilities": {} }), cache);
    open(&client, &uri);
    assert_eq!(client.diagnostics(), Vec::<String>::new());
    assert_eq!(
        client.diagnostics(),
        ["F021"],
        "from the cache, its registry gone"
    );
    client.stop();
}

#[test]
fn an_import_typed_in_is_fetched() {
    let dir = folder("registry-typed");
    let registry = LocalRegistry::serve(&[notes()]).expect("served");
    let uri = project(&dir, registry.url());
    let (client, _) = Client::start_with(json!({ "capabilities": {} }), dir.join("cache"));
    let body = DOCUMENT
        .split_once("---\n\n")
        .map(|(_, body)| body)
        .unwrap_or_default();
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": body } }));
    assert_eq!(client.diagnostics(), ["F002"], "no import yet");
    let frontmatter = DOCUMENT.trim_end_matches(body);
    let at =
        json!({ "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } });
    client.notify("textDocument/didChange", json!({ "textDocument": { "uri": uri, "version": 2 }, "contentChanges": [{ "range": at, "text": frontmatter }] }));
    assert_eq!(client.diagnostics(), Vec::<String>::new(), "pending");
    assert_eq!(client.diagnostics(), ["F021"]);
    client.stop();
}

#[test]
fn a_registry_pack_that_cant_be_had_is_f026() {
    let dir = folder("registry-missing");
    let registry = LocalRegistry::serve(&[]).expect("served");
    let uri = project(&dir, registry.url());
    let (client, _) = Client::start_with(json!({ "capabilities": {} }), dir.join("cache"));
    open(&client, &uri);
    assert_eq!(client.diagnostics(), Vec::<String>::new(), "pending");
    assert_eq!(client.diagnostics(), ["F026"], "the registry hasn't it");
    assert_eq!(registry.asked(), ["/@kit%2fnotes"]);
    client.stop();
}

#[test]
fn with_the_network_off_only_the_cache_is_used() {
    let dir = folder("registry-offline");
    let registry = LocalRegistry::serve(&[notes()]).expect("served");
    let uri = project(&dir, registry.url());
    let settings =
        json!({ "capabilities": {}, "initializationOptions": { "packs": { "network": false } } });
    let (client, _) = Client::start_with(settings, dir.join("cache"));
    open(&client, &uri);
    assert_eq!(client.diagnostics(), Vec::<String>::new());
    let log = client.next_of(LogMessage::METHOD);
    assert_eq!(
        log["message"],
        "`@kit/notes` couldn't be fetched: the network is off"
    );
    assert_eq!(client.diagnostics(), ["F026"]);
    assert_eq!(registry.asked(), Vec::<String>::new());
    client.stop();
}
