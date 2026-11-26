//! CompMD's own packs with no configuration (decision 4, Q4): resolved from npm, here a registry served on the
//! machine, with a warning on each import the site can't load, its quick fix, and the question.

use std::path::Path;

use compmd_host::registry::{LocalRegistry, ServedPack};

use super::*;

/// A document importing `@comp-markdown/notes@1`'s `note`, and `absent`, which the pack doesn't export (F021 once it
/// loads).
const DOCUMENT: &str = "---\nmd:\n  import:\n    - from: \"@comp-markdown/notes@1\"\n      names: [note, absent]\n---\n\n:: note[Hi]\n";

/// The warning on its import.
const WARNING: &str = "your site can't load `@comp-markdown/notes`: install it, or add a registry";

fn notes() -> ServedPack {
    let package =
        r#"{"name": "@comp-markdown/notes", "version": "1.0.0", "compmd": "index.comp.md"}"#;
    let document = "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=note\n---\nA note.\n```\n";
    ServedPack::new("@comp-markdown/notes").version(
        "1.0.0",
        &[("package.json", package), ("index.comp.md", document)],
    )
}

/// A client that applies the server's edits, CompMD's own packs from `registry`, its cache in `dir`.
fn client(dir: &Path, registry: &LocalRegistry) -> Client {
    let params = json!({ "capabilities": { "workspace": { "applyEdit": true } } });
    Client::start_own(params, dir.join("cache"), registry.url().to_owned()).0
}

/// Opens the document `name` in `dir`, and gives its URI.
fn open(client: &Client, dir: &Path, name: &str) -> String {
    let uri = uri_of(&dir.join(name));
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": DOCUMENT } }));
    uri
}

/// The next diagnostics published, each its code or, without one, its message.
fn published(client: &Client) -> Vec<Value> {
    client.next_of(PublishDiagnostics::METHOD)["diagnostics"]
        .as_array()
        .expect("a list")
        .clone()
}

fn codes(diagnostics: &[Value]) -> Vec<&str> {
    diagnostics
        .iter()
        .map(|diagnostic| {
            diagnostic["code"]
                .as_str()
                .or_else(|| diagnostic["message"].as_str())
                .unwrap_or_default()
        })
        .collect()
}

#[test]
fn with_no_configuration_compmds_pack_loads_and_its_import_says_what_the_site_needs() {
    let dir = folder("own-unconfigured");
    let registry = LocalRegistry::serve(&[notes()]).expect("served");
    let client = client(&dir, &registry);
    let uri = open(&client, &dir, "a.comp.md");
    assert_eq!(codes(&published(&client)), Vec::<&str>::new(), "pending");
    let question = client.next_request();
    assert_eq!(question.method, "window/showMessageRequest");
    let message = question.params["message"].as_str().unwrap_or_default();
    assert!(
        message.starts_with("Your site loads packs from npm at runtime only if compmd.config.json names it. Add npm for `@comp-markdown`?"),
        "{message}"
    );
    assert!(
        message.contains("`fetch: \"direct\"`"),
        "no proxy: {message}"
    );
    let titles: Vec<&str> = question.params["actions"]
        .as_array()
        .expect("actions")
        .iter()
        .filter_map(|action| action["title"].as_str())
        .collect();
    assert_eq!(titles, ["Add", "Not now"]);
    let loaded = published(&client);
    assert_eq!(
        codes(&loaded),
        ["F021", WARNING],
        "loaded, and a warning, not F026"
    );
    let warning = &loaded[1];
    assert_eq!(warning["severity"], 2);
    assert_eq!(
        warning["range"]["start"],
        json!({ "line": 3, "character": 12 })
    );

    client.send(Message::Response(Response::new_ok(
        question.id,
        json!({ "title": "Add" }),
    )));
    let applied = client.next_request();
    assert_eq!(applied.method, "workspace/applyEdit");
    let operations = &applied.params["edit"]["documentChanges"];
    assert_eq!(operations[0]["kind"], "create");
    assert_eq!(
        operations[0]["uri"],
        uri_of(&dir.join("compmd.config.json"))
    );
    let written = operations[1]["edits"][0]["newText"]
        .as_str()
        .unwrap_or_default();
    let config: Value = serde_json::from_str(written).expect("JSON");
    assert_eq!(
        config,
        json!({ "registries": { "@comp-markdown": registry.url() }, "fetch": "direct" })
    );

    let at = &warning["range"];
    let actions = client.ask(3, "textDocument/codeAction", json!({ "textDocument": { "uri": uri }, "range": at, "context": { "diagnostics": [], "only": ["quickfix"] } }));
    let actions = actions.result.expect("actions");
    let fix = actions
        .as_array()
        .expect("a list")
        .iter()
        .find(|action| action["title"] == "Add npm for `@comp-markdown` to compmd.config.json")
        .expect("the fix");
    assert_eq!(fix["edit"], applied.params["edit"]);
    assert_eq!(fix["diagnostics"][0]["message"], WARNING);
    client.stop();
}

#[test]
fn not_now_keeps_the_warning_and_a_project_is_asked_once() {
    let dir = folder("own-not-now");
    let registry = LocalRegistry::serve(&[notes()]).expect("served");
    let client = client(&dir, &registry);
    open(&client, &dir, "a.comp.md");
    let question = client.next_request();
    client.send(Message::Response(Response::new_ok(
        question.id,
        json!({ "title": "Not now" }),
    )));
    let uri = open(&client, &dir, "b.comp.md");
    loop {
        let published = client.next_unasked(PublishDiagnostics::METHOD);
        let diagnostics = published["diagnostics"].as_array();
        if published["uri"] == uri
            && diagnostics.is_some_and(|found| codes(found).contains(&WARNING))
        {
            break;
        }
    }
    let at =
        json!({ "start": { "line": 3, "character": 12 }, "end": { "line": 3, "character": 12 } });
    let actions = client.ask(
        3,
        "textDocument/codeAction",
        json!({ "textDocument": { "uri": uri }, "range": at, "context": { "diagnostics": [] } }),
    );
    let titles: Vec<String> = actions
        .result
        .expect("actions")
        .as_array()
        .expect("a list")
        .iter()
        .filter_map(|action| action["title"].as_str().map(str::to_owned))
        .collect();
    assert!(
        titles.contains(&"Add npm for `@comp-markdown` to compmd.config.json".to_owned()),
        "{titles:?}"
    );
    client.stop();
}

#[test]
fn a_site_that_names_their_registry_loads_them_with_no_warning() {
    let dir = folder("own-configured");
    let registry = LocalRegistry::serve(&[notes()]).expect("served");
    let config = json!({ "registries": { "@comp-markdown": registry.url() } });
    fs::write(dir.join("compmd.config.json"), config.to_string()).expect("written");
    let client = client(&dir, &registry);
    open(&client, &dir, "a.comp.md");
    assert_eq!(codes(&published(&client)), Vec::<&str>::new(), "pending");
    assert_eq!(codes(&published(&client)), ["F021"]);
    client.stop();
}
