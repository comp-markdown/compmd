//! A document's preview, as its worker keeps it ([the language server], decision 13): the server, once the writer opens
//! it, and what it is given: the document's request after each processing, and each change with the line being
//! edited. It stops when the document closes.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::path::Path;
use std::time::Duration;

use compmd_host::registry::{Client, Options, Proxy};
use compmd_project::{DiskFiles, PackFolders, config_of, integrities};
use lsp_types::TextDocumentContentChangeEvent;

use crate::preview::{PageRequest, Preview, PreviewFiles, ProjectFiles, ProjectStore};

/// How long one request to a registry may take, through the page's `registry/`.
const TIMEOUT: Duration = Duration::from_secs(30);

/// Where a preview's page and bundle come from: what the binary embeds ([`PreviewFiles::embedded`]), or a test's.
pub(crate) type PreviewSource = fn() -> Option<Result<PreviewFiles, String>>;

/// A document's preview, and what it was last given.
pub(crate) struct DocumentPreview {
    source: PreviewSource,
    preview: Option<Preview>,
    request: Option<PageRequest>,
    /// The line being edited, from 1.
    line: usize,
}

/// The line being edited after `changes`, from 1: where the last one's text ends; the first line after a whole new
/// text.
pub(crate) fn line_of(changes: &[TextDocumentContentChangeEvent]) -> Option<usize> {
    let last = changes.last()?;
    let Some(range) = last.range else {
        return Some(1);
    };
    let start = usize::try_from(range.start.line).unwrap_or(usize::MAX);
    Some(
        start
            .saturating_add(last.text.matches('\n').count())
            .saturating_add(1),
    )
}

/// The packs found in the project at `root`, for the frames: its folder packs and its installed ones.
fn project_store(root: &Path) -> ProjectStore {
    let config = config_of(root, &DiskFiles).unwrap_or_default();
    let listed = PackFolders::listed(&config, root, DiskFiles).map(|found| found.packs().to_vec());
    let installed = PackFolders::installed(root, DiskFiles, &integrities(root, &DiskFiles))
        .map(|found| found.packs().to_vec());
    ProjectStore::new(listed.into_iter().chain(installed).flatten().collect())
}

impl DocumentPreview {
    /// A preview not open yet, its page and bundle from `source` when it opens.
    pub(crate) fn new(source: PreviewSource) -> Self {
        Self {
            source,
            preview: None,
            request: None,
            line: 1,
        }
    }

    /// The document was processed: its preview, if open, gets the request it makes, and keeps it for when it opens.
    pub(crate) fn processed(&mut self, request: PageRequest) {
        if let Some(preview) = &self.preview {
            preview.set_request(&request.json());
        }
        self.request = Some(request);
    }

    /// What the processor read grew, after an edit: the open preview gets the request again, a `reload` if it changed.
    pub(crate) fn read_more(&self) {
        if let (Some(preview), Some(request)) = (&self.preview, &self.request) {
            preview.set_request(&request.json());
        }
    }

    /// The document's text is now `text`, the line `line` (from 1) being edited, or the one before when unknown.
    pub(crate) fn changed(&mut self, text: &str, line: Option<usize>) {
        self.line = line.unwrap_or(self.line).max(1);
        if let Some(preview) = &self.preview {
            preview.change(text, self.line);
        }
    }

    /// The preview's address, opening it first when it isn't: for the document at `root` whose text is `text`, the
    /// page's `registry/` passing on to `registries`.
    ///
    /// # Errors
    ///
    /// Why it can't be opened: a build without the preview, or a port or a token it can't have.
    pub(crate) fn open(
        &mut self,
        root: &Path,
        text: &str,
        registries: Option<&Options>,
    ) -> Result<String, String> {
        if let Some(preview) = &self.preview {
            return Ok(preview.url());
        }
        let files = (self.source)().ok_or_else(|| {
            "this build of compmd-lsp has no preview: its page wasn't built".to_owned()
        })??;
        let proxy = registries.map(|options| Proxy::new(options.clone(), Client::new(TIMEOUT)));
        let preview = Preview::start(
            files,
            Box::new(project_store(root)),
            proxy,
            ProjectFiles::new(root),
        )
        .map_err(|error| format!("the preview can't be started: {error}"))?;
        if let Some(request) = &self.request {
            preview.set_request(&request.json());
        }
        preview.change(text, self.line.max(1));
        let url = preview.url();
        self.preview = Some(preview);
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use lsp_types::{Position, Range};

    use super::*;

    fn edit(line: u32, text: &str) -> TextDocumentContentChangeEvent {
        let at = Position { line, character: 0 };
        TextDocumentContentChangeEvent {
            range: Some(Range { start: at, end: at }),
            range_length: None,
            text: text.into(),
        }
    }

    #[test]
    fn the_line_being_edited_is_where_the_last_changes_text_ends() {
        assert_eq!(line_of(&[]), None);
        assert_eq!(line_of(&[edit(4, "x")]), Some(5));
        assert_eq!(line_of(&[edit(0, "a"), edit(2, "two\nlines\n")]), Some(5));
        let whole = TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: "# A\n".into(),
        };
        assert_eq!(line_of(&[whole]), Some(1));
    }

    #[test]
    fn a_build_without_the_preview_says_so() {
        let mut preview = DocumentPreview::new(|| None);
        let opened = preview.open(Path::new("/site"), "# A\n", None);
        assert!(opened.is_err_and(|why| why.contains("has no preview")));
        let broken = || Some(Err("the bundle can't be used".to_owned()));
        assert_eq!(
            DocumentPreview::new(broken).open(Path::new("/site"), "# A\n", None),
            Err("the bundle can't be used".to_owned())
        );
    }
}
