//! A document's worker ([the language server], decision 2): a thread of its own that owns the document's processor,
//! processes it when it opens, applies each edit incrementally, processes it again when what it reads changed, and
//! posts its diagnostics. A panic in it is caught and logged, and the document processed again at its next change.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::BTreeMap;
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
use compmd_project::{DiskFiles, Origins, PackFolders, config_of, integrities};
use crossbeam_channel::{Receiver, Sender};
use lsp_types::{Diagnostic as LspDiagnostic, TextDocumentContentChangeEvent, Uri};

use crate::diagnostics::{lsp_diagnostic, project_diagnostic};
use crate::encoding::Encoding;
use crate::files::{EditorFiles, OpenTexts};

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
    /// The document closed.
    Close,
}

/// What a worker posts.
#[derive(Debug, PartialEq)]
pub(crate) enum Output {
    /// The document's diagnostics at `version`.
    Diagnostics {
        uri: Uri,
        version: Option<i32>,
        diagnostics: Vec<LspDiagnostic>,
    },
    /// Something for the editor's log.
    Log(String),
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

/// `text`, the document at `path` in `root`, processed with the project's configuration and packs, its components
/// analysed; or why the project can't be read.
fn process(
    path: &Path,
    root: &Path,
    text: &str,
    open: &OpenTexts,
) -> Result<Processor<Expanded>, String> {
    let config = config_of(root, &DiskFiles).map_err(|error| error.to_string())?;
    let installed = PackFolders::installed(root, DiskFiles, &integrities(root, &DiskFiles))
        .map_err(|error| error.to_string())?;
    let listed =
        PackFolders::listed(&config, root, DiskFiles).map_err(|error| error.to_string())?;
    let translated = config.translated(
        |key| listed.name_of(key).or_else(|| key.parse().ok()),
        &BTreeMap::new(),
    );
    let ambient = Ambient {
        packs: translated.ambient,
        report: false,
    };
    let allowed = match translated.allowed_origins {
        Origins::Any => AllowedOrigins::Every,
        Origins::Listed(origins) => AllowedOrigins::Listed(origins),
    };
    let context = RenderContext {
        now: now(),
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    };
    let packs = ChainedPacks::new(vec![Box::new(listed), Box::new(installed)]);
    Ok(Processor::new(relative(path, root), text)
        .parse(&ParseOptions::default())
        .analysing(ComponentReader)
        .resolve_with(
            EditorFiles::new(root.to_owned(), open.clone()),
            packs,
            &ambient,
            &translated.policy,
        )
        .validate(allowed)
        .load(context, (), ())
        .expand(&ExpandOptions::default()))
}

/// A document's worker, its processor and what it posts on.
struct Worker {
    document: Document,
    encoding: Encoding,
    open: OpenTexts,
    output: Sender<Output>,
    processor: Option<Processor<Expanded>>,
}

impl Worker {
    /// Processes the whole text again, and posts its diagnostics; or, when the project can't be read, why.
    fn process_whole(&mut self) {
        let Document {
            path, root, text, ..
        } = &self.document;
        let processed = catch_unwind(AssertUnwindSafe(|| process(path, root, text, &self.open)));
        let diagnostics = match processed {
            Ok(Ok(processor)) => {
                let diagnostics = self.lsp_diagnostics(&processor);
                self.processor = Some(processor);
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
        let mut whole = self.processor.is_none();
        for change in changes {
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
        if whole {
            return self.process_whole();
        }
        if let Some(processor) = &self.processor {
            let diagnostics = self.lsp_diagnostics(processor);
            self.post(diagnostics);
        }
    }
}

/// Runs the worker of `document` until it closes: processes it, then each job `jobs` brings, posting on `output`.
pub(crate) fn run(
    document: Document,
    encoding: Encoding,
    open: OpenTexts,
    jobs: &Receiver<Job>,
    output: Sender<Output>,
) {
    open.set(document.path.clone(), document.text.clone());
    let mut worker = Worker {
        document,
        encoding,
        open,
        output,
        processor: None,
    };
    worker.process_whole();
    for job in jobs {
        match job {
            Job::Change { version, changes } => worker.change(version, changes),
            Job::Reload => worker.process_whole(),
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
