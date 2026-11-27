//! The main loop ([the language server], decision 2): it reads the client's messages and hands each document's work
//! to its worker, and sends the client what the workers post. A request about a document waits only for that
//! document's worker; nothing here processes a document.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;

use compmd_project::{DiskFiles, root_of};
use crossbeam_channel::{Receiver, Sender, select, unbounded};
use lsp_server::ErrorCode;
use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{
    DidChangeConfiguration, DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument,
    DidOpenTextDocument, LogMessage, Notification as _, PublishDiagnostics,
};
use lsp_types::request::{
    ApplyWorkspaceEdit, ExecuteCommand, InlayHintRefreshRequest, RegisterCapability, Request as _,
    ShowMessageRequest, WillRenameFiles, WorkspaceConfiguration, WorkspaceSymbolRequest,
};
use lsp_types::{
    ApplyWorkspaceEditParams, ClientCapabilities, CodeActionOptions, CodeActionProviderCapability,
    CompletionOptions, ConfigurationItem, ConfigurationParams, DidChangeConfigurationParams,
    DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DocumentOnTypeFormattingOptions, ExecuteCommandOptions,
    ExecuteCommandParams, FileEvent, FileOperationFilter, FileOperationPattern,
    FileOperationRegistrationOptions, FoldingRangeProviderCapability, HoverProviderCapability,
    InitializeParams, InitializeResult, LogMessageParams, MessageActionItem, MessageType, OneOf,
    PublishDiagnosticsParams, RegistrationParams, RenameFilesParams, RenameOptions,
    SemanticTokensFullOptions, SemanticTokensOptions, SemanticTokensServerCapabilities,
    ServerCapabilities, ServerInfo, ShowMessageRequestParams, SignatureHelpOptions,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions, Uri,
    WorkDoneProgressOptions, WorkspaceEdit, WorkspaceFileOperationsServerCapabilities,
    WorkspaceServerCapabilities, WorkspaceSymbolParams,
};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::add_registry::AddRegistry;
use crate::browser::open_in_browser;
use crate::document_preview::PreviewSource;
use crate::encoding::Encoding;
use crate::error::ServerError;
use crate::features::broken_links::is_broken_link;
use crate::features::code_actions::open_preview;
use crate::features::drawing::{self, ClientDrawing, Planning, Refresh};
use crate::features::{
    DocumentRequest, code_actions, file_renames, inlay_hints, on_type_formatting, semantic_tokens,
    workspace_symbols,
};
use crate::files::OpenTexts;
use crate::links::wiki_links::WikiLinks;
use crate::own_packs::NPM;
use crate::plain_markdown_file::is_plain_markdown_file;
use crate::preview::PreviewFiles;
use crate::project_index::ProjectIndex;
use crate::registry_packs::{RegistryPacks, user_cache};
use crate::settings::{Settings, Trace};
use crate::watched_files;
use crate::worker::{self, Document, Job, Output, Queue};

/// A document open in the editor: its worker's queue.
struct Open {
    jobs: Sender<Job>,
    /// Whether it is a `.md` file, which `markdown.enabled` switches off.
    plain_markdown_file: bool,
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
    /// The fetch thread for registry packs.
    registry: RegistryPacks,
    /// Whether the client applies the server's edits (`workspace/applyEdit`).
    applies_edits: bool,
    /// The projects whose writer was asked to add the registry of CompMD's own packs, each once.
    questioned: HashSet<PathBuf>,
    /// The edit each question still unanswered offers, by its request's id.
    questions: HashMap<RequestId, WorkspaceEdit>,
    /// What opens a page in the browser: the system's, or the one the settings name.
    open_page: Opener,
    /// Where a document's preview gets its page and bundle.
    previews: PreviewSource,
    /// The drawing the client declared, if any.
    drawing: Option<ClientDrawing>,
    /// What a drawing plan is made with now: the client's primitives and the drawing's settings.
    planning: Arc<Planning>,
    /// Whether the client takes `workspace/inlayHint/refresh`.
    refreshes_inlay_hints: bool,
    /// Each root's project index, made when its first document opens (decision 15).
    indexes: HashMap<PathBuf, Arc<ProjectIndex>>,
}

/// Opens a page, at the address it is given, in the browser the second argument names, or the system's.
pub(crate) type Opener = Box<dyn Fn(&str, Option<&str>) -> io::Result<()>>;

/// The answer to the question that adds a registry.
const ADD: &str = "Add";

/// The path of a `file:` URI.
fn path_of(uri: &Uri) -> Option<PathBuf> {
    file_path(uri.as_str())
}

