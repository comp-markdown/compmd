//! The drawing plan through the protocol ([a document drawn in the editor], decisions 3, 4 and 6): the capabilities
//! both sides declare, `compmd/drawing`, and `compmd/drawing/refresh`, sent only to a client that takes it.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use super::*;
use crate::features::drawing::METHOD;

/// A client's capabilities with the drawing declared: every primitive, and refreshes when `refreshes`.
fn drawing(refreshes: bool) -> Value {
    let primitives = ["hide", "replace", "insert", "line", "prefix", "style"];
    json!({ "experimental": { "compmd": { "drawing": { "primitives": primitives, "refreshSupport": refreshes } } } })
}

/// The plan request for the first lines of `uri`.
fn asking_plan(uri: &str) -> Value {
    let range =
        json!({ "start": { "line": 0, "character": 0 }, "end": { "line": 20, "character": 0 } });
    json!({ "textDocument": { "uri": uri, "version": 1 }, "range": range })
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
    /// Sends `change` to the line `line` of `uri` at `version`.
    fn change_line(&self, uri: &str, version: i32, line: u32, text: &str) {
        let at = json!({ "line": line, "character": 0 });
        let change = json!({ "range": { "start": at, "end": at }, "text": text });
        self.notify(
            "textDocument/didChange",
            json!({ "textDocument": { "uri": uri, "version": version }, "contentChanges": [change] }),
        );
    }

    /// The plan of `uri` as request `id`; the server sending a request first fails the test.
    fn plan_unrefreshed(&self, id: i32, uri: &str) -> Value {
        self.send(Message::Request(Request::new(
            id.into(),
            METHOD.into(),
            asking_plan(uri),
        )));
        loop {
            match self.next() {
                Message::Response(response) => return response.result.expect("a plan"),
                Message::Request(request) => panic!("the server asked {}", request.method),
                Message::Notification(_) => {}
            }
        }
    }

    /// Answers the server's next request, which is a refresh of the drawing.
    fn refreshed(&self) {
        let refresh = self.next_request();
        assert_eq!(refresh.method, "compmd/drawing/refresh");
        assert_eq!(refresh.params, Value::Null, "no params");
        self.send(Message::Response(Response::new_ok(refresh.id, json!(null))));
    }
}

#[test]
fn the_server_draws_only_for_a_client_that_declared_the_drawing() {
    let (client, initialized) = Client::start(json!({ "capabilities": drawing(false) }));
    assert_eq!(
        initialized["capabilities"]["experimental"]["compmd"]["drawing"],
        json!({})
    );
    assert_eq!(
        initialized["capabilities"]["experimental"]["compmdBlocks"],
        json!(true)
    );
    client.stop();
    let (client, initialized, uri) = opened("undeclared", &json!({}), "# Notes\n");
    assert_eq!(
        initialized["capabilities"]["experimental"]["compmd"],
        Value::Null
    );
    let plan = client.plan_unrefreshed(3, &uri);
    assert_eq!(
        plan,
        json!({ "version": 1, "marks": [] }),
        "nothing it draws"
    );
    client.notify(
        "workspace/didChangeConfiguration",
        json!({ "settings": { "compmd": { "render": { "glyphs": "ascii" } } } }),
    );
    assert_eq!(
        client.plan_unrefreshed(4, &uri)["marks"],
        json!([]),
        "and no refresh"
    );
    client.stop();
}

#[test]
fn the_plan_is_asked_for_by_range_and_follows_the_settings_live() {
    let (client, _, uri) = opened("drawing", &drawing(true), "# Notes\n");
    let plan = client.plan_unrefreshed(3, &uri);
    assert_eq!(plan["version"], json!(1));
    assert_eq!(
        plan["marks"][0],
        json!({ "kind": "replace", "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 1 } }, "text": "◉", "style": "compmd.heading.1", "reveal": { "first": 0, "last": 0 } })
    );
    client.notify(
        "workspace/didChangeConfiguration",
        json!({ "settings": { "compmd": { "render": { "glyphs": "ascii" } } } }),
    );
    client.refreshed();
    assert_eq!(
        client.plan_unrefreshed(4, &uri)["marks"][0]["text"],
        json!("#")
    );
    client.notify(
        "workspace/didChangeConfiguration",
        json!({ "settings": { "compmd": { "render": { "glyphs": "ascii" }, "trace": "off" } } }),
    );
    assert_eq!(
        client.plan_unrefreshed(5, &uri)["marks"][0]["text"],
        json!("#"),
        "the same drawing settings ask for no refresh"
    );
    let wrong = client.ask(6, METHOD, json!({ "textDocument": { "uri": uri } }));
    assert_eq!(
        wrong.error.map(|error| error.code),
        Some(ErrorCode::InvalidParams as i32)
    );
    let elsewhere = client.ask(7, METHOD, asking_plan("file:///nowhere.comp.md"));
    assert_eq!(
        elsewhere.result,
        Some(Value::Null),
        "a document that isn't open"
    );
    client.stop();
}

#[test]
fn a_change_that_moves_marks_beyond_its_lines_asks_the_client_to_ask_again() {
    let text = "::: box\nIn.\n::: /box\n";
    let (client, _, uri) = opened("refresh", &drawing(true), text);
    client.change_line(&uri, 2, 1, "More.\n");
    assert_eq!(
        client.plan_unrefreshed(3, &uri)["version"],
        json!(2),
        "a line in the box moves nothing else"
    );
    client.change_line(&uri, 3, 0, "```\n");
    client.refreshed();
    assert_eq!(client.plan_unrefreshed(4, &uri)["version"], json!(3));
    client.stop();
}

#[test]
fn a_client_that_takes_no_refreshes_is_sent_none() {
    let (client, _, uri) = opened("no-refresh", &drawing(false), "::: box\nIn.\n::: /box\n");
    client.change_line(&uri, 2, 0, "```\n");
    assert_eq!(client.plan_unrefreshed(3, &uri)["version"], json!(2));
    client.stop();
}
