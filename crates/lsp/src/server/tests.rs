//! The server driven by a client in memory, as an editor drives it.

use std::fs;
use std::thread::JoinHandle;
use std::time::Duration;

use lsp_server::{Request, Response};
use serde_json::{Value, json};

use super::*;

/// A client: its side of the connection, and the server's thread.
struct Client {
    connection: Connection,
    server: JoinHandle<Result<(), ServerError>>,
}

impl Client {
    /// A client initialized with `params`, and the server's answer.
    fn start(params: Value) -> (Self, Value) {
        let (server_side, connection) = Connection::memory();
        let server = thread::spawn(move || serve(&server_side));
        let client = Self { connection, server };
        client.send(Message::Request(Request::new(
            1.into(),
            "initialize".into(),
            params,
        )));
        let Message::Response(Response {
            result: Some(result),
            ..
        }) = client.next()
        else {
            panic!("an answer to initialize");
        };
        client.notify("initialized", json!({}));
        (client, result)
    }

    fn send(&self, message: Message) {
        self.connection.sender.send(message).expect("sent");
    }

    fn notify(&self, method: &str, params: Value) {
        self.send(Message::Notification(Notification::new(
            method.into(),
            params,
        )));
    }

    fn next(&self) -> Message {
        self.connection
            .receiver
            .recv_timeout(Duration::from_secs(30))
            .expect("a message")
    }

    /// The answer to the request `method` with `params`, as request `id`, skipping the notifications before it.
    fn ask(&self, id: i32, method: &str, params: Value) -> Response {
        self.send(Message::Request(Request::new(
            id.into(),
            method.into(),
            params,
        )));
        loop {
            if let Message::Response(response) = self.next() {
                return response;
            }
        }
    }

    /// The next notification of `method`, skipping the others.
    fn next_of(&self, method: &str) -> Value {
        loop {
            if let Message::Notification(notification) = self.next()
                && notification.method == method
            {
                return notification.params;
            }
        }
    }

    /// The next request the server sends.
    fn next_request(&self) -> Request {
        loop {
            if let Message::Request(request) = self.next() {
                return request;
            }
        }
    }

    /// Whether opening `path` is traced in the log.
    fn traces_opening(&self, path: &std::path::Path) -> bool {
        self.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri_of(path), "languageId": "compmd", "version": 1, "text": "# A\n" } }));
        let message = self.next_of(LogMessage::METHOD);
        message["message"]
            .as_str()
            .is_some_and(|message| message.contains("opened"))
    }

    /// The codes of the next diagnostics published.
    fn diagnostics(&self) -> Vec<String> {
        let params = self.next_of(PublishDiagnostics::METHOD);
        params["diagnostics"]
            .as_array()
            .expect("a list")
            .iter()
            .map(|diagnostic| diagnostic["code"].as_str().unwrap_or_default().to_owned())
            .collect()
    }

    /// Shuts the server down, as an editor does, and waits for it.
    fn stop(self) {
        self.send(Message::Request(Request::new(
            2.into(),
            "shutdown".into(),
            json!(null),
        )));
        while !matches!(self.next(), Message::Response(_)) {}
        self.notify("exit", json!(null));
        self.server
            .join()
            .expect("the server's thread")
            .expect("a clean exit");
    }
}

/// A folder for one test's documents.
fn folder(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("compmd-lsp-server-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a folder");
    dir
}

fn uri_of(path: &std::path::Path) -> String {
    url::Url::from_file_path(path)
        .expect("a file URL")
        .to_string()
}

#[test]
fn initialize_answers_with_the_encoding_the_client_offers_and_incremental_sync() {
    let (client, result) = Client::start(
        json!({ "capabilities": { "general": { "positionEncodings": ["utf-8", "utf-32"] } } }),
    );
    assert_eq!(result["capabilities"]["positionEncoding"], "utf-32");
    assert_eq!(result["capabilities"]["textDocumentSync"]["change"], 2);
    assert_eq!(result["serverInfo"]["name"], "compmd-lsp");
    client.stop();
}

#[test]
fn a_document_is_diagnosed_as_it_opens_changes_and_closes() {
    let dir = folder("documents");
    let uri = uri_of(&dir.join("a.comp.md"));
    let params = json!({ "capabilities": {}, "workspaceFolders": [{ "uri": uri_of(&dir), "name": "notes" }], "initializationOptions": { "trace": "messages" } });
    let (client, _) = Client::start(params);
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": ":: nowhere[Here]\n" } }));
    assert!(
        client.next_of(LogMessage::METHOD)["message"]
            .as_str()
            .is_some_and(|message| message.contains("opened")),
        "traced"
    );
    assert_eq!(client.diagnostics(), ["F002"]);
    let change = json!({ "range": { "start": { "line": 0, "character": 3 }, "end": { "line": 0, "character": 10 } }, "text": "box" });
    client.notify(
        "textDocument/didChange",
        json!({ "textDocument": { "uri": uri, "version": 2 }, "contentChanges": [change] }),
    );
    assert_eq!(client.diagnostics(), Vec::<String>::new());
    client.notify(
        "workspace/didChangeConfiguration",
        json!({ "settings": { "compmd": { "trace": "off" } } }),
    );
    client.notify("workspace/didChangeWatchedFiles", json!({ "changes": [] }));
    assert_eq!(
        client.diagnostics(),
        Vec::<String>::new(),
        "processed again"
    );
    client.notify(
        "textDocument/didClose",
        json!({ "textDocument": { "uri": uri } }),
    );
    assert_eq!(client.diagnostics(), Vec::<String>::new(), "cleared");
    client.stop();
}

