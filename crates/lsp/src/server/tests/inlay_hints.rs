//! Values as inlay hints through the protocol ([a document drawn in the editor], decision 9): the capability,
//! `textDocument/inlayHint`, `compmd.render.values` applied live, and `workspace/inlayHint/refresh`, sent only to a
//! client that takes it, when the setting or the values change.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use super::*;

const NOTES: &str = "---\ntitle: Harbour notes\n---\n\n# {{md.doc.title}}\n";

/// A client's capabilities, taking inlay hint refreshes when `refreshes`.
fn refreshing(refreshes: bool) -> Value {
    json!({ "workspace": { "inlayHint": { "refreshSupport": refreshes } } })
}

/// A client with `capabilities` and a document `a.comp.md` open with `text`, once it is processed.
fn opened(name: &str, capabilities: &Value, text: &str) -> (Client, Value, String) {
    let uri = uri_of(&folder(name).join("a.comp.md"));
    let (client, initialized) = Client::start(json!({ "capabilities": capabilities }));
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": text } }));
    let _ = client.diagnostics();
    (client, initialized, uri)
}

impl Client {
    /// The labels of the inlay hints of `uri`'s first lines as request `id`, `None` for no answer; the server sending a
    /// request first fails the test.
    fn labels_unrefreshed(&self, id: i32, uri: &str) -> Option<Vec<Value>> {
        let range = json!({ "start": { "line": 0, "character": 0 }, "end": { "line": 20, "character": 0 } });
        let params = json!({ "textDocument": { "uri": uri }, "range": range });
        let hints = self.answer_unrefreshed(id, "textDocument/inlayHint", params);
        let hints = hints.as_array()?.iter();
        Some(hints.map(|hint| hint["label"].clone()).collect())
    }

    /// The answer to the request `method` with `params`, as request `id`; the server sending a request first fails
    /// the test.
    fn answer_unrefreshed(&self, id: i32, method: &str, params: Value) -> Value {
        self.send(Message::Request(Request::new(
            id.into(),
            method.into(),
            params,
        )));
        loop {
            match self.next() {
                Message::Response(response) => return response.result.expect("an answer"),
                Message::Request(request) => panic!("the server asked {}", request.method),
                Message::Notification(_) => {}
            }
        }
    }

    /// Answers the server's next request, which is a refresh of the inlay hints.
    fn hints_refreshed(&self) {
        let refresh = self.next_request();
        assert_eq!(refresh.method, "workspace/inlayHint/refresh");
        self.send(Message::Response(Response::new_ok(refresh.id, json!(null))));
    }

    /// Sends `text` in place of the columns `from` to `to` of the line `line` of `uri`, at `version`.
    fn edit(&self, uri: &str, version: i32, (line, from, to): (u32, u32, u32), text: &str) {
        let range = json!({ "start": { "line": line, "character": from }, "end": { "line": line, "character": to } });
        self.notify(
            "textDocument/didChange",
            json!({ "textDocument": { "uri": uri, "version": version }, "contentChanges": [{ "range": range, "text": text }] }),
        );
    }

    /// Sends the `compmd` settings `settings`.
    fn configure(&self, settings: &Value) {
        self.notify(
            "workspace/didChangeConfiguration",
            json!({ "settings": { "compmd": settings } }),
        );
    }
}

#[test]
fn the_server_offers_inlay_hints_and_answers_with_the_values() {
    let (client, initialized, uri) = opened("values", &json!({}), NOTES);
    assert_eq!(
        initialized["capabilities"]["inlayHintProvider"],
        json!(true)
    );
    assert_eq!(
        client.labels_unrefreshed(3, &uri),
        Some(vec![json!("= Harbour notes")])
    );
    client.stop();
}

#[test]
fn the_setting_turns_them_off_live_and_asks_the_client_to_ask_again() {
    let (client, _, uri) = opened("values-setting", &refreshing(true), NOTES);
    client.configure(&json!({ "render": { "values": false } }));
    client.hints_refreshed();
    assert_eq!(client.labels_unrefreshed(3, &uri), None, "none when off");
    client.configure(&json!({ "render": { "values": false, "glyphs": "ascii" } }));
    assert_eq!(
        client.labels_unrefreshed(4, &uri),
        None,
        "another drawing setting asks for no refresh of the hints"
    );
    client.edit(&uri, 2, (1, 7, 14), "Quay");
    // Asked of the worker, so answered once it took the edit: values that change while they are off ask for no
    // refresh, which the next answer would meet.
    let folding = client.answer_unrefreshed(
        5,
        "textDocument/foldingRange",
        json!({ "textDocument": { "uri": uri } }),
    );
    assert!(folding.is_array());
    client.configure(&json!({ "render": { "values": true } }));
    client.hints_refreshed();
    assert_eq!(
        client.labels_unrefreshed(6, &uri),
        Some(vec![json!("= Quay notes")])
    );
    client.stop();
}

#[test]
fn a_change_of_the_values_asks_the_client_to_ask_again_and_a_move_doesnt() {
    let (client, _, uri) = opened("values-refresh", &refreshing(true), NOTES);
    client.edit(&uri, 2, (3, 0, 0), "Intro.\n\n");
    assert_eq!(
        client.labels_unrefreshed(3, &uri),
        Some(vec![json!("= Harbour notes")]),
        "the hint moved with its text: no refresh"
    );
    client.edit(&uri, 3, (1, 7, 14), "Quay");
    client.hints_refreshed();
    assert_eq!(
        client.labels_unrefreshed(4, &uri),
        Some(vec![json!("= Quay notes")])
    );
    client.stop();
}

#[test]
fn a_client_that_takes_no_refreshes_is_sent_none() {
    let (client, _, uri) = opened("values-no-refresh", &refreshing(false), NOTES);
    client.edit(&uri, 2, (1, 7, 14), "Quay");
    assert_eq!(
        client.labels_unrefreshed(3, &uri),
        Some(vec![json!("= Quay notes")])
    );
    client.configure(&json!({ "render": { "values": false } }));
    assert_eq!(client.labels_unrefreshed(4, &uri), None);
    client.stop();
}
