//! Links and headings through the protocol ([the language server], decision 15): the capabilities, the project
//! index read in the background (workspace symbols), a file changed on the disk read again, go to definition across
//! files, and `compmd.links.enabled` switching them off.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use std::time::{Duration, Instant};

use super::*;

/// The names of the headings `workspace/symbol` gives for `query`, as request `id`.
fn symbols(client: &Client, id: i32, query: &str) -> Vec<String> {
    let response = client.ask(id, "workspace/symbol", json!({ "query": query }));
    response
        .result
        .and_then(|result| result.as_array().cloned())
        .unwrap_or_default()
        .iter()
        .map(|symbol| symbol["name"].as_str().unwrap_or_default().to_owned())
        .collect()
}

/// `symbols` asked again until it gives `expected`, or 20 seconds pass: the index reads in the background.
fn symbols_until(client: &Client, first_id: i32, query: &str, expected: &[&str]) -> Vec<String> {
    let started = Instant::now();
    let mut id = first_id;
    loop {
        let found = symbols(client, id, query);
        if found == expected || started.elapsed() > Duration::from_secs(20) {
            return found;
        }
        id += 1;
        thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn the_index_answers_workspace_symbols_and_definitions_across_files() {
    let dir = folder("links");
    fs::write(dir.join("tides.md"), "# Tides\n\n## High water\n").expect("written");
    let harbour = dir.join("harbour.md");
    let text = "# Harbour\n\nSee [high water](tides.md#high-water).\n";
    let (client, initialized) = Client::start(
        json!({ "capabilities": {}, "workspaceFolders": [{ "uri": uri_of(&dir), "name": "notes" }] }),
    );
    assert_eq!(
        initialized["capabilities"]["referencesProvider"],
        json!(true)
    );
    assert_eq!(
        initialized["capabilities"]["workspaceSymbolProvider"],
        json!(true)
    );
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri_of(&harbour), "languageId": "markdown", "version": 1, "text": text } }));
    let _ = client.diagnostics();
    assert_eq!(
        symbols_until(&client, 3, "water", &["High water"]),
        ["High water"]
    );
    let definition = client.ask(
        100,
        "textDocument/definition",
        json!({ "textDocument": { "uri": uri_of(&harbour) }, "position": { "line": 2, "character": 6 } }),
    );
    let result = definition.result.expect("an answer");
    assert!(
        result["uri"]
            .as_str()
            .is_some_and(|uri| uri.ends_with("tides.md")),
        "{result}"
    );
    assert_eq!(result["range"]["start"]["line"], 2);

    fs::write(dir.join("tides.md"), "# Tides\n\n## Low water\n").expect("written");
    client.notify(
        "workspace/didChangeWatchedFiles",
        json!({ "changes": [{ "uri": uri_of(&dir.join("tides.md")), "type": 2 }] }),
    );
    let _ = client.diagnostics();
    assert_eq!(symbols(&client, 101, "water"), ["Low water"], "read again");

    client.notify(
        "workspace/didChangeConfiguration",
        json!({ "settings": { "compmd": { "links": { "enabled": false } } } }),
    );
    assert_eq!(
        symbols(&client, 102, ""),
        Vec::<String>::new(),
        "switched off"
    );
    let off = client.ask(
        103,
        "textDocument/definition",
        json!({ "textDocument": { "uri": uri_of(&harbour) }, "position": { "line": 2, "character": 6 } }),
    );
    assert_eq!(off.result, Some(Value::Null));
    let references = client.ask(
        104,
        "textDocument/references",
        json!({ "textDocument": { "uri": uri_of(&harbour) }, "position": { "line": 0, "character": 2 }, "context": { "includeDeclaration": true } }),
    );
    assert_eq!(references.result, Some(json!([])));
    client.stop();
}