#[test]
fn the_server_registers_the_files_it_watches_and_pulls_its_settings_where_the_client_allows() {
    let dir = folder("pulled");
    let capabilities = json!({ "workspace": { "configuration": true, "didChangeWatchedFiles": { "dynamicRegistration": true } } });
    let (client, _) = Client::start(json!({ "capabilities": capabilities }));
    let registration = client.next_request();
    assert_eq!(registration.method, "client/registerCapability");
    assert_eq!(
        registration.params["registrations"][0]["method"],
        "workspace/didChangeWatchedFiles"
    );
    assert_eq!(
        registration.params["registrations"][0]["registerOptions"]["watchers"][0]["globPattern"],
        "**/*.md"
    );
    client.send(Message::Response(Response::new_ok(
        registration.id,
        json!(null),
    )));
    let pull = client.next_request();
    assert_eq!(
        (pull.method.as_str(), &pull.params["items"][0]["section"]),
        ("workspace/configuration", &json!("compmd"))
    );
    client.send(Message::Response(Response::new_ok(
        pull.id,
        json!([{ "trace": "messages" }]),
    )));
    assert!(
        client.traces_opening(&dir.join("a.comp.md")),
        "the pulled settings hold"
    );
    client.notify(
        "workspace/didChangeConfiguration",
        json!({ "settings": null }),
    );
    let again = client.next_request();
    assert_eq!(again.method, "workspace/configuration", "asked again");
    client.send(Message::Response(Response::new_ok(again.id, json!([null]))));
    assert!(
        client.traces_opening(&dir.join("b.comp.md")),
        "no settings keep those there were"
    );
    client.stop();
}

#[test]
fn a_documents_semantic_tokens_come_from_its_worker_all_or_in_a_range() {
    let dir = folder("tokens");
    let uri = uri_of(&dir.join("a.comp.md"));
    let (client, initialized) = Client::start(json!({ "capabilities": {} }));
    let provider = &initialized["capabilities"]["semanticTokensProvider"];
    assert_eq!(
        (&provider["full"], &provider["range"]),
        (&json!(true), &json!(true))
    );
    assert_eq!(provider["legend"]["tokenModifiers"], json!(["unknown"]));
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": "::: box\nText.\n::: /box\n" } }));
    let full = client.ask(
        3,
        "textDocument/semanticTokens/full",
        json!({ "textDocument": { "uri": uri } }),
    );
    // `:::` and `box`, twice, with `/` between.
    assert_eq!(
        full.result.expect("tokens")["data"]
            .as_array()
            .map(Vec::len),
        Some(5 * 5)
    );
    let second =
        json!({ "start": { "line": 2, "character": 0 }, "end": { "line": 3, "character": 0 } });
    let ranged = client.ask(
        4,
        "textDocument/semanticTokens/range",
        json!({ "textDocument": { "uri": uri }, "range": second }),
    );
    assert_eq!(
        ranged.result.expect("tokens")["data"]
            .as_array()
            .map(Vec::len),
        Some(3 * 5)
    );
    let elsewhere = client.ask(
        5,
        "textDocument/semanticTokens/full",
        json!({ "textDocument": { "uri": "file:///nowhere.comp.md" } }),
    );
    assert_eq!(
        elsewhere.result,
        Some(Value::Null),
        "a document that isn't open"
    );
    let wrong = client.ask(
        6,
        "textDocument/semanticTokens/range",
        json!({ "textDocument": { "uri": uri } }),
    );
    assert_eq!(
        wrong.error.map(|error| error.code),
        Some(ErrorCode::InvalidParams as i32)
    );
    let unknown = client.ask(7, "compmd/unknown", json!({}));
    assert_eq!(
        unknown.error.map(|error| error.code),
        Some(ErrorCode::MethodNotFound as i32)
    );
    client.stop();
}

#[test]
fn what_isnt_a_file_or_isnt_understood_changes_nothing() {
    let (client, _) = Client::start(json!({ "capabilities": {} }));
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": "untitled:Untitled-1", "languageId": "compmd", "version": 1, "text": "# A\n" } }));
    assert!(
        client.next_of(LogMessage::METHOD)["message"]
            .as_str()
            .is_some_and(|message| message.contains("isn't a file"))
    );
    client.notify("textDocument/didChange", json!({ "textDocument": { "uri": "untitled:Untitled-1", "version": 2 }, "contentChanges": [] }));
    client.notify("textDocument/didOpen", json!({ "nonsense": true }));
    client.notify("compmd/unknown", json!({}));
    client.send(Message::Response(Response::new_ok(7.into(), json!(null))));
    client.stop();
}

#[test]
fn initialize_parameters_that_arent_lsps_stop_the_server_and_a_closed_connection_ends_it() {
    let (server_side, connection) = Connection::memory();
    let server = thread::spawn(move || serve(&server_side));
    connection
        .sender
        .send(Message::Request(Request::new(
            1.into(),
            "initialize".into(),
            json!({ "capabilities": 3 }),
        )))
        .expect("sent");
    assert!(matches!(
        server.join().expect("the thread"),
        Err(ServerError::Json(_))
    ));
    let (server_side, connection) = Connection::memory();
    let server = thread::spawn(move || serve(&server_side));
    connection
        .sender
        .send(Message::Request(Request::new(
            1.into(),
            "initialize".into(),
            json!({ "capabilities": {} }),
        )))
        .expect("sent");
    let _ = connection.receiver.recv().expect("the answer");
    connection
        .sender
        .send(Message::Notification(Notification::new(
            "initialized".into(),
            json!({}),
        )))
        .expect("sent");
    drop(connection);
    assert!(server.join().expect("the thread").is_ok());
}
