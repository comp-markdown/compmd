//! References (`textDocument/references`, [the language server], *Links and headings*): on a heading, every link to
//! its anchor and every wiki link to it across the project; on a reference or a definition, the uses of its label in
//! the document.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use std::path::{Path, PathBuf};

use lsp_types::{Location, Position};

use super::Asked;
use super::link_targets::{followed, heading_at, link_at, links, location, root, wiki_file_of};
use crate::links::document_links::{DocumentLink, LinkKind};
use crate::links::headings::Heading;
use crate::links::target::target_of;
use crate::links::wiki_target::{WikiTarget, names_heading, wiki_target};

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
pub(super) fn label_of(link: &DocumentLink) -> Option<String> {
    match &link.kind {
        LinkKind::Reference { identifier } | LinkKind::Definition { identifier, .. } => {
            Some(identifier.clone())
        }
        LinkKind::Inline { .. } | LinkKind::Wiki { .. } => None,
    }
}

/// The links across the project to `heading` of the document, as locations.
fn links_to(heading: &Heading, asked: &Asked) -> Vec<Location> {
    links_reaching(heading, asked)
        .into_iter()
        .filter_map(|(file, link)| location(&file, Some(&link.position), asked))
        .collect()
}

/// The links across the project to `heading` of the document, each with its file: the document's own, then each
/// indexed file's.
pub(super) fn links_reaching(heading: &Heading, asked: &Asked) -> Vec<(PathBuf, DocumentLink)> {
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
            links
                .into_iter()
                .filter(|link| followed(link, asked) && leads_to(link, &file, heading, asked))
                .map(|link| (file.clone(), link))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Whether `link`, in `file`, leads to `heading` of the document: by its anchor, or a wiki link by its text.
fn leads_to(link: &DocumentLink, file: &Path, heading: &Heading, asked: &Asked) -> bool {
    if let LinkKind::Wiki { target } = &link.kind {
        let WikiTarget {
            page,
            heading: written,
        } = wiki_target(target);
        return written.is_some_and(|written| names_heading(&written, heading))
            && wiki_file_of(&page, file, asked).as_deref() == Some(asked.path);
    }
    let Some(target) = link
        .url()
        .and_then(|url| target_of(url, file, &root(asked)))
    else {
        return false;
    };
    target.file.as_deref().unwrap_or(file) == asked.path
        && target.anchor.as_deref() == Some(&heading.anchor)
}
