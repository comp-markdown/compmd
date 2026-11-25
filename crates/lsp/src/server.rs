//! The main loop ([the language server], decision 2): it reads the client's messages and hands each document's work
//! to its worker, and sends the client what the workers post. A request about a document waits only for that
//! document's worker; nothing here processes a document.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;

use compmd_project::{DiskFiles, root_of};
use crossbeam_channel::{Receiver, Sender, select, unbounded};
use lsp_server::ErrorCode;
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
    DidChangeConfiguration, DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument,
    DidOpenTextDocument, LogMessage, Notification as _, PublishDiagnostics,
};
use lsp_types::request::{RegisterCapability, WorkspaceConfiguration};
use lsp_types::{
    ClientCapabilities, CompletionOptions, ConfigurationItem, ConfigurationParams,
    DidChangeConfigurationParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, FoldingRangeProviderCapability, HoverProviderCapability,
    InitializeParams, InitializeResult, LogMessageParams, MessageType, OneOf,
    PublishDiagnosticsParams, RegistrationParams, SemanticTokensFullOptions, SemanticTokensOptions,
    SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo, SignatureHelpOptions,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, Uri,
};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::encoding::Encoding;
use crate::error::ServerError;
use crate::features::{DocumentRequest, semantic_tokens};
use crate::files::OpenTexts;
use crate::settings::{Settings, Trace};
use crate::watched_files;
use crate::worker::{self, Document, Job, Output};

/// A document open in the editor: its worker's queue.
struct Open {
    jobs: Sender<Job>,
}

/// The server between `initialize` and `shutdown`.
struct Server {
    sender: Sender<Message>,
    encoding: Encoding,
    workspace_folders: Vec<PathBuf>,
    settings: Settings,
    /// Whether the client answers `workspace/configuration`.
    pulls_settings: bool,
    /// The id of the last `workspace/configuration` request, until it is answered.
    settings_request: Option<RequestId>,
    /// The id of the last request the server sent.
    last_request: i32,
    open: OpenTexts,
    documents: HashMap<String, Open>,
    output: Sender<Output>,
}

/// The path of a `file:` URI.
fn path_of(uri: &Uri) -> Option<PathBuf> {
    url::Url::parse(uri.as_str()).ok()?.to_file_path().ok()
}

/// The parameters of `notification`, when they are what its method takes.
fn params<T: DeserializeOwned>(notification: Notification) -> Option<T> {
    serde_json::from_value(notification.params).ok()
}

/// What the server can do, its positions counted in `encoding`.
fn capabilities(encoding: Encoding) -> ServerCapabilities {
    ServerCapabilities {
        position_encoding: Some(encoding.kind()),
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::INCREMENTAL),
                ..TextDocumentSyncOptions::default()
            },
        )),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                legend: semantic_tokens::legend(),
                range: Some(true),
                full: Some(SemanticTokensFullOptions::Bool(true)),
                ..SemanticTokensOptions::default()
            },
        )),
        completion_provider: Some(CompletionOptions {
            // A component's name after its marker, a key or a value in a param list, a path's next step.
            trigger_characters: Some([":", " ", "{", "=", ".", "#"].map(String::from).to_vec()),
            ..CompletionOptions::default()
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        document_symbol_provider: Some(OneOf::Left(true)),
        folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
        // `compmd/blocks`, a request of CompMD's own (content languages in editors, decision 3).
        experimental: Some(serde_json::json!({ "compmdBlocks": true })),
        signature_help_provider: Some(SignatureHelpOptions {
            // A call's arguments, a param list's params.
            trigger_characters: Some(["(", ",", "{", " "].map(String::from).to_vec()),
            ..SignatureHelpOptions::default()
        }),
        ..ServerCapabilities::default()
    }
}

/// Whether the client lets the server register the files it watches.
fn registers_watched_files(capabilities: &ClientCapabilities) -> bool {
    capabilities
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.did_change_watched_files?.dynamic_registration)
        == Some(true)
}

