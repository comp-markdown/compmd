//! A document's worker ([the language server], decision 2): a thread of its own that owns the document's processor,
//! processes it when it opens, applies each edit incrementally, processes it again when what it reads changed, and
//! posts its diagnostics. A panic in it is caught and logged, and the document processed again at its next change.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use compmd::context::RenderContext;
use compmd::expand::Options as ExpandOptions;
use compmd::processor::Expanded;
use compmd::{AllowedOrigins, Ambient, ParseOptions, Processor, TextEdit};
use compmd_components::ComponentReader;
use compmd_host::ChainedPacks;
use compmd_host::pack::PackName;
use compmd_host::registry::{Options, Resolved};
use compmd_project::Origins;
use crossbeam_channel::{Receiver, Sender};
use lsp_server::{RequestId, Response};
use lsp_types::{Diagnostic as LspDiagnostic, TextDocumentContentChangeEvent, Uri};
use serde_json::json;

use crate::add_registry::AddRegistry;
use crate::diagnostics::{lsp_diagnostic, project_diagnostic};
use crate::document_preview::{DocumentPreview, PreviewSource, line_of};
use crate::encoding::Encoding;
use crate::features::drawing::Layout;
use crate::features::{self, Asked, DocumentRequest};
use crate::fetched_packs::FetchedPacks;
use crate::files::{EditorFiles, OpenTexts};
use crate::own_packs::Unregistered;
use crate::pack_components::{PackComponent, pack_components};
use crate::places::Places;
use crate::preview::PageRequest;
use crate::processor_reads::{Reads, Recorded};
use crate::project::Project;
use crate::registry_packs::RegistryPacks;
use crate::suggested_packs::{SuggestedPacks, scopes_of};

/// What a worker is asked to do.
#[derive(Debug)]
pub(crate) enum Job {
    /// The editor changed the text, at `version`.
    Change {
        version: i32,
        changes: Vec<TextDocumentContentChangeEvent>,
    },
    /// What the document reads changed (its configuration, its packs, a file it imports): process it again.
    Reload,
    /// The client asks `request` about the document, as its request `id`.
    Request {
        id: RequestId,
        request: DocumentRequest,
    },
    /// The fetch thread got what it could of `packs`: the document is processed again with it.
    Fetched {
        packs: Vec<PackName>,
        resolved: Resolved,
    },
    /// The fetch thread got the packs the suggested scopes list, and why a scope couldn't be searched: completion and
    /// auto-import offer them.
    Suggested {
        resolved: Resolved,
        unsearched: Vec<String>,
    },
    /// The writer asks for the document's preview (`compmd.preview`), as its request `id`: it opens if it isn't.
    Preview { id: RequestId },
    /// The document closed.
    Close,
}

/// What a worker posts.
#[derive(Debug)]
pub(crate) enum Output {
    /// The document's diagnostics at `version`.
    Diagnostics {
        uri: Uri,
        version: Option<i32>,
        diagnostics: Vec<LspDiagnostic>,
    },
    /// Something for the editor's log.
    Log(String),
    /// The answer to a request.
    Response(Response),
    /// The document imports one of CompMD's own packs its site can't load: the writer is asked whether to add its
    /// registry to the configuration of the project at `root` (Q4).
    Question { root: PathBuf, add: AddRegistry },
    /// The document's preview is served at `url`: the server opens the browser at it, and answers the request `id`.
    Preview { id: RequestId, url: String },
    /// A change moved the document's drawing beyond the lines it edited: the server asks the client to ask for its
    /// plan again (`compmd/drawing/refresh`).
    DrawingMoved,
}

/// A document as its worker knows it.
pub(crate) struct Document {
    /// Its URI, as the editor names it.
    pub(crate) uri: Uri,
    /// Where it is on the disk.
    pub(crate) path: PathBuf,
    /// Its root folder, which its relative paths resolve in (decision 3).
    pub(crate) root: PathBuf,
    /// Its text, as the editor holds it.
    pub(crate) text: String,
    /// The editor's version of the text.
    pub(crate) version: i32,
}

