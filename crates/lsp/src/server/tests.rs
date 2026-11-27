//! The server driven by a client in memory, as an editor drives it.

use std::fs;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use lsp_server::{Request, Response};
use serde_json::{Value, json};

use super::*;

/// A preview's page and bundle for the tests: the page, and the bundle's manifest and one frame.
#[expect(
    clippy::unnecessary_wraps,
    reason = "a preview's source, which gives none in a build without the page"
)]
fn previews() -> Option<Result<PreviewFiles, String>> {
    let hash = "1df336e75685de8c";
    let manifest = format!(r#"{{"format": 1, "frames": {{"hash": "{hash}"}}}}"#);
    let bundle = compmd_host::Bundle::from_files([
        ("manifest.json".to_owned(), manifest.into_bytes()),
        (
            format!("frames/{hash}/strict.html"),
            b"<!doctype html>".to_vec(),
        ),
    ])
    .map_err(|error| error.to_string());
    let page = [(
        "index.html",
        b"<!doctype html><html><head></head></html>".to_vec(),
    )];
    Some(bundle.map(|bundle| PreviewFiles::new(page, bundle)))
}

/// The pages the server opened in a browser, each with the browser named, which a test's server records in place of
/// starting one.
type Opened = Arc<Mutex<Vec<(String, Option<String>)>>>;

/// A client: its side of the connection, the server's thread, and the pages it opened.
struct Client {
    connection: Connection,
    server: JoinHandle<Result<(), ServerError>>,
    opened: Opened,
}

impl Client {
    /// A client initialized with `params`, and the server's answer; registry packs kept in a folder of the test's.
    fn start(params: Value) -> (Self, Value) {
        let cache =
            std::env::temp_dir().join(format!("compmd-lsp-server-{}-cache", std::process::id()));
        Self::start_with(params, cache)
    }

    /// A client initialized with `params`, registry packs kept in `cache`, and the server's answer; CompMD's own packs
    /// from a registry nothing serves.
    fn start_with(params: Value, cache: PathBuf) -> (Self, Value) {
        Self::start_own(params, cache, "http://127.0.0.1:9/".into())
    }

    /// A client initialized with `params`, registry packs kept in `cache` and CompMD's own fetched from `own`, and the
    /// server's answer.
    fn start_own(params: Value, cache: PathBuf, own: String) -> (Self, Value) {
        let (server_side, connection) = Connection::memory();
        let opened = Opened::default();
        let recorded = Arc::clone(&opened);
        let server = thread::spawn(move || {
            let open: Opener = Box::new(move |url, browser| {
                recorded
                    .lock()
                    .expect("recorded")
                    .push((url.to_owned(), browser.map(str::to_owned)));
                Ok(())
            });
            serve_with(&server_side, cache, &own, open, previews)
        });
        let client = Self {
            connection,
            server,
            opened,
        };
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

    /// The next notification of `method`, skipping the others; the server sending a request meanwhile fails the test.
    fn next_unasked(&self, method: &str) -> Value {
        loop {
            match self.next() {
                Message::Notification(notification) if notification.method == method => {
                    return notification.params;
                }
                Message::Request(request) => panic!("the server asked {}", request.method),
                _ => {}
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

    /// Whether opening `path` is traced in the log: the next line about it says it opened. Lines about other things
    /// (a registry an earlier document couldn't search) are skipped.
    fn traces_opening(&self, path: &std::path::Path) -> bool {
        self.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri_of(path), "languageId": "compmd", "version": 1, "text": "# A\n" } }));
        let named = path.display().to_string();
        loop {
            let message = self.next_of(LogMessage::METHOD);
            let line = message["message"].as_str().unwrap_or_default();
            if line.contains(&named) {
                return line.contains("opened");
            }
        }
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

mod drawing;
mod inlay_hints;
mod links;
mod markdown_files;
mod own_packs;
mod preview;
mod registry;
mod suggested;
mod tables;

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
fn completion_hover_and_signature_help_come_from_the_documents_hints() {
    let dir = folder("hints");
    let uri = uri_of(&dir.join("a.comp.md"));
    let (client, initialized) = Client::start(json!({ "capabilities": {} }));
    assert_eq!(initialized["capabilities"]["hoverProvider"], json!(true));
    assert!(initialized["capabilities"]["completionProvider"]["triggerCharacters"].is_array());
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": "::: bo\n" } }));
    let at = json!({ "textDocument": { "uri": uri }, "position": { "line": 0, "character": 6 } });
    let completed = client
        .ask(3, "textDocument/completion", at.clone())
        .result
        .expect("completions");
    assert!(
        completed
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item["label"] == "box")),
        "{completed}"
    );
    let hovered = client
        .ask(4, "textDocument/hover", at)
        .result
        .expect("a hover");
    assert!(
        hovered["contents"]["value"]
            .as_str()
            .is_some_and(|text| text.contains("`bo`")),
        "{hovered}"
    );
    let change = json!({ "text": "::: box {padding=(md.number(1, ))}\n::: /box\n" });
    client.notify(
        "textDocument/didChange",
        json!({ "textDocument": { "uri": uri, "version": 2 }, "contentChanges": [change] }),
    );
    let in_call =
        json!({ "textDocument": { "uri": uri }, "position": { "line": 0, "character": 31 } });
    let helped = client
        .ask(5, "textDocument/signatureHelp", in_call)
        .result
        .expect("help");
    assert_eq!(helped["activeParameter"], json!(1), "{helped}");
    client.stop();
}

#[test]
fn a_definition_is_found_in_the_document_or_in_the_file_it_imports() {
    let dir = folder("definition");
    fs::write(
        dir.join("notes.comp.md"),
        "---\nmd:\n  export:\n    names: \"*\"\n---\n```compmd component=note\n---\nA note.\n```\n",
    )
    .expect("written");
    let uri = uri_of(&dir.join("a.comp.md"));
    let text =
        "---\nmd:\n  import:\n    - from: ./notes.comp.md\n      names: \"*\"\n---\n\n:: note[N]\n";
    let (client, initialized) = Client::start(json!({ "capabilities": {} }));
    assert_eq!(
        initialized["capabilities"]["definitionProvider"],
        json!(true)
    );
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": text } }));
    let on_note =
        json!({ "textDocument": { "uri": uri }, "position": { "line": 7, "character": 4 } });
    let found = client
        .ask(3, "textDocument/definition", on_note)
        .result
        .expect("a location");
    assert_eq!(found["uri"], json!(uri_of(&dir.join("notes.comp.md"))));
    assert_eq!(
        found["range"]["start"],
        json!({ "line": 5, "character": 0 })
    );
    let prose =
        json!({ "textDocument": { "uri": uri }, "position": { "line": 6, "character": 0 } });
    assert_eq!(
        client.ask(4, "textDocument/definition", prose).result,
        Some(Value::Null)
    );
    client.stop();
}

#[test]
fn a_folder_packs_component_completes_with_its_import() {
    let dir = folder("auto-import");
    fs::create_dir_all(dir.join("components")).expect("a folder");
    fs::write(
        dir.join("compmd.config.json"),
        r#"{"packs": {"./components": {}}}"#,
    )
    .expect("written");
    fs::write(
        dir.join("components/package.json"),
        r#"{"name": "@site/components", "version": "2.1.0"}"#,
    )
    .expect("written");
    fs::write(dir.join("components/index.comp.md"), "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=note\n---\nA note.\n```\n").expect("written");
    let uri = uri_of(&dir.join("a.comp.md"));
    let (client, _) = Client::start(json!({ "capabilities": {} }));
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": "::: no\n" } }));
    let at = json!({ "textDocument": { "uri": uri }, "position": { "line": 0, "character": 6 } });
    let items = client
        .ask(3, "textDocument/completion", at)
        .result
        .expect("completions");
    let note = items
        .as_array()
        .and_then(|items| items.iter().find(|item| item["label"] == "note"))
        .expect("`note`");
    assert_eq!(note["detail"], "imports it from `@site/components@2`");
    assert!(
        note["additionalTextEdits"][0]["newText"]
            .as_str()
            .is_some_and(|text| text.starts_with("---\nmd:\n  import:\n"))
    );
    let unknown = uri_of(&dir.join("b.comp.md"));
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": unknown, "languageId": "compmd", "version": 1, "text": ":: note[x]\n" } }));
    let caret =
        json!({ "start": { "line": 0, "character": 4 }, "end": { "line": 0, "character": 4 } });
    let actions = client
        .ask(
            4,
            "textDocument/codeAction",
            json!({ "textDocument": { "uri": unknown }, "range": caret, "context": { "diagnostics": [] } }),
        )
        .result
        .expect("actions");
    assert_eq!(
        actions[0]["title"],
        "Import `note` from `@site/components@2`"
    );
    assert_eq!(actions[0]["kind"], "quickfix");
    client.stop();
}

#[test]
fn an_imported_component_files_signature_is_generated_from_its_import_and_updated_from_its_fence() {
    let dir = folder("component-file");
    fs::write(
        dir.join("Chart.tsx"),
        "export default function Chart({ rows, height = 200 }: { rows: string; height?: number }) { return null; }\n",
    )
    .expect("written");
    let uri = uri_of(&dir.join("a.comp.md"));
    let (client, _) = Client::start(json!({ "capabilities": {} }));
    let import = "---\nmd:\n  import:\n    - from: ./Chart.tsx\n      names: [{name: Chart, as: chart}]\n---\n";
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": format!("{import}\n# Sales\n") } }));
    let on = |line: u32| json!({ "textDocument": { "uri": uri }, "range": { "start": { "line": line, "character": 6 }, "end": { "line": line, "character": 6 } }, "context": { "diagnostics": [], "only": ["refactor.rewrite"] } });
    let titled = |actions: &Value, title: &str| {
        actions
            .as_array()
            .and_then(|actions| actions.iter().find(|action| action["title"] == title))
            .cloned()
            .unwrap_or_else(|| panic!("{title} in {actions}"))
    };
    let actions = client
        .ask(3, "textDocument/codeAction", on(3))
        .result
        .expect("actions");
    let generate = titled(&actions, "Generate the signature of `chart`");
    let edit = &generate["edit"]["changes"][&uri][0];
    assert_eq!(edit["range"]["start"], json!({ "line": 5, "character": 3 }));
    assert_eq!(
        edit["newText"],
        "\n\n```compmd component=chart signature\nparams:\n  rows: string\n  height: number = 200\n```"
    );
    // With a signature fence that lacks `height`, the fence is brought up to date.
    let signed =
        format!("{import}\n```compmd component=chart signature\nparams:\n  rows: string\n```\n");
    client.notify("textDocument/didChange", json!({ "textDocument": { "uri": uri, "version": 2 }, "contentChanges": [{ "text": signed }] }));
    let actions = client
        .ask(4, "textDocument/codeAction", on(8))
        .result
        .expect("actions");
    let update = titled(&actions, "Update the signature of `chart`");
    assert_eq!(
        update["edit"]["changes"][&uri][0]["newText"],
        "params:\n  rows: string\n  height: number = 200\n"
    );
    client.stop();
}

#[test]
fn a_rendered_block_has_its_languages_other_names_from_its_pack() {
    let dir = folder("languages");
    fs::create_dir_all(dir.join("diagrams")).expect("a folder");
    fs::write(
        dir.join("compmd.config.json"),
        r#"{"packs": {"./diagrams": {}}}"#,
    )
    .expect("written");
    fs::write(
        dir.join("diagrams/package.json"),
        r#"{"name": "@site/diagrams", "version": "1.0.0", "compmd": {"document": "index.comp.md", "languages": [{"id": "flow", "aliases": ["flowchart"], "languageId": "flow", "extensions": [".flow"]}]}}"#,
    )
    .expect("written");
    fs::write(dir.join("diagrams/index.comp.md"), "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=flow-diagram\nrenders: flow\n---\n{{code}}\n```\n").expect("written");
    let uri = uri_of(&dir.join("a.comp.md"));
    let (client, _) = Client::start(json!({ "capabilities": {} }));
    let text = "---\nmd:\n  import:\n    - from: \"@site/diagrams@1\"\n      names: \"*\"\n---\n\n```flow\na -> b\n```\n";
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": text } }));
    let blocks = client
        .ask(
            3,
            "compmd/blocks",
            json!({ "textDocument": { "uri": uri } }),
        )
        .result
        .expect("blocks");
    assert_eq!(
        blocks,
        json!([{ "language": "flow", "component": "flow-diagram", "aliases": ["flowchart"], "languageId": "flow", "extensions": [".flow"], "range": { "start": { "line": 8, "character": 0 }, "end": { "line": 8, "character": 6 } } }])
    );
    client.stop();
}

#[test]
fn symbols_folding_and_rendered_blocks_of_a_document() {
    let dir = folder("outline");
    let uri = uri_of(&dir.join("a.comp.md"));
    let text =
        "```compmd component=shout\nrenders: yell\n---\n{{code}}\n```\n\n# A\n\n```yell\nhi\n```\n";
    let (client, initialized) = Client::start(json!({ "capabilities": {} }));
    let capabilities = &initialized["capabilities"];
    assert_eq!(
        (
            &capabilities["documentSymbolProvider"],
            &capabilities["foldingRangeProvider"]
        ),
        (&json!(true), &json!(true))
    );
    assert_eq!(capabilities["experimental"]["compmdBlocks"], json!(true));
    assert_eq!(
        capabilities["codeActionProvider"]["codeActionKinds"][0],
        json!("quickfix")
    );
    client.notify("textDocument/didOpen", json!({ "textDocument": { "uri": uri, "languageId": "compmd", "version": 1, "text": text } }));
    let document = json!({ "textDocument": { "uri": uri } });
    let symbols = client
        .ask(3, "textDocument/documentSymbol", document.clone())
        .result
        .expect("symbols");
    assert_eq!(
        (&symbols[0]["name"], &symbols[1]["name"]),
        (&json!("shout"), &json!("A"))
    );
    let folded = client
        .ask(4, "textDocument/foldingRange", document.clone())
        .result
        .expect("ranges");
    assert_eq!(folded.as_array().map(Vec::len), Some(2));
    let blocks = client
        .ask(5, "compmd/blocks", document)
        .result
        .expect("blocks");
    assert_eq!(
        blocks,
        json!([{ "language": "yell", "component": "shout", "range": { "start": { "line": 9, "character": 0 }, "end": { "line": 9, "character": 2 } } }])
    );
    let wrong = client
        .ask(6, "compmd/blocks", json!({}))
        .error
        .expect("an error");
    assert_eq!(wrong.code, ErrorCode::InvalidParams as i32);
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
