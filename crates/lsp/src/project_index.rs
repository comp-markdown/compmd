//! The project index ([the language server], decision 15, Q9): for each Markdown file under a root, its headings and
//! its links, which the features that look beyond one file ask. It is read in the background when the root's first
//! document opens, and answers from what it has read so far; an open document's entry is its worker's, kept current
//! with each change, and a file changed on the disk is read again.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;

use compmd::context::RenderContext;
use compmd::processor::Expanded;
use compmd::{Ambient, ParseOptions, Processor};
use crossbeam_channel::Sender;

use crate::links::document_links::{DocumentLink, document_links};
use crate::links::headings::{Heading, headings};
use crate::plain_markdown_file::is_plain_markdown_file;
use crate::worker::Output;

/// How many files a root's index reads at most, so a root that is a home folder doesn't stall the editor.
const MOST_FILES: usize = 10_000;

/// The largest file the index reads, in bytes.
const LARGEST_FILE: u64 = 1024 * 1024;

/// What the index keeps of a file.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct IndexedFile {
    /// Its headings, with their anchors.
    pub(crate) headings: Vec<Heading>,
    /// Its links.
    pub(crate) links: Vec<DocumentLink>,
}

impl IndexedFile {
    /// What `processor`'s document gives the index.
    pub(crate) fn of(processor: &Processor<Expanded>) -> Self {
        Self {
            headings: headings(processor),
            links: document_links(&processor.tree().children),
        }
    }

    /// The file at `path` with `text`, processed alone (no packs, no project), as the index reads a file that isn't
    /// open: a `.md` file as plain Markdown unless it has an `md` key.
    pub(crate) fn read(path: &Path, text: &str) -> Self {
        let context = RenderContext {
            now: 0,
            tz: "UTC".into(),
            time: Rc::new(()),
            inputs: serde_json::Map::new(),
        };
        let name = path
            .file_name()
            .map_or_else(String::new, |name| name.to_string_lossy().into_owned());
        let processor = Processor::new(name, text)
            .parse(&ParseOptions {
                plain_markdown: is_plain_markdown_file(path),
                ..ParseOptions::default()
            })
            .resolve((), (), &Ambient::default())
            .validate(&[])
            .load(context, (), ())
            .expand(&compmd::expand::Options {
                origins: true,
                ..compmd::expand::Options::default()
            });
        Self::of(&processor)
    }
}

/// A root's index, shared by its documents' workers and the server.
#[derive(Debug)]
pub(crate) struct ProjectIndex {
    root: PathBuf,
    files: Mutex<BTreeMap<PathBuf, IndexedFile>>,
}

impl ProjectIndex {
    /// An empty index of `root`.
    pub(crate) fn new(root: PathBuf) -> Arc<Self> {
        Arc::new(Self {
            root,
            files: Mutex::default(),
        })
    }

    /// The root it indexes.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// Reads the root's Markdown files on a thread of its own, logging on `log` when it stops at its limit. A file
    /// already in the index (an open document's) is kept as it is.
    pub(crate) fn read_in_background(self: &Arc<Self>, log: Sender<Output>) {
        let index = Arc::clone(self);
        thread::spawn(move || {
            let (paths, stopped) = markdown_files(&index.root, MOST_FILES);
            if stopped {
                let _ = log.send(Output::Log(format!(
                    "the links of {} are indexed for its first {MOST_FILES} Markdown files only",
                    index.root.display()
                )));
            }
            for path in paths {
                let Ok(text) = fs::read_to_string(&path) else {
                    continue;
                };
                let file = IndexedFile::read(&path, &text);
                index.files().entry(path).or_insert(file);
            }
        });
    }

    /// Sets what the index has of the file at `path`.
    pub(crate) fn set(&self, path: PathBuf, file: IndexedFile) {
        self.files().insert(path, file);
    }

    /// Reads the file at `path` again from the disk, or leaves it out when it is gone.
    pub(crate) fn read_again(&self, path: &Path) {
        match fs::read_to_string(path) {
            Ok(text) => self.set(path.to_path_buf(), IndexedFile::read(path, &text)),
            Err(_) => {
                self.files().remove(path);
            }
        }
    }

    /// The files indexed so far, to read.
    pub(crate) fn files(&self) -> MutexGuard<'_, BTreeMap<PathBuf, IndexedFile>> {
        self.files.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The Markdown files under `root` (`.md`, `.comp.md` among them): not in `node_modules`, a hidden folder or what a
/// `.gitignore` names, and none larger than [`LARGEST_FILE`]; at most `most`, and whether it stopped there.
fn markdown_files(root: &Path, most: usize) -> (Vec<PathBuf>, bool) {
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .require_git(false)
        .filter_entry(|entry| entry.file_name() != "node_modules")
        .build();
    let mut paths = Vec::new();
    for entry in walker.flatten() {
        let path = entry.path();
        let markdown = path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("md"));
        let small = entry
            .metadata()
            .is_ok_and(|metadata| metadata.is_file() && metadata.len() <= LARGEST_FILE);
        if !(markdown && small) {
            continue;
        }
        if paths.len() == most {
            return (paths, true);
        }
        paths.push(path.to_path_buf());
    }
    (paths, false)
}

#[cfg(test)]
mod tests;
