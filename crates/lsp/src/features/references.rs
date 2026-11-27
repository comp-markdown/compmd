//! References (`textDocument/references`, [the language server], *Links and headings*): on a heading, every link to
//! its anchor across the project; on a reference or a definition, the uses of its label in the document.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use std::path::{Path, PathBuf};

use compmd::tree::Position as TreePosition;
use lsp_types::{Location, Position};

use super::Asked;
use super::link_targets::{heading_at, link_at, links, location, root};
use crate::links::document_links::{DocumentLink, LinkKind};
use crate::links::headings::Heading;
use crate::links::target::target_of;

/// The references of what is at `position`, with its own place when `with_declaration`.
pub(crate) fn references(
    position: Position,
    with_declaration: bool,
    asked: &Asked,
) -> Vec<Location> {
    if let Some(heading) = heading_at(position, asked) {
        let mut found = links_to(&heading, asked);
        if with_declaration {
            found.extend(location(asked.path, Some(&heading.position), asked));
        }
        return found;
    }
    let Some(label) = link_at(position, asked).and_then(|link| label_of(&link)) else {
        return Vec::new();
    };
    links(asked)
        .into_iter()
        .filter(|link| label_of(link).as_deref() == Some(&label))
        .filter(|link| with_declaration || matches!(link.kind, LinkKind::Reference { .. }))
        .filter_map(|link| location(asked.path, Some(&link.position), asked))
        .collect()
}

/// The label a reference or a definition names.
fn label_of(link: &DocumentLink) -> Option<String> {
    match &link.kind {
        LinkKind::Reference { identifier } | LinkKind::Definition { identifier, .. } => {
            Some(identifier.clone())
        }
        LinkKind::Inline { .. } => None,
    }
}

/// The links across the project to `heading` of the document: the document's own, then each indexed file's.
fn links_to(heading: &Heading, asked: &Asked) -> Vec<Location> {
    let root = root(asked);
    let mut files: Vec<(PathBuf, Vec<DocumentLink>)> =
        vec![(asked.path.to_path_buf(), links(asked))];
    if let Some(index) = asked.index {
        files.extend(
            index
                .files()
                .iter()
                .filter(|(file, _)| file.as_path() != asked.path)
                .map(|(file, indexed)| (file.clone(), indexed.links.clone())),
        );
    }
    files
        .into_iter()
        .flat_map(|(file, links)| {
            let positions: Vec<TreePosition> = links
                .iter()
                .filter(|link| leads_to(link, &file, heading, asked.path, &root))
                .map(|link| link.position)
                .collect();
            positions
                .into_iter()
                .filter_map(move |position| location(&file, Some(&position), asked))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Whether `link`, in `file`, leads to `heading` of the document at `document`.
fn leads_to(
    link: &DocumentLink,
    file: &Path,
    heading: &Heading,
    document: &Path,
    root: &Path,
) -> bool {
    let Some(target) = link.url().and_then(|url| target_of(url, file, root)) else {
        return false;
    };
    target.file.as_deref().unwrap_or(file) == document
        && target.anchor.as_deref() == Some(&heading.anchor)
}
