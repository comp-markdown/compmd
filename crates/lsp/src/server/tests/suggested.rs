//! The known packs from registries (decision 7, Q2): the packs the suggested scopes' search lists, here on a registry
//! served on the machine, offered by completion with their import, in the scopes' order, and from the cache with the
//! network off.

use std::path::Path;

use compmd_host::registry::{LocalRegistry, ServedPack};

use super::*;

/// The pack `name`, a pack by its keyword, at 1.0.0, exporting `component`.
fn pack(name: &str, component: &str) -> ServedPack {
    let manifest = json!({ "name": name, "version": "1.0.0", "keywords": ["compmd-pack"], "description": format!("The {component}.") });
    let document = format!(
        "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component={component}\n---\nA {component}.\n```\n"
    );
    ServedPack::new(name).version(
        "1.0.0",
        &[
            ("package.json", &manifest.to_string()),
            ("index.comp.md", &document),
        ],
    )
}

/// A client whose CompMD packs come from `registry`, its cache in `dir`, and `settings`.
fn client(dir: &Path, registry: &LocalRegistry, settings: &Value) -> Client {
    let params = json!({ "capabilities": {}, "initializationOptions": settings });
    Client::start_own(params, dir.join("cache"), registry.url().to_owned()).0
}

/// Opens a document in `dir` that is writing a component's name, and gives its URI.
fn open(client: &Client, dir: &Path) -> String {
    let uri = uri_of(&dir.join("a.comp.md"));
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": "::: no\n" } }));
    uri
}

/// The completions at the name, once they offer `wanted`: the suggested packs come after the document opened.
fn offered(client: &Client, uri: &str, wanted: &str) -> Vec<Value> {
    let at = json!({ "textDocument": { "uri": uri }, "position": { "line": 0, "character": 6 } });
    for id in 3.. {
        let items = client
            .ask(id, "textDocument/completion", at.clone())
            .result
            .and_then(|items| items.as_array().cloned())
            .unwrap_or_default();
        if items.iter().any(|item| item["label"] == wanted) {
            return items;
        }
    }
    unreachable!("asked until offered")
}

fn labels(items: &[Value]) -> Vec<&str> {
    items
        .iter()
        .filter_map(|item| item["label"].as_str())
        .collect()
}

#[test]
fn compmds_own_packs_are_offered_with_their_import_and_no_configuration() {
    let dir = folder("suggested-own");
    let registry = LocalRegistry::serve(&[pack("@comp-markdown/badge", "badge")]).expect("served");
    let client = client(&dir, &registry, &json!({}));
    let uri = open(&client, &dir);
    let items = offered(&client, &uri, "badge");
    let badge = items
        .iter()
        .find(|item| item["label"] == "badge")
        .expect("`badge`");
    assert_eq!(badge["detail"], "imports it from `@comp-markdown/badge@1`");
    assert_eq!(
        badge["documentation"]["value"],
        "**`badge`**, from `@comp-markdown/badge@1`\n\nThe badge."
    );
    assert!(
        badge["additionalTextEdits"][0]["newText"]
            .as_str()
            .is_some_and(|text| text.contains("- from: \"@comp-markdown/badge@1\"")),
        "{badge}"
    );
    let other = uri_of(&dir.join("b.comp.md"));
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": other, "languageId": "compmd", "version": 1, "text": "::: no\n" } }));
    offered(&client, &other, "badge");
    let searches = registry
        .asked()
        .into_iter()
        .filter(|path| {
            path == "/-/v1/search?text=scope:comp-markdown%20keywords:compmd-pack&size=250"
        })
        .count();
    assert_eq!(searches, 1, "once a session, however many documents");
    client.stop();
}

#[test]
fn the_scopes_come_compmds_then_the_projects_then_the_writers() {
    let dir = folder("suggested-order");
    let served = [
        pack("@comp-markdown/badge", "badge"),
        pack("@acme/map", "map"),
        pack("@zine/zebra", "zebra"),
    ];
    let registry = LocalRegistry::serve(&served).expect("served");
    let config =
        json!({ "registries": { "@acme": registry.url() }, "editor": { "suggest": ["@acme"] } });
    fs::write(dir.join("compmd.config.json"), config.to_string()).expect("written");
    let client = client(
        &dir,
        &registry,
        &json!({ "packs": { "suggest": ["@zine"] } }),
    );
    let uri = open(&client, &dir);
    let items = offered(&client, &uri, "zebra");
    let suggested: Vec<&str> = labels(&items)
        .into_iter()
        .filter(|label| ["badge", "map", "zebra"].contains(label))
        .collect();
    assert_eq!(suggested, ["badge", "map", "zebra"]);
    client.stop();
}

#[test]
fn with_the_network_off_the_packs_kept_are_offered_and_no_registry_is_asked() {
    let dir = folder("suggested-offline");
    let registry = LocalRegistry::serve(&[pack("@comp-markdown/badge", "badge")]).expect("served");
    let online = client(&dir, &registry, &json!({}));
    let uri = open(&online, &dir);
    offered(&online, &uri, "badge");
    online.stop();
    let asked = registry.asked().len();
    let offline = client(&dir, &registry, &json!({ "packs": { "network": false } }));
    let uri = open(&offline, &dir);
    offered(&offline, &uri, "badge");
    assert_eq!(registry.asked().len(), asked, "from the cache alone");
    offline.stop();
}
