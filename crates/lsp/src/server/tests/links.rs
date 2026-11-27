//! Links and headings through the protocol ([the language server], decision 15): the capabilities, the project
//! index read in the background (workspace symbols), a file changed on the disk read again, go to definition across
//! files, a link's anchors completed, broken links warned about, headings, labels and files renamed, and
//! `compmd.links`' switches turning each off.
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
fn a_broken_link_is_warned_about_until_links_diagnostics_is_off() {
    let dir = folder("broken-links");
    let harbour = dir.join("harbour.md");
    let text = "# Harbour\n\nSee [the tides](gone.md) and [the tables][tables].\n";
    let (client, _) = Client::start(
        json!({ "capabilities": {}, "workspaceFolders": [{ "uri": uri_of(&dir), "name": "notes" }] }),
    );
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri_of(&harbour), "languageId": "markdown", "version": 1, "text": text } }));
    assert_eq!(client.diagnostics(), ["broken-link", "broken-link"]);
    client.notify(
        "workspace/didChangeConfiguration",
        json!({ "settings": { "compmd": { "links": { "diagnostics": false } } } }),
    );
    assert_eq!(
        client.diagnostics(),
        Vec::<String>::new(),
        "switched off, posted again"
    );
    client.stop();
}

#[test]
fn a_markdown_files_wiki_links_are_followed_once_markdown_wiki_links_is_on_until_links_wiki_is_off()
{
    let dir = folder("wiki-links");
    let harbour = dir.join("harbour.md");
    let text = "# Harbour\n\n[[Gone]] and [[harbour#Harbour]]\n";
    let (client, _) = Client::start(
        json!({ "capabilities": {}, "workspaceFolders": [{ "uri": uri_of(&dir), "name": "notes" }] }),
    );
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri_of(&harbour), "languageId": "markdown", "version": 1, "text": text } }));
    assert_eq!(
        client.diagnostics(),
        ["broken-link", "broken-link"],
        "off by default: references in brackets, their labels undefined"
    );
    assert_eq!(
        symbols_until(&client, 100, "Harbour", &["Harbour"]),
        ["Harbour"],
        "indexed"
    );
    let switch = |settings: Value| {
        client.notify(
            "workspace/didChangeConfiguration",
            json!({ "settings": { "compmd": settings } }),
        );
    };
    switch(json!({ "markdown": { "wikiLinks": true } }));
    assert_eq!(client.diagnostics(), ["broken-link"], "`Gone` isn't there");
    let definition = client.ask(
        3,
        "textDocument/definition",
        json!({ "textDocument": { "uri": uri_of(&harbour) }, "position": { "line": 2, "character": 16 } }),
    );
    assert_eq!(
        definition.result.expect("an answer")["range"]["start"]["line"],
        0,
        "its own heading"
    );
    switch(json!({ "markdown": { "wikiLinks": true }, "links": { "wiki": false } }));
    assert_eq!(client.diagnostics(), Vec::<String>::new(), "links.wiki off");
    client.stop();
}

#[test]
fn headings_labels_and_files_are_renamed_until_links_rename_is_off() {
    let dir = folder("rename");
    fs::write(
        dir.join("log.md"),
        "# Log\n\n[Harbour](harbour.md#harbour)\n",
    )
    .expect("written");
    let harbour = dir.join("harbour.md");
    let text = "# Harbour\n\n[top](#harbour)\n";
    let (client, initialized) = Client::start(
        json!({ "capabilities": {}, "workspaceFolders": [{ "uri": uri_of(&dir), "name": "notes" }] }),
    );
    let capabilities = &initialized["capabilities"];
    assert_eq!(
        capabilities["renameProvider"]["prepareProvider"],
        json!(true)
    );
    assert_eq!(
        capabilities["workspace"]["fileOperations"]["willRename"]["filters"][0]["pattern"]["glob"],
        json!("**/*")
    );
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri_of(&harbour), "languageId": "markdown", "version": 1, "text": text } }));
    let _ = client.diagnostics();
    assert_eq!(
        symbols_until(&client, 100, "Log", &["Log"]),
        ["Log"],
        "indexed"
    );
    let at = json!({ "textDocument": { "uri": uri_of(&harbour) }, "position": { "line": 0, "character": 3 } });
    let prepared = client.ask(3, "textDocument/prepareRename", at.clone());
    assert_eq!(prepared.result.expect("a range")["end"]["character"], 9);
    let rename = |id: i32| {
        let mut params = at.clone();
        params["newName"] = json!("Harbour log");
        client
            .ask(id, "textDocument/rename", params)
            .result
            .expect("an answer")
    };
    let edit = rename(4);
    let changes = edit["changes"].as_object().expect("changes");
    assert_eq!(
        changes.len(),
        2,
        "the document and the file linking to it: {edit}"
    );
    let renamed_files = |id: i32| {
        let files = json!({ "files": [{ "oldUri": uri_of(&dir.join("log.md")), "newUri": uri_of(&dir.join("notes/log.md")) }] });
        client
            .ask(id, "workspace/willRenameFiles", files)
            .result
            .expect("an answer")
    };
    let moved = renamed_files(5);
    assert_eq!(
        moved["changes"][uri_of(&dir.join("log.md"))][0]["newText"],
        json!("../harbour.md"),
        "{moved}"
    );
    client.notify(
        "workspace/didChangeConfiguration",
        json!({ "settings": { "compmd": { "links": { "rename": false } } } }),
    );
    assert_eq!(rename(6), Value::Null, "links.rename off");
    assert_eq!(renamed_files(7), Value::Null);
    client.stop();
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

    let anchors = |id: i32| -> Vec<String> {
        let at = json!({ "textDocument": { "uri": uri_of(&harbour) }, "position": { "line": 2, "character": 35 } });
        let response = client.ask(id, "textDocument/completion", at);
        response
            .result
            .and_then(|result| result.as_array().cloned())
            .unwrap_or_default()
            .iter()
            .map(|item| item["label"].as_str().unwrap_or_default().to_owned())
            .collect()
    };
    assert_eq!(
        anchors(110),
        ["tides", "low-water"],
        "a link's anchors completed"
    );
    client.notify(
        "workspace/didChangeConfiguration",
        json!({ "settings": { "compmd": { "links": { "completion": false } } } }),
    );
    assert_eq!(anchors(111), Vec::<String>::new(), "links.completion off");

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