impl Server {
    /// Sends the client the request `R`, and gives its id.
    fn request<R: lsp_types::request::Request>(&mut self, params: R::Params) -> RequestId {
        self.last_request += 1;
        let id = RequestId::from(self.last_request);
        let _ = self.sender.send(Message::Request(Request::new(
            id.clone(),
            R::METHOD.to_owned(),
            params,
        )));
        id
    }

    /// Registers the files watched (decision 8).
    fn watch_files(&mut self) {
        self.request::<RegisterCapability>(RegistrationParams {
            registrations: vec![watched_files::registration()],
        });
    }

    /// Asks the client for the `compmd` settings, when it answers that (decision 12).
    fn pull_settings(&mut self) {
        if self.pulls_settings {
            let item = ConfigurationItem {
                scope_uri: None,
                section: Some("compmd".into()),
            };
            self.settings_request = Some(
                self.request::<WorkspaceConfiguration>(ConfigurationParams { items: vec![item] }),
            );
        }
    }

    /// The client's answer to a request: the settings, when it answers the last `workspace/configuration`; a client
    /// with none keeps those it gave at `initialize`.
    fn answered(&mut self, response: Response) {
        if self.settings_request.as_ref() != Some(&response.id) {
            return;
        }
        self.settings_request = None;
        if let Some(Value::Array(sections)) = response.result
            && let Some(settings) = sections.first().filter(|settings| !settings.is_null())
        {
            self.settings = Settings::from_value(settings);
        }
    }

    fn notify<N: lsp_types::notification::Notification>(&self, params: N::Params) {
        let _ = self.sender.send(Message::Notification(Notification::new(
            N::METHOD.to_owned(),
            params,
        )));
    }

    fn log(&self, message: String) {
        self.notify::<LogMessage>(LogMessageParams {
            typ: MessageType::LOG,
            message,
        });
    }

    /// What a worker posted, to the client.
    fn send(&self, output: Output) {
        match output {
            Output::Diagnostics {
                uri,
                version,
                diagnostics,
            } => {
                self.notify::<PublishDiagnostics>(PublishDiagnosticsParams {
                    uri,
                    diagnostics,
                    version,
                });
            }
            Output::Log(message) => self.log(message),
            Output::Response(response) => self.respond(response),
        }
    }

    /// The client's request: one about a document goes to its worker, which answers it; one about a document that
    /// isn't open is answered with nothing; one the server doesn't know, with an error.
    fn requested(&self, request: Request) {
        let Request { id, method, params } = request;
        let Some(asked) = DocumentRequest::read(&method, params) else {
            let message = format!("`{method}` isn't a request this server answers");
            return self.respond(Response::new_err(
                id,
                ErrorCode::MethodNotFound as i32,
                message,
            ));
        };
        match asked {
            Ok((uri, request)) => match self.documents.get(uri.as_str()) {
                Some(open) => {
                    let _ = open.jobs.send(Job::Request { id, request });
                }
                None => self.respond(Response::new_ok(id, Value::Null)),
            },
            Err(error) => self.respond(Response::new_err(
                id,
                ErrorCode::InvalidParams as i32,
                error.to_string(),
            )),
        }
    }

    fn respond(&self, response: Response) {
        let _ = self.sender.send(Message::Response(response));
    }

    fn handle(&mut self, notification: Notification) {
        match notification.method.as_str() {
            DidOpenTextDocument::METHOD => {
                if let Some(params) = params::<DidOpenTextDocumentParams>(notification) {
                    self.opened(params);
                }
            }
            DidChangeTextDocument::METHOD => {
                if let Some(DidChangeTextDocumentParams {
                    text_document,
                    content_changes,
                }) = params(notification)
                    && let Some(open) = self.documents.get(text_document.uri.as_str())
                {
                    let _ = open.jobs.send(Job::Change {
                        version: text_document.version,
                        changes: content_changes,
                    });
                }
            }
            DidCloseTextDocument::METHOD => {
                if let Some(DidCloseTextDocumentParams { text_document }) = params(notification)
                    && let Some(open) = self.documents.remove(text_document.uri.as_str())
                {
                    let _ = open.jobs.send(Job::Close);
                }
            }
            // A file a document reads changed on the disk (its configuration, its packs, an import), or what the
            // editor holds of one: every document is processed again.
            DidChangeWatchedFiles::METHOD => self.reload_all(),
            // A client that answers `workspace/configuration` is asked again; another sends the settings.
            DidChangeConfiguration::METHOD if self.pulls_settings => self.pull_settings(),
            DidChangeConfiguration::METHOD => {
                if let Some(DidChangeConfigurationParams { settings }) = params(notification) {
                    self.settings = Settings::from_value(&settings);
                }
            }
            _ => {}
        }
    }

