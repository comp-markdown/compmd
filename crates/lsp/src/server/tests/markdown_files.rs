//! `.md` files through the protocol ([a document drawn in the editor], decision 13): read as the spec reads them, a
//! plain one with Markdown's features only, and `compmd.markdown.enabled` switching them off live.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use super::*;

/// A plain Markdown file: `:::` and `{{…}}` are text, so nothing in it is CompMD's to report.
const PLAIN: &str = "# Notes\n\n::: gird\nText {{nobody}}.\n:::\n";

/// A client with the document `name` in a folder of its own open with `text`, and its first diagnostics' codes.
fn opened(folder_name: &str, name: &str, text: &str) -> (Client, String, Vec<String>) {
    let uri = uri_of(&folder(folder_name).join(name));
    let (client, _) = Client::start(json!({ "capabilities": {} }));
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "markdown", "version": 1, "text": text } }));
    let diagnostics = client.diagnostics();
    (client, uri, diagnostics)
}

/// The hover at line `line`, character `character` of `uri`, as request `id`.
fn hover(client: &Client, id: i32, uri: &str, (line, character): (u32, u32)) -> Value {
    let params = json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": character } });
    client
        .ask(id, "textDocument/hover", params)
        .result
        .unwrap_or_default()
}

#[test]
fn a_plain_markdown_file_has_no_compmd_diagnostics_or_hints() {
    let (client, uri, diagnostics) = opened("markdown-plain", "notes.md", PLAIN);
    assert_eq!(diagnostics, Vec::<String>::new(), "no F002 for `gird`");
    assert_eq!(hover(&client, 3, &uri, (2, 5)), Value::Null);
    let folding = client.ask(
        4,
        "textDocument/foldingRange",
        json!({ "textDocument": { "uri": uri } }),
    );
    assert!(folding.result.is_some_and(|ranges| ranges.is_array()));
    client.stop();
}

#[test]
fn a_md_file_with_an_md_key_is_compmd() {
    let text = format!("---\nmd:\n  version: 1\n---\n{PLAIN}");
    let (client, uri, diagnostics) = opened("markdown-compmd", "notes.md", &text);
    assert!(diagnostics.contains(&"F002".to_owned()), "{diagnostics:?}");
    assert_ne!(hover(&client, 3, &uri, (6, 5)), Value::Null);
    client.stop();
}

#[test]
fn markdown_files_are_switched_off_and_on_live() {
    let text = format!("---\nmd: {{}}\n---\n{PLAIN}");
    let (client, uri, diagnostics) = opened("markdown-switch", "notes.md", &text);
    assert!(diagnostics.contains(&"F002".to_owned()));
    let switched = |enabled: bool| {
        client.notify(
            "workspace/didChangeConfiguration",
            json!({ "settings": { "compmd": { "markdown": { "enabled": enabled } } } }),
        );
        client.diagnostics()
    };
    assert_eq!(switched(false), Vec::<String>::new(), "cleared");
    assert_eq!(hover(&client, 3, &uri, (5, 5)), Value::Null, "not served");
    assert!(switched(true).contains(&"F002".to_owned()), "served again");
    assert_ne!(hover(&client, 4, &uri, (5, 5)), Value::Null);
    client.stop();
}

#[test]
fn a_comp_md_file_is_served_whatever_the_markdown_setting() {
    let uri = uri_of(&folder("markdown-off-compmd").join("a.comp.md"));
    let (client, _) = Client::start(
        json!({ "capabilities": {}, "initializationOptions": { "markdown": { "enabled": false } } }),
    );
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": PLAIN } }));
    assert!(client.diagnostics().contains(&"F002".to_owned()));
    client.stop();
}