/// The path of a `file:` URI written as a string.
fn file_path(uri: &str) -> Option<PathBuf> {
    url::Url::parse(uri).ok()?.to_file_path().ok()
}

/// The parameters of `notification`, when they are what its method takes.
fn params<T: DeserializeOwned>(notification: Notification) -> Option<T> {
    serde_json::from_value(notification.params).ok()
}

/// What the server can do, its positions counted in `encoding`; the drawing plan when the client declared it `draws`.
fn capabilities(encoding: Encoding, draws: bool) -> ServerCapabilities {
    // `compmd/blocks` (content languages in editors, decision 3) and `compmd/drawing` (a document drawn in the editor,
    // decision 3), requests of CompMD's own.
    let mut experimental = serde_json::json!({ "compmdBlocks": true });
    if draws {
        experimental["compmd"] = serde_json::json!({ "drawing": {} });
    }
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
            // A component's name after its marker, a key or a value in a param list, a path's next step; a link's
            // target after `(` or `/`, its anchor after `#`, a reference's label after `[`.
            trigger_characters: Some(
                [":", " ", "{", "=", ".", "#", "(", "/", "["]
                    .map(String::from)
                    .to_vec(),
            ),
            ..CompletionOptions::default()
        }),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        // Links and headings across a project (decision 15).
        references_provider: Some(OneOf::Left(true)),
        workspace_symbol_provider: Some(OneOf::Left(true)),
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),
        // A file or a folder renamed or moved: the links to it, and its own, edited first.
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: None,
            file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                will_rename: Some(FileOperationRegistrationOptions {
                    filters: vec![FileOperationFilter {
                        scheme: Some("file".into()),
                        pattern: FileOperationPattern {
                            glob: "**/*".into(),
                            matches: None,
                            options: None,
                        },
                    }],
                }),
                ..WorkspaceFileOperationsServerCapabilities::default()
            }),
        }),
        document_symbol_provider: Some(OneOf::Left(true)),
        folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(code_actions::kinds()),
            ..CodeActionOptions::default()
        })),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec![open_preview::COMMAND.to_owned()],
            ..ExecuteCommandOptions::default()
        }),
        // Tables aligned (a document drawn in the editor, decision 7).
        document_formatting_provider: Some(OneOf::Left(true)),
        document_range_formatting_provider: Some(OneOf::Left(true)),
        document_on_type_formatting_provider: Some(DocumentOnTypeFormattingOptions {
            first_trigger_character: on_type_formatting::TRIGGER.to_owned(),
            more_trigger_character: Some(
                on_type_formatting::MORE_TRIGGERS.map(String::from).to_vec(),
            ),
        }),
        experimental: Some(experimental),
        // Values beside what gives them (a document drawn in the editor, decision 9).
        inlay_hint_provider: Some(OneOf::Left(true)),
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
        if let Some(edit) = self.questions.remove(&response.id) {
            return self.question_answered(response.result.as_ref(), edit);
        }
        if self.settings_request.as_ref() != Some(&response.id) {
            return;
        }
        self.settings_request = None;
        if let Some(Value::Array(sections)) = response.result
            && let Some(settings) = sections.first().filter(|settings| !settings.is_null())
        {
            self.set_settings(Settings::from_value(settings));
        }
    }

    /// Asks the writer whether to add the registry of CompMD's own packs to the configuration of the project at
    /// `root`, as `add` does: once for each project, and only of a client that applies the server's edits.
    fn ask_to_add(&mut self, root: PathBuf, add: &AddRegistry) {
        if !self.applies_edits || !self.questioned.insert(root) {
            return;
        }
        let action = |title: &str| MessageActionItem {
            title: title.to_owned(),
            properties: HashMap::new(),
        };
        let id = self.request::<ShowMessageRequest>(ShowMessageRequestParams {
            typ: MessageType::INFO,
            message: add.question(),
            actions: Some(vec![action(ADD), action("Not now")]),
        });
        self.questions.insert(id, add.edit.clone());
    }

    /// The writer's answer to a question that offers `edit`: **Add** has the client make it; anything else leaves the
    /// import's warning and its quick fix.
    fn question_answered(&mut self, answer: Option<&Value>, edit: WorkspaceEdit) {
        let chosen = answer
            .and_then(|answer| answer.get("title"))
            .and_then(Value::as_str);
        if chosen == Some(ADD) {
            self.request::<ApplyWorkspaceEdit>(ApplyWorkspaceEditParams {
                label: Some("Add a registry for CompMD's packs".into()),
                edit,
            });
        }
    }

    /// Takes the client's `settings`: new suggested scopes make every document's suggestions asked again, new drawing
    /// settings every open document's plan, and values shown or hidden every open document's inlay hints.
    fn set_settings(&mut self, settings: Settings) {
        let values_switched = settings.render.values != self.settings.render.values;
        let markdown_switched = settings.markdown.enabled != self.settings.markdown.enabled;
        let warnings_switched = settings.links.warns() != self.settings.links.warns();
        self.registry.set_network(settings.packs.network);
        if self
            .registry
            .set_writer_scopes(settings.packs.suggest.clone())
        {
            self.reload_all();
        }
        if settings.render != self.planning.render {
            self.planning = Arc::new(Planning {
                primitives: self.planning.primitives,
                render: settings.render.clone(),
            });
            if !self.documents.is_empty() {
                self.refresh_drawing();
            }
        }
        let wiki_links = WikiLinks::of(&settings);
        let wiki_switched = wiki_links != WikiLinks::of(&self.settings);
        self.settings = settings;
        if markdown_switched {
            self.markdown_switched();
        }
        if wiki_switched {
            self.wiki_links_switched(wiki_links);
        } else if warnings_switched {
            // Each document's diagnostics posted again, with its broken links or without.
            self.reload_all();
        }
        if values_switched && !self.documents.is_empty() {
            self.refresh_inlay_hints();
        }
    }

    /// `markdown.enabled` changed: the `.md` files' diagnostics posted again (none while it is off), and their
    /// drawing and inlay hints asked for again.
    fn markdown_switched(&mut self) {
        let mut any = false;
        for open in self
            .documents
            .values()
            .filter(|open| open.plain_markdown_file)
        {
            let _ = open.jobs.send(Job::Reload);
            any = true;
        }
        if any {
            self.refresh_drawing();
            self.refresh_inlay_hints();
        }
    }

    /// `markdown.wikiLinks`, `links.wiki` or `links.enabled` changed how wiki links are read or followed: each index
    /// told, and read again where `.md` files now parse otherwise; every document processed again, and its drawing
    /// asked for again.
    fn wiki_links_switched(&mut self, wiki_links: WikiLinks) {
        for index in self.indexes.values() {
            if index.set_wiki_links(wiki_links) {
                index.read_again_in_background(self.open.clone(), self.output.clone());
            }
        }
        self.reload_all();
        if !self.documents.is_empty() {
            self.refresh_drawing();
        }
    }

    /// Asks the client to ask for its documents' plans again, when it takes refreshes.
    fn refresh_drawing(&mut self) {
        if self.drawing.is_some_and(|drawing| drawing.refresh_support) {
            self.request::<Refresh>(());
        }
    }

    /// Asks the client to ask for its documents' inlay hints again, when it takes refreshes.
    fn refresh_inlay_hints(&mut self) {
        if self.refreshes_inlay_hints {
            self.request::<InlayHintRefreshRequest>(());
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
    fn send(&mut self, output: Output) {
        match output {
            Output::Diagnostics {
                uri,
                version,
                diagnostics,
            } => {
                let served = self
                    .documents
                    .get(uri.as_str())
                    .is_none_or(|open| self.is_served(open));
                let warns = self.settings.links.warns();
                let diagnostics = diagnostics
                    .into_iter()
                    .filter(|diagnostic| served && (warns || !is_broken_link(diagnostic)))
                    .collect();
                self.notify::<PublishDiagnostics>(PublishDiagnosticsParams {
                    uri,
                    diagnostics,
                    version,
                });
            }
            Output::Log(message) => self.log(message),
            Output::Response(response) => self.respond(response),
            Output::Question { root, add } => self.ask_to_add(root, &add),
            Output::Preview { id, url } => self.open_preview(id, &url),
            Output::DrawingMoved => self.refresh_drawing(),
            Output::ValuesChanged if self.settings.render.values => self.refresh_inlay_hints(),
            Output::ValuesChanged => {}
        }
    }

    /// The document's preview is at `url`: the browser opens it, and the request `id` is answered with it, or with why
    /// the browser couldn't be started.
    fn open_preview(&self, id: RequestId, url: &str) {
        let browser = self.settings.preview.browser.as_deref();
        let response = match (self.open_page)(url, browser) {
            Ok(()) => Response::new_ok(id, url),
            Err(error) => Response::new_err(
                id,
                ErrorCode::RequestFailed as i32,
                format!("the browser can't be opened at {url}: {error}"),
            ),
        };
        self.respond(response);
    }

    /// `workspace/executeCommand`: `compmd.preview` goes to its document's worker, which opens its preview.
    fn command(&self, id: RequestId, params: Value) {
        let invalid = |message: String| {
            Response::new_err(id.clone(), ErrorCode::InvalidParams as i32, message)
        };
        let params: ExecuteCommandParams = match serde_json::from_value(params) {
            Ok(params) => params,
            Err(error) => return self.respond(invalid(error.to_string())),
        };
        if params.command != open_preview::COMMAND {
            return self.respond(invalid(format!(
                "`{}` isn't a command this server runs",
                params.command
            )));
        }
        let uri = params.arguments.first().and_then(Value::as_str);
        match uri.and_then(|uri| self.documents.get(uri)) {
            Some(open) => {
                let _ = open.jobs.send(Job::Preview { id });
            }
            None => self.respond(invalid(format!(
                "`{}` takes the URI of a document open in the editor",
                open_preview::COMMAND
            ))),
        }
    }

    /// The client's request: one about a document goes to its worker, which answers it; one about a document that
    /// isn't open is answered with nothing; one the server doesn't know, with an error.
    fn requested(&self, request: Request) {
        let Request { id, method, params } = request;
        if method == ExecuteCommand::METHOD {
            return self.command(id, params);
        }
        if method == WorkspaceSymbolRequest::METHOD {
            return self.workspace_symbols(id, params);
        }
        if method == WillRenameFiles::METHOD {
            return self.will_rename_files(id, params);
        }
        let read = if method == drawing::METHOD {
            Some(drawing::read(params).map(|(uri, range)| {
                let planning = Arc::clone(&self.planning);
                (uri, DocumentRequest::Drawing { range, planning })
            }))
        } else {
            DocumentRequest::read(&method, params)
        };
        let Some(asked) = read else {
            let message = format!("`{method}` isn't a request this server answers");
            return self.respond(Response::new_err(
                id,
                ErrorCode::MethodNotFound as i32,
                message,
            ));
        };
        match asked {
            Ok((_, request)) if self.is_switched_off(&request) => {
                self.respond(Response::new_ok(id, Value::Null));
            }
            Ok((uri, request)) => match self.documents.get(uri.as_str()) {
                Some(open) if self.is_served(open) => {
                    let links = self.settings.links;
                    let _ = open.jobs.send(Job::Request { id, request, links });
                }
                _ => self.respond(Response::new_ok(id, Value::Null)),
            },
            Err(error) => self.respond(Response::new_err(
                id,
                ErrorCode::InvalidParams as i32,
                error.to_string(),
            )),
        }
    }

    /// `workspace/symbol`: the headings of every root's index the query matches; none while `links.enabled` is off.
    fn workspace_symbols(&self, id: RequestId, params: Value) {
        let query = serde_json::from_value::<WorkspaceSymbolParams>(params)
            .map(|params| params.query)
            .unwrap_or_default();
        let symbols = if self.settings.links.enabled {
            workspace_symbols::workspace_symbols(&query, self.indexes.values().map(AsRef::as_ref))
        } else {
            Vec::new()
        };
        self.respond(Response::new_ok(id, symbols));
    }

    /// `workspace/willRenameFiles`: the edits to every root's links the renames need; none while `links.rename` or
    /// `links.enabled` is off.
    fn will_rename_files(&self, id: RequestId, params: Value) {
        let renames: Vec<file_renames::Moved> = serde_json::from_value::<RenameFilesParams>(params)
            .map(|params| params.files)
            .unwrap_or_default()
            .iter()
            .filter_map(|rename| Some((file_path(&rename.old_uri)?, file_path(&rename.new_uri)?)))
            .collect();
        let edit = (self.settings.links.renames() && !renames.is_empty()).then(|| {
            file_renames::file_renames(
                &renames,
                self.indexes.values().map(AsRef::as_ref),
                &self.open,
                self.encoding,
            )
        });
        self.respond(Response::new_ok(id, edit));
    }

    /// Whether the server answers about `open`: a `.md` file only while `markdown.enabled` is on (a document drawn in
    /// the editor, decision 13).
    fn is_served(&self, open: &Open) -> bool {
        !open.plain_markdown_file || self.settings.markdown.enabled
    }

    /// Whether the settings switch off what `request` asks for: tables aligned on formatting, or as they are typed;
    /// values as inlay hints.
    fn is_switched_off(&self, request: &DocumentRequest) -> bool {
        let tables = &self.settings.tables;
        match request {
            DocumentRequest::Formatting(_) => !tables.align_on_format,
            DocumentRequest::OnTypeFormatting { .. } => !tables.align_on_type,
            DocumentRequest::InlayHints(_) => !self.settings.render.values,
            _ => false,
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
            DidChangeWatchedFiles::METHOD => {
                let changes = params::<DidChangeWatchedFilesParams>(notification)
                    .map(|params| params.changes)
                    .unwrap_or_default();
                self.watched_files_changed(&changes);
            }
            // A client that answers `workspace/configuration` is asked again; another sends the settings.
            DidChangeConfiguration::METHOD if self.pulls_settings => self.pull_settings(),
            DidChangeConfiguration::METHOD => {
                if let Some(DidChangeConfigurationParams { settings }) = params(notification) {
                    self.set_settings(Settings::from_value(&settings));
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
        let plain_markdown_file = is_plain_markdown_file(&path);
        let index = self.index_of(&root);
        let document = Document {
            uri: text_document.uri.clone(),
            path,
            root,
            index,
            text: text_document.text,
            version: text_document.version,
        };
        let (jobs, received) = unbounded();
        let (encoding, open, output) = (self.encoding, self.open.clone(), self.output.clone());
        let (sender, registry, previews) = (jobs.clone(), self.registry.clone(), self.previews);
        thread::spawn(move || {
            let queue = Queue {
                jobs: &received,
                sender,
            };
            worker::run(document, encoding, open, queue, output, registry, previews);
        });
        self.documents.insert(
            text_document.uri.as_str().to_owned(),
            Open {
                jobs,
                plain_markdown_file,
            },
        );
    }

    fn reload_all(&self) {
        for open in self.documents.values() {
            let _ = open.jobs.send(Job::Reload);
        }
    }

    /// The project index of `root`: made, and read in the background, when its first document opens.
    fn index_of(&mut self, root: &Path) -> Arc<ProjectIndex> {
        if let Some(index) = self.indexes.get(root) {
            return Arc::clone(index);
        }
        let index = ProjectIndex::new(root.to_path_buf());
        index.set_wiki_links(WikiLinks::of(&self.settings));
        index.read_in_background(self.output.clone());
        self.indexes.insert(root.to_path_buf(), Arc::clone(&index));
        index
    }

    /// Files changed on the disk: each Markdown file read again by the index it is under, unless it is open (its
    /// worker keeps its entry); then every document processed again.
    fn watched_files_changed(&self, changes: &[FileEvent]) {
        for change in changes {
            let Some(path) = path_of(&change.uri) else {
                continue;
            };
            let markdown = path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"));
            if !markdown || self.documents.contains_key(change.uri.as_str()) {
                continue;
            }
            for index in self
                .indexes
                .values()
                .filter(|index| path.starts_with(index.root()))
            {
                index.read_again(&path);
            }
        }
        self.reload_all();
    }
}

/// Serves the client on `connection`: `initialize`, then its messages until `shutdown` and `exit`. Registry packs are
/// kept in the user's cache folder.
///
/// # Errors
///
/// [`ServerError`] when the connection fails, or `initialize`'s parameters aren't LSP's.
pub fn serve(connection: &Connection) -> Result<(), ServerError> {
    serve_with(
        connection,
        user_cache(),
        NPM,
        Box::new(open_in_browser),
        PreviewFiles::embedded,
    )
}

/// [`serve`], registry packs kept in `cache`, CompMD's own packs from `own` when a configuration names no registry for
/// them, pages opened with `open_page`, and previews' pages from `previews`.
fn serve_with(
    connection: &Connection,
    cache: PathBuf,
    own: &str,
    open_page: Opener,
    previews: PreviewSource,
) -> Result<(), ServerError> {
    let (id, params) = connection.initialize_start()?;
    let initialize: InitializeParams = serde_json::from_value(params)?;
    let encoding = Encoding::negotiate(&initialize.capabilities);
    let drawing = ClientDrawing::of(&initialize.capabilities);
    let result = InitializeResult {
        capabilities: capabilities(encoding, drawing.is_some()),
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
    let settings = initialize
        .initialization_options
        .as_ref()
        .map(Settings::from_value)
        .unwrap_or_default();
    let mut server = Server {
        sender: connection.sender.clone(),
        encoding,
        workspace_folders,
        settings: Settings::default(),
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
        registry: RegistryPacks::start(cache, own),
        applies_edits: initialize
            .capabilities
            .workspace
            .as_ref()
            .and_then(|workspace| workspace.apply_edit)
            == Some(true),
        questioned: HashSet::new(),
        questions: HashMap::new(),
        open_page,
        previews,
        drawing,
        planning: Arc::new(Planning {
            primitives: drawing
                .map(|drawing| drawing.primitives)
                .unwrap_or_default(),
            render: settings.render.clone(),
        }),
        refreshes_inlay_hints: inlay_hints::takes_refreshes(&initialize.capabilities),
        indexes: HashMap::new(),
    };
    server.set_settings(settings);
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
