//! Rename ([the language server], *Links and headings*, `links.rename`; `textDocument/prepareRename` and `rename`): a
//! heading's text, with every link to its anchor across the project given the new anchor and every wiki link to it
//! the new text; a reference's or a definition's label, with its uses in the document. The new anchor is the one the
//! processor gives the heading with its new text, so a heading of the same text elsewhere is counted as the spec
//! counts it (`-1`).
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use std::collections::HashMap;
use std::path::Path;

use lsp_types::{Position, Range, TextEdit, Uri, WorkspaceEdit};

use super::Asked;
use super::link_targets::{heading_at, link_at, links, text_of, uri_of};
use super::references::{label_of, links_reaching};
use crate::encoding::Encoding;
use crate::links::document_links::{DocumentLink, LinkKind};
use crate::links::headings::Heading;
use crate::links::link_parts::{Span, fragment_span, label_span, wiki_spans};
use crate::links::text_span::{range_of, spanned};
use crate::project_index::IndexedFile;

/// The range renamed at `position`: a heading's text, or a reference's or a definition's label; `None` where nothing
/// is renamed, or while `links.rename` is off.
pub(crate) fn prepare_rename(position: Position, asked: &Asked) -> Option<Range> {
    if !asked.links.renames() {
        return None;
    }
    if let Some(heading) = heading_at(position, asked) {
        let span = heading_text_span(asked.text, &heading)?;
        return Some(range_of(asked.text, asked.encoding, span));
    }
    let link = link_at(position, asked)?;
    label_of(&link)?;
    let (from, to) = label_span(&spanned(asked.text, &link.position), &link)?;
    let start = link.position.start.offset;
    Some(range_of(
        asked.text,
        asked.encoding,
        (start + from, start + to),
    ))
}

/// The edits that rename what is at `position` to `new_name`; `None` where nothing is renamed, `new_name` can't be
/// written there (a line break, a label's bracket), or while `links.rename` is off.
pub(crate) fn rename(position: Position, new_name: &str, asked: &Asked) -> Option<WorkspaceEdit> {
    if !asked.links.renames() || new_name.trim().is_empty() || new_name.contains(['\n', '\r']) {
        return None;
    }
    let mut edits = Edits::new(asked.encoding);
    if let Some(heading) = heading_at(position, asked) {
        rename_heading(&heading, new_name, asked, &mut edits)?;
    } else {
        rename_label(position, new_name, asked, &mut edits)?;
    }
    Some(edits.into_workspace_edit())
}

/// The heading's text edited, and the links to its anchor given the new one.
fn rename_heading(
    heading: &Heading,
    new_name: &str,
    asked: &Asked,
    edits: &mut Edits,
) -> Option<()> {
    let (from, to) = heading_text_span(asked.text, heading)?;
    let renamed: String = asked
        .text
        .chars()
        .take(from)
        .chain(new_name.chars())
        .chain(asked.text.chars().skip(to))
        .collect();
    let markdown_wiki_links = asked
        .index
        .is_some_and(|index| index.wiki_links().in_markdown_files);
    let anchor = IndexedFile::read(asked.path, &renamed, markdown_wiki_links)
        .headings
        .into_iter()
        .find(|new| new.position.start.line == heading.position.start.line)?
        .anchor;
    edits.add(asked.path, asked.text, (from, to), new_name);
    for (file, link) in links_reaching(heading, asked) {
        let Some(text) = text_of(&file, asked) else {
            continue;
        };
        let written = spanned(&text, &link.position);
        let Some(((from, to), new)) = heading_part(&written, &link, heading, (&anchor, new_name))
        else {
            continue;
        };
        let unchanged: String = written.chars().skip(from).take(to - from).collect();
        if unchanged != new {
            let start = link.position.start.offset;
            edits.add(&file, &text, (start + from, start + to), new);
        }
    }
    Some(())
}

/// Where `written`, `link`'s text, names `heading`, and what it names it after the rename, given the heading's new
/// `anchor` and `text`: a link's anchor the new anchor; a wiki link's heading the new text, or the new anchor where
/// it wrote the anchor.
fn heading_part<'a>(
    written: &str,
    link: &DocumentLink,
    heading: &Heading,
    (anchor, text): (&'a str, &'a str),
) -> Option<(Span, &'a str)> {
    if !matches!(link.kind, LinkKind::Wiki { .. }) {
        return Some((fragment_span(written, link)?, anchor));
    }
    let (from, to) = wiki_spans(written)?.1?;
    let named: String = written.chars().skip(from).take(to - from).collect();
    Some((
        (from, to),
        if named == heading.anchor {
            anchor
        } else {
            text
        },
    ))
}

/// The label of the reference or definition at `position` renamed, with every use of it in the document.
fn rename_label(
    position: Position,
    new_name: &str,
    asked: &Asked,
    edits: &mut Edits,
) -> Option<()> {
    if new_name.contains(['[', ']']) {
        return None;
    }
    let label = label_of(&link_at(position, asked)?)?;
    for link in links(asked) {
        if label_of(&link).as_deref() != Some(&label) {
            continue;
        }
        if let Some((from, to)) = label_span(&spanned(asked.text, &link.position), &link) {
            let start = link.position.start.offset;
            edits.add(asked.path, asked.text, (start + from, start + to), new_name);
        }
    }
    Some(())
}

/// Where `heading`'s text is in `text`: its first line, past an ATX heading's `#`s and before its closing ones.
fn heading_text_span(text: &str, heading: &Heading) -> Option<(usize, usize)> {
    let start = heading.position.start.offset;
    let line: Vec<char> = text
        .chars()
        .skip(start)
        .take_while(|c| *c != '\n' && *c != '\r')
        .collect();
    let mut from = line.iter().take_while(|c| **c == ' ').count();
    let marks = line[from..].iter().take_while(|c| **c == '#').count();
    if marks > 0 {
        from += marks;
        from += line[from..]
            .iter()
            .take_while(|c| matches!(c, ' ' | '\t'))
            .count();
    }
    let mut to = line.len();
    while to > from && matches!(line[to - 1], ' ' | '\t') {
        to -= 1;
    }
    if marks > 0 {
        let closing = line[from..to]
            .iter()
            .rev()
            .take_while(|c| **c == '#')
            .count();
        if closing > 0 && (to - closing == from || matches!(line[to - closing - 1], ' ' | '\t')) {
            to -= closing;
            while to > from && matches!(line[to - 1], ' ' | '\t') {
                to -= 1;
            }
        }
    }
    (to > from).then_some((start + from, start + to))
}

/// The edits of a rename, by file, their ranges in the client's encoding.
struct Edits {
    encoding: Encoding,
    changes: HashMap<Uri, Vec<TextEdit>>,
}

impl Edits {
    fn new(encoding: Encoding) -> Self {
        Self {
            encoding,
            changes: HashMap::new(),
        }
    }

    /// `new_text` in place of the code points `span` of `file`, whose text is `text`.
    fn add(&mut self, file: &Path, text: &str, span: (usize, usize), new_text: &str) {
        let Some(uri) = uri_of(file) else { return };
        self.changes.entry(uri).or_default().push(TextEdit {
            range: range_of(text, self.encoding, span),
            new_text: new_text.to_owned(),
        });
    }

    fn into_workspace_edit(self) -> WorkspaceEdit {
        WorkspaceEdit {
            changes: Some(self.changes),
            ..WorkspaceEdit::default()
        }
    }
}

#[cfg(test)]
mod tests;
