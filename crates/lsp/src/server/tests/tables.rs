//! Tables aligned through the protocol ([a document drawn in the editor], decisions 6 and 7): the formatting
//! capabilities, formatting, range formatting and on-type formatting, and the settings that switch them off live.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use super::*;

const TABLE: &str = "| a | bb |\n|-|-|\n";

/// The edits' new texts in `response`, or `None` when it answered with nothing.
fn new_texts(response: Response) -> Option<Vec<String>> {
    let result = response.result.expect("an answer");
    let edits = result.as_array()?;
    Some(
        edits
            .iter()
            .map(|edit| edit["newText"].as_str().expect("a text").to_owned())
            .collect(),
    )
}

#[test]
fn formatting_and_on_type_formatting_align_tables_until_the_settings_switch_them_off() {
    let uri = uri_of(&folder("tables").join("a.comp.md"));
    let (client, initialized) = Client::start(json!({ "capabilities": {} }));
    let capabilities = &initialized["capabilities"];
    assert_eq!(capabilities["documentFormattingProvider"], json!(true));
    assert_eq!(capabilities["documentRangeFormattingProvider"], json!(true));
    assert_eq!(
        capabilities["documentOnTypeFormattingProvider"],
        json!({ "firstTriggerCharacter": "|", "moreTriggerCharacter": ["\n"] })
    );
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": TABLE } }));
    let _ = client.diagnostics();
    let document = json!({ "uri": uri });
    let options = json!({ "tabSize": 2, "insertSpaces": true });
    let formatting = json!({ "textDocument": document, "options": options });
    let all = ["   ", "  ", " --- | --- "].map(String::from).to_vec();
    assert_eq!(
        new_texts(client.ask(3, "textDocument/formatting", formatting.clone())),
        Some(all.clone())
    );
    let line = |line| json!({ "start": { "line": line, "character": 0 }, "end": { "line": line, "character": 1 } });
    let ranged = json!({ "textDocument": document, "range": line(1), "options": options });
    assert_eq!(
        new_texts(client.ask(4, "textDocument/rangeFormatting", ranged)),
        Some(all.clone()),
        "the table the range meets"
    );
    let typed = json!({ "textDocument": document, "position": { "line": 0, "character": 10 }, "ch": "|", "options": options });
    assert_eq!(
        new_texts(client.ask(5, "textDocument/onTypeFormatting", typed.clone())),
        Some(all)
    );
    client.notify(
        "workspace/didChangeConfiguration",
        json!({ "settings": { "compmd": { "tables": { "alignOnFormat": false } } } }),
    );
    assert_eq!(
        new_texts(client.ask(6, "textDocument/formatting", formatting.clone())),
        None
    );
    assert!(new_texts(client.ask(7, "textDocument/onTypeFormatting", typed.clone())).is_some());
    client.notify(
        "workspace/didChangeConfiguration",
        json!({ "settings": { "compmd": { "tables": { "alignOnType": false } } } }),
    );
    assert_eq!(
        new_texts(client.ask(8, "textDocument/onTypeFormatting", typed)),
        None
    );
    assert!(new_texts(client.ask(9, "textDocument/formatting", formatting)).is_some());
    client.stop();
}
