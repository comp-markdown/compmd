//! The preview, as an editor opens it (decision 13): `compmd.preview` on a document opens its page in the browser the
//! settings name, and the page gets what the server's processor read, and each change with the line being edited.

use crate::preview::testing::{get, listen, next_event, page_of};

use super::*;

/// A document that reads `tide.csv`.
const DOCUMENT: &str =
    "---\nmd:\n  data:\n    - from: ./tide.csv\n      as: tide\n---\n\n# Harbour notes\n";

/// Asks for the preview of the document `uri`, as request `id`.
fn preview(client: &Client, id: i32, uri: &str) -> Response {
    client.ask(
        id,
        "workspace/executeCommand",
        json!({ "command": "compmd.preview", "arguments": [uri] }),
    )
}

#[test]
fn the_command_opens_the_documents_preview_with_what_its_processor_read() {
    let dir = folder("preview");
    fs::write(dir.join("tide.csv"), "level\n2\n").expect("written");
    let uri = uri_of(&dir.join("notes.comp.md"));
    let settings = json!({ "capabilities": {}, "initializationOptions": { "preview": { "browser": "firefox" } } });
    let (client, initialized) = Client::start(settings);
    assert_eq!(
        initialized["capabilities"]["executeCommandProvider"]["commands"],
        json!(["compmd.preview"])
    );
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": DOCUMENT } }));
    let url = preview(&client, 3, &uri)
        .result
        .expect("the page's address");
    let url = url.as_str().expect("an address").to_owned();
    assert_eq!(
        *client.opened.lock().expect("recorded"),
        [(url.clone(), Some("firefox".to_owned()))]
    );
    let again = preview(&client, 4, &uri).result.expect("the address");
    assert_eq!(again, url.as_str(), "the same preview, opened again");
    let (port, token) = page_of(&url);
    let (status, _, body) = get(port, &format!("{token}request.json"));
    assert_eq!(status, 200);
    let request: Value = serde_json::from_str(&body).expect("JSON");
    assert_eq!(request["path"], "notes.comp.md");
    assert_eq!(request["files"]["tide.csv"], "level\n2\n");
    let mut events = listen(port, &token);
    let (event, change) = next_event(&mut events);
    assert_eq!(
        (event.as_str(), &change["text"]),
        ("change", &json!(DOCUMENT))
    );
    let end = json!({ "line": 8, "character": 0 });
    client.notify("textDocument/didChange", json!({ "textDocument": { "uri": uri, "version": 2 }, "contentChanges": [{ "range": { "start": end, "end": end }, "text": "\nThe tide is in.\n" }] }));
    let (_, change) = next_event(&mut events);
    assert_eq!(change["line"], 11);
    assert!(
        change["text"]
            .as_str()
            .is_some_and(|text| text.ends_with("The tide is in.\n"))
    );
    client.stop();
}

#[test]
fn a_command_this_server_doesnt_run_or_a_document_not_open_is_an_error() {
    let (client, _) = Client::start(json!({ "capabilities": {} }));
    let unknown = client.ask(
        3,
        "workspace/executeCommand",
        json!({ "command": "other", "arguments": [] }),
    );
    assert!(
        unknown
            .error
            .is_some_and(|error| error.message.contains("isn't a command"))
    );
    let closed = preview(&client, 4, "file:///nowhere/a.comp.md");
    assert!(
        closed
            .error
            .is_some_and(|error| error.message.contains("open in the editor"))
    );
    let malformed = client.ask(5, "workspace/executeCommand", json!({ "arguments": 1 }));
    assert!(malformed.error.is_some());
    assert_eq!(*client.opened.lock().expect("recorded"), []);
    client.stop();
}