/// The time now, in nanoseconds since 1970, for `md.now()`.
fn now() -> i128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| i128::try_from(since.as_nanos()).unwrap_or(0))
}

/// The path of `path` in `root`, `/`-separated, as the processor names a document: its file name alone when it isn't
/// under the root.
fn relative(path: &Path, root: &Path) -> String {
    let Ok(inside) = path.strip_prefix(root) else {
        return path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
    };
    let parts: Vec<_> = inside
        .components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect();
    parts.join("/")
}

/// A document processed.
struct Processed {
    processor: Processor<Expanded>,
    /// Where the packs found are.
    places: Places,
    /// How the editor resolves its project's packs from registries.
    registries: Options,
    /// Whether its project's configuration names a registry for CompMD's own packs.
    site_fetches_own: bool,
    /// The scopes its project's configuration suggests packs from.
    suggest: Vec<String>,
    /// What the preview's page is given to process it the same.
    page: PageRequest,
}

/// `text`, the document at `path` in `root`, processed with the project's configuration and packs and those `fetched`
/// from registries (CompMD's own from `own` when the configuration names no registry for them), its components
/// analysed; or why the project can't be read.
fn process(
    path: &Path,
    root: &Path,
    text: &str,
    open: &OpenTexts,
    (fetched, own): (&FetchedPacks, &str),
) -> Result<Processed, String> {
    let Project {
        listed,
        installed,
        mut translated,
        registries,
        site_fetches_own,
        suggest,
    } = Project::read(root, own)?;
    translated.policy.refused.extend(fetched.refused().clone());
    let reads = Rc::new(RefCell::new(Reads::default()));
    let (allowed, allowed_origins) = match translated.allowed_origins {
        Origins::Any => (AllowedOrigins::Every, json!("any")),
        Origins::Listed(origins) => (AllowedOrigins::Listed(origins.clone()), json!(origins)),
    };
    let page = PageRequest {
        path: relative(path, root),
        reads: Rc::clone(&reads),
        policy: translated.policy.clone(),
        ambient: translated.ambient.clone(),
        allowed_origins,
    };
    let ambient = Ambient {
        packs: translated.ambient,
        report: false,
    };
    let context = RenderContext {
        now: now(),
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    };
    let places = Places::new(root, listed.packs().iter().chain(installed.packs()));
    let packs = ChainedPacks::new(vec![
        Box::new(listed),
        Box::new(installed),
        fetched.source(),
    ]);
    let processor = Processor::new(relative(path, root), text)
        .parse(&ParseOptions::default())
        .analysing(ComponentReader)
        .resolve_with(
            Recorded::new(EditorFiles::new(root.to_owned(), open.clone()), &reads),
            Recorded::new(packs, &reads),
            &ambient,
            &translated.policy,
        )
        .validate(allowed)
        .load(context, (), ())
        .expand(&ExpandOptions::default());
    Ok(Processed {
        processor,
        places,
        registries,
        site_fetches_own,
        suggest,
        page,
    })
}

/// A document's worker, its processor and what it posts on.
struct Worker {
    document: Document,
    encoding: Encoding,
    open: OpenTexts,
    output: Sender<Output>,
    processor: Option<Processor<Expanded>>,
    /// Where the files the processor names are.
    places: Places,
    /// The components the project's packs and the suggested ones export, found when a completion first needs them,
    /// and again after the project is read again or the suggested packs come.
    pack_components: Option<Vec<PackComponent>>,
    /// The packs fetched for the document from registries, and those on their way.
    fetched: FetchedPacks,
    /// The packs the suggested scopes list, which completion and auto-import offer.
    suggested: SuggestedPacks,
    /// How the editor resolves the document's project's packs from registries, as it was last read.
    registries: Option<Options>,
    /// The document's imports of CompMD's own packs its site can't load, when its configuration names no registry
    /// for them.
    unregistered: Option<Unregistered>,
    /// Whether the writer was asked to add their registry, which a worker does once.
    questioned: bool,
    /// The document's preview, once the writer opens it.
    preview: DocumentPreview,
    /// The fetch thread.
    registry: RegistryPacks,
    /// The worker's own queue, which the fetch thread answers on.
    jobs: Sender<Job>,
    /// The lines of the document's elements that span lines, as it was last processed.
    layout: Layout,
}