    fn opened(&mut self, DidOpenTextDocumentParams { text_document }: DidOpenTextDocumentParams) {
        let Some(path) = path_of(&text_document.uri) else {
            return self.log(format!(
                "{} isn't a file: it isn't processed",
                text_document.uri.as_str()
            ));
        };
        let root = root_of(&path, &self.workspace_folders, &DiskFiles);
        if self.settings.trace != Trace::Off {
            self.log(format!(
                "{} opened, its root {}",
                path.display(),
                root.display()
            ));
        }
        let document = Document {
            uri: text_document.uri.clone(),
            path,
            root,
            text: text_document.text,
            version: text_document.version,
        };
        let (jobs, received) = unbounded();
        let (encoding, open, output) = (self.encoding, self.open.clone(), self.output.clone());
        thread::spawn(move || worker::run(document, encoding, open, &received, output));
        self.documents
            .insert(text_document.uri.as_str().to_owned(), Open { jobs });
    }

    fn reload_all(&self) {
        for open in self.documents.values() {
            let _ = open.jobs.send(Job::Reload);
        }
    }
}

/// Serves the client on `connection`: `initialize`, then its messages until `shutdown` and `exit`.
///
/// # Errors
///
/// [`ServerError`] when the connection fails, or `initialize`'s parameters aren't LSP's.
pub fn serve(connection: &Connection) -> Result<(), ServerError> {
    let (id, params) = connection.initialize_start()?;
    let initialize: InitializeParams = serde_json::from_value(params)?;
    let encoding = Encoding::negotiate(&initialize.capabilities);
    let result = InitializeResult {
        capabilities: capabilities(encoding),
        server_info: Some(ServerInfo {
            name: "compmd-lsp".into(),
            version: Some(env!("CARGO_PKG_VERSION").into()),
        }),
    };
    connection.initialize_finish(id, serde_json::to_value(result)?)?;
    let workspace_folders = initialize
        .workspace_folders
        .unwrap_or_default()
        .iter()
        .filter_map(|folder| path_of(&folder.uri))
        .collect();
    let (output, posted): (Sender<Output>, Receiver<Output>) = unbounded();
    let mut server = Server {
        sender: connection.sender.clone(),
        encoding,
        workspace_folders,
        settings: initialize
            .initialization_options
            .as_ref()
            .map(Settings::from_value)
            .unwrap_or_default(),
        pulls_settings: initialize
            .capabilities
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.configuration)
            == Some(true),
        settings_request: None,
        last_request: 0,
        open: OpenTexts::default(),
        documents: HashMap::new(),
        output,
    };
    if registers_watched_files(&initialize.capabilities) {
        server.watch_files();
    }
    server.pull_settings();
    loop {
        select! {
            recv(connection.receiver) -> message => match message {
                Ok(Message::Request(request)) => {
                    if connection.handle_shutdown(&request)? {
                        return Ok(());
                    }
                    server.requested(request);
                }
                Ok(Message::Notification(notification)) => server.handle(notification),
                Ok(Message::Response(response)) => server.answered(response),
                Err(_) => return Ok(()),
            },
            recv(posted) -> output => {
                if let Ok(output) = output {
                    server.send(output);
                }
            },
        }
    }
}

#[cfg(test)]
mod tests;
