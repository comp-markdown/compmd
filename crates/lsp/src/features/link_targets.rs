//! What links and headings' features share ([the language server], *Links and headings*): the link or the heading at
//! a place, where a link leads, and a file's headings and text, from the document's own processor or the project index.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use std::fs;
use std::path::{Path, PathBuf};

use compmd::tree::Position as TreePosition;
use lsp_types::{Location, Position, Range, Uri};

use super::Asked;
use crate::links::document_links::{DocumentLink, LinkKind, document_links};
use crate::links::headings::{Heading, headings};
use crate::links::link_parts::Span;
use crate::links::target::target_of;
use crate::links::wiki_target::{WikiTarget, names_heading, wiki_file, wiki_target};

/// Where a link leads: a file, and a heading of it when the link names one that is there, or a passage of it where
/// the passage matches.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Reached {
    pub(super) file: PathBuf,
    pub(super) heading: Option<Heading>,
    /// The passage's code points in the file's text.
    pub(super) passage: Option<Span>,
}

/// The project's root: the index's, or the document's folder without one.
pub(super) fn root(asked: &Asked) -> PathBuf {
    asked.index.map_or_else(
        || {
            asked
                .path
                .parent()
                .map_or_else(PathBuf::new, Path::to_path_buf)
        },
        |index| index.root().to_path_buf(),
    )
}

/// The document's links the server follows.
pub(super) fn links(asked: &Asked) -> Vec<DocumentLink> {
    asked
        .processor
        .map(|processor| document_links(&processor.tree().children))
        .unwrap_or_default()
        .into_iter()
        .filter(|link| followed(link, asked))
        .collect()
}

/// Whether the server follows `link`: a wiki link while `links.wiki` is on, any other always.
pub(super) fn followed(link: &DocumentLink, asked: &Asked) -> bool {
    asked.links.follows_wiki() || !matches!(link.kind, LinkKind::Wiki { .. })
}

/// The file a wiki link's `page`, in `file`, names among the project's: the index's, and the document.
pub(super) fn wiki_file_of(page: &str, file: &Path, asked: &Asked) -> Option<PathBuf> {
    let mut files: Vec<PathBuf> = asked
        .index
        .map(|index| index.files().keys().cloned().collect())
        .unwrap_or_default();
    if !files.iter().any(|known| known == asked.path) {
        files.push(asked.path.to_path_buf());
    }
    wiki_file(page, file, &root(asked), files.iter().map(PathBuf::as_path))
}

/// The heading of `file` that `written`, a wiki link's heading, names.
pub(super) fn wiki_heading(written: &str, file: &Path, asked: &Asked) -> Option<Heading> {
    headings_of(file, asked)?
        .into_iter()
        .find(|heading| names_heading(written, heading))
}

/// The link whose text holds `position`.
pub(super) fn link_at(position: Position, asked: &Asked) -> Option<DocumentLink> {
    links(asked).into_iter().find(|link| {
        let range = asked.encoding.span(asked.text, &link.position);
        range.start <= position && position <= range.end
    })
}

/// The heading on `position`'s line.
pub(super) fn heading_at(position: Position, asked: &Asked) -> Option<Heading> {
    let processor = asked.processor?;
    headings(processor)
        .into_iter()
        .find(|heading| line_of(&heading.position) == position.line)
}

/// The headings of `file`: the document's own from its processor, another's from the index.
pub(super) fn headings_of(file: &Path, asked: &Asked) -> Option<Vec<Heading>> {
    if file == asked.path {
        return asked.processor.map(headings);
    }
    let index = asked.index?;
    index
        .files()
        .get(file)
        .map(|indexed| indexed.headings.clone())
}

/// The text of `file`: the document's own as the editor holds it, another's from the disk.
pub(super) fn text_of(file: &Path, asked: &Asked) -> Option<String> {
    if file == asked.path {
        return Some(asked.text.to_owned());
    }
    fs::read_to_string(file).ok()
}

/// Where `link` leads: its destination's file (the document's own for an anchor or a passage alone) when that file is
/// there, with the heading it names or its passage; a reference by the definition of its label; a wiki link by its
/// page. `None` for a URL or a file that isn't there.
pub(super) fn reached(link: &DocumentLink, asked: &Asked) -> Option<Reached> {
    let url = match &link.kind {
        LinkKind::Reference { identifier } => definition_url(identifier, asked)?,
        LinkKind::Wiki { target } => return wiki_reached(target, asked),
        _ => link.url()?.to_owned(),
    };
    let target = target_of(&url, asked.path, &root(asked))?;
    let file = target.file.unwrap_or_else(|| asked.path.to_path_buf());
    if file != asked.path && !file.is_file() {
        return None;
    }
    let heading = target.anchor.and_then(|anchor| {
        headings_of(&file, asked)?
            .into_iter()
            .find(|heading| heading.anchor == anchor)
    });
    let passage = target
        .passage
        .and_then(|passage| passage.find(&text_of(&file, asked)?));
    Some(Reached {
        file,
        heading,
        passage,
    })
}

/// Where a wiki link to `target` leads: the file its page names, with the heading it names.
fn wiki_reached(target: &str, asked: &Asked) -> Option<Reached> {
    let WikiTarget { page, heading } = wiki_target(target);
    let file = wiki_file_of(&page, asked.path, asked)?;
    let heading = heading.and_then(|written| wiki_heading(&written, &file, asked));
    Some(Reached {
        file,
        heading,
        passage: None,
    })
}

/// The destination of the document's definition of the label `identifier`.
pub(super) fn definition_url(identifier: &str, asked: &Asked) -> Option<String> {
    links(asked).into_iter().find_map(|link| match link.kind {
        LinkKind::Definition {
            identifier: defined,
            url,
        } if defined == identifier => Some(url),
        _ => None,
    })
}

/// The document's definition of the label `identifier`.
pub(super) fn definition_of(identifier: &str, asked: &Asked) -> Option<DocumentLink> {
    links(asked).into_iter().find(
        |link| matches!(&link.kind, LinkKind::Definition { identifier: defined, .. } if defined == identifier),
    )
}

/// `file` as a location: at `position` in it, or at its start.
pub(super) fn location(
    file: &Path,
    position: Option<&TreePosition>,
    asked: &Asked,
) -> Option<Location> {
    let uri = uri_of(file)?;
    let range = match position {
        Some(position) => asked.encoding.span(&text_of(file, asked)?, position),
        None => Range::default(),
    };
    Some(Location { uri, range })
}

/// The URI of `file`.
pub(super) fn uri_of(file: &Path) -> Option<Uri> {
    url::Url::from_file_path(file).ok()?.as_str().parse().ok()
}

/// The line, from 0, a node at `position` starts on.
pub(super) fn line_of(position: &TreePosition) -> u32 {
    u32::try_from(position.start.line.saturating_sub(1)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod wiki_tests;