impl Worker {
    /// Processes the whole text again, and posts its diagnostics; or, when the project can't be read, why. The packs
    /// it imports that nothing holds are asked of the registries first, and processed as pending.
    fn process_whole(&mut self) {
        let processed = loop {
            let Document {
                path, root, text, ..
            } = &self.document;
            let own = self.registry.own();
            let processed = catch_unwind(AssertUnwindSafe(|| {
                process(path, root, text, &self.open, (&self.fetched, own))
            }));
            let Ok(Ok(Processed {
                processor,
                registries,
                ..
            })) = &processed
            else {
                break processed;
            };
            self.registries = Some(registries.clone());
            if !self.ask_missing(processor) {
                break processed;
            }
        };
        let diagnostics = match processed {
            Ok(Ok(Processed {
                processor,
                places,
                registries,
                site_fetches_own,
                suggest,
                page,
            })) => {
                self.places = places;
                self.preview.processed(page);
                self.suggest(registries, &suggest);
                self.unregistered = (!site_fetches_own).then(|| Unregistered {
                    add: AddRegistry::of(&self.document.root, self.registry.own(), self.encoding),
                    warnings: Vec::new(),
                });
                let diagnostics = self.diagnostics_of(&processor);
                self.processor = Some(processor);
                self.pack_components = None;
                diagnostics
            }
            Ok(Err(why)) => {
                self.processor = None;
                vec![project_diagnostic(why)]
            }
            Err(_) => return self.panicked(),
        };
        self.post(diagnostics);
    }

    /// Asks the fetch thread for the packs of the suggested scopes, CompMD's, the project's (`project`) and the
    /// writer's, from the registries of `registries`, unless they were asked as they are.
    fn suggest(&mut self, registries: Options, project: &[String]) {
        let scopes = scopes_of(project, &self.registry.writer_scopes());
        if self.suggested.ask(&scopes) {
            self.registry.suggest(registries, scopes, self.jobs.clone());
        }
    }

    /// What the fetch thread got of the suggested scopes: held for completion, and why a scope couldn't be searched,
    /// logged.
    fn suggested(&mut self, resolved: Resolved, unsearched: Vec<String>) {
        for message in unsearched {
            let _ = self.output.send(Output::Log(message));
        }
        self.suggested.add(resolved);
        self.pack_components = None;
    }

    /// Asks the registries for the packs `processor`'s document imports that nothing holds and weren't asked yet, of
    /// those a registry is set for; gives whether it asked any, which are now pending.
    fn ask_missing(&mut self, processor: &Processor<Expanded>) -> bool {
        let Some(options) = &self.registries else {
            return false;
        };
        let missing = processor.sources().packs.into_iter();
        let asked = self.fetched.ask(
            missing
                .filter(|needed| options.registry_of(&needed.pack).is_some())
                .collect(),
        );
        if asked.is_empty() {
            return false;
        }
        self.registry
            .fetch(options.clone(), asked, self.jobs.clone());
        true
    }

    /// The document's diagnostics: the processor's, and a warning on each import of one of CompMD's own packs its site
    /// can't load. The first time there is one, the writer is asked whether to add their registry.
    fn diagnostics_of(&mut self, processor: &Processor<Expanded>) -> Vec<LspDiagnostic> {
        let mut diagnostics = self.lsp_diagnostics(processor);
        let Some(unregistered) = &mut self.unregistered else {
            return diagnostics;
        };
        unregistered.find(processor, &self.places, &self.document.text, self.encoding);
        diagnostics.extend(unregistered.warnings.iter().cloned());
        if !self.questioned
            && !unregistered.warnings.is_empty()
            && let Some(add) = &unregistered.add
        {
            self.questioned = true;
            let _ = self.output.send(Output::Question {
                root: self.document.root.clone(),
                add: add.clone(),
            });
        }
        diagnostics
    }

    /// The processor's diagnostics as LSP's.
    fn lsp_diagnostics(&self, processor: &Processor<Expanded>) -> Vec<LspDiagnostic> {
        let Document { uri, text, .. } = &self.document;
        processor
            .diagnostics()
            .iter()
            .map(|diagnostic| lsp_diagnostic(diagnostic, uri, text, self.encoding))
            .collect()
    }

    fn post(&self, diagnostics: Vec<LspDiagnostic>) {
        let _ = self.output.send(Output::Diagnostics {
            uri: self.document.uri.clone(),
            version: Some(self.document.version),
            diagnostics,
        });
    }

    /// The document's layout as it is processed now.
    fn current_layout(&self) -> Layout {
        self.processor
            .as_ref()
            .map(|processor| Layout::of(processor.tree(), &self.document.text))
            .unwrap_or_default()
    }

    /// The document was processed again: when its layout isn't the one `expected` (none for a change of the whole
    /// text), the drawing moved.
    fn laid_out(&mut self, expected: Option<&Layout>) {
        let layout = self.current_layout();
        if expected != Some(&layout) {
            let _ = self.output.send(Output::DrawingMoved);
        }
        self.layout = layout;
    }

    /// The document was processed again with the same text: when its layout changed, the drawing moved.
    fn relaid(&mut self) {
        let before = std::mem::take(&mut self.layout);
        self.laid_out(Some(&before));
    }

    /// Processing panicked: logged, and the document processed from scratch at its next change.
    fn panicked(&mut self) {
        self.processor = None;
        let _ = self.output.send(Output::Log(format!(
            "processing {} failed; it is processed again at its next change",
            self.document.uri.as_str()
        )));
    }

    /// Applies the editor's `changes`: each a range's new text, applied to the processor incrementally, or the
    /// whole text, processed again.
    fn change(&mut self, version: i32, changes: Vec<TextDocumentContentChangeEvent>) {
        self.document.version = version;
        let line = line_of(&changes);
        let mut whole = self.processor.is_none();
        let mut expected = Some(self.layout.clone());
        for change in changes {
            expected = expected.and_then(|layout| layout.moved(&change));
            let Some(range) = change.range else {
                self.document.text = change.text;
                whole = true;
                continue;
            };
            let text = &self.document.text;
            let (start, end) = (
                self.encoding.offset(text, range.start),
                self.encoding.offset(text, range.end),
            );
            let edit = TextEdit {
                start,
                end,
                text: change.text,
            };
            let byte = |offset: usize| {
                text.char_indices()
                    .nth(offset)
                    .map_or(text.len(), |(byte, _)| byte)
            };
            let (from, to) = (byte(start), byte(end.max(start)));
            self.document.text.replace_range(from..to, &edit.text);
            if let (false, Some(processor)) = (whole, self.processor.take()) {
                if let Ok(edited) = catch_unwind(AssertUnwindSafe(|| processor.edit(&edit))) {
                    self.processor = Some(edited);
                } else {
                    self.panicked();
                    whole = true;
                }
            }
        }
        self.open
            .set(self.document.path.clone(), self.document.text.clone());
        self.preview.changed(&self.document.text, line);
        if whole {
            self.process_whole();
            return self.laid_out(expected.as_ref());
        }
        let Some(processor) = self.processor.take() else {
            return;
        };
        if self.ask_missing(&processor) {
            self.process_whole();
            return self.laid_out(expected.as_ref());
        }
        let diagnostics = self.diagnostics_of(&processor);
        self.processor = Some(processor);
        self.post(diagnostics);
        self.preview.read_more();
        self.laid_out(expected.as_ref());
    }

    /// The writer asks for the preview, as the request `id`: opened if it isn't, and its address posted for the
    /// server to open; or why it can't be.
    fn open_preview(&mut self, id: RequestId) {
        let Document { root, text, .. } = &self.document;
        let output = match self.preview.open(root, text, self.registries.as_ref()) {
            Ok(url) => Output::Preview { id, url },
            Err(why) => Output::Response(Response::new_err(
                id,
                lsp_server::ErrorCode::RequestFailed as i32,
                why,
            )),
        };
        let _ = self.output.send(output);
    }

    /// What the fetch thread got of `packs`: held, its problems logged, and the document processed again with it.
    fn fetched(&mut self, packs: &[PackName], resolved: Resolved) {
        for problem in self.fetched.add(packs, resolved) {
            let _ = self.output.send(Output::Log(format!(
                "`{}` couldn't be fetched: {}",
                problem.pack, problem.message
            )));
        }
        self.process_whole();
        self.relaid();
    }
}

/// A worker's queue: the jobs it takes, and a sender to it, for the fetch thread to answer on.
pub(crate) struct Queue<'q> {
    pub(crate) jobs: &'q Receiver<Job>,
    pub(crate) sender: Sender<Job>,
}

/// Runs the worker of `document` until it closes: processes it, then each job its `queue` brings, posting on `output`
/// and asking `registry` for the packs nothing holds; its preview, when opened, from `previews`.
pub(crate) fn run(
    document: Document,
    encoding: Encoding,
    open: OpenTexts,
    queue: Queue<'_>,
    output: Sender<Output>,
    registry: RegistryPacks,
    previews: PreviewSource,
) {
    open.set(document.path.clone(), document.text.clone());
    let mut worker = Worker {
        document,
        encoding,
        open,
        output,
        processor: None,
        places: Places::default(),
        pack_components: None,
        fetched: FetchedPacks::default(),
        suggested: SuggestedPacks::default(),
        registries: None,
        unregistered: None,
        questioned: false,
        preview: DocumentPreview::new(previews),
        registry,
        jobs: queue.sender,
        layout: Layout::default(),
    };
    worker.process_whole();
    worker.layout = worker.current_layout();
    for job in queue.jobs {
        match job {
            Job::Change { version, changes } => worker.change(version, changes),
            Job::Reload => {
                worker.fetched.ask_again();
                worker.suggested.ask_again();
                worker.process_whole();
                worker.relaid();
            }
            Job::Fetched { packs, resolved } => worker.fetched(&packs, resolved),
            Job::Suggested {
                resolved,
                unsearched,
            } => worker.suggested(resolved, unsearched),
            Job::Preview { id } => worker.open_preview(id),
            Job::Request { id, request } => {
                if matches!(
                    request,
                    DocumentRequest::Completion(_) | DocumentRequest::CodeActions { .. }
                ) && worker.pack_components.is_none()
                {
                    worker.pack_components = Some(pack_components(
                        &worker.document.root,
                        worker.suggested.source(),
                    ));
                }
                let asked = Asked {
                    processor: worker.processor.as_ref(),
                    uri: &worker.document.uri,
                    text: &worker.document.text,
                    version: worker.document.version,
                    encoding: worker.encoding,
                    places: &worker.places,
                    pack_components: worker.pack_components.as_deref().unwrap_or_default(),
                    unregistered: worker.unregistered.as_ref(),
                };
                let answer = features::answer(&request, &asked);
                let _ = worker
                    .output
                    .send(Output::Response(Response::new_ok(id, answer)));
            }
            Job::Close => break,
        }
    }
    worker.open.close(&worker.document.path);
    let _ = worker.output.send(Output::Diagnostics {
        uri: worker.document.uri.clone(),
        version: None,
        diagnostics: Vec::new(),
    });
}

#[cfg(test)]
mod tests;
