//! The broken-link warning ([the language server], *Links and headings*, `links.diagnostics`): a link to a project
//! file that isn't there, to a heading anchor its target hasn't, to a passage that matches nothing in it, a wiki link
//! to a page or a heading that isn't there, and a reference whose label no definition defines (Q10). The server's own
//! warning, beside the processor's diagnostics: source `compmd`, code `broken-link`, never one of the spec's codes. A
//! target the index hasn't read yet, or that isn't Markdown, has its anchors left unchecked, and a wiki link's page is
//! checked once the index has read the root.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range};

use super::Asked;
use super::link_targets::{headings_of, links, root, text_of, wiki_file_of, wiki_heading};
use crate::diagnostics::SOURCE;
use crate::links::document_links::{DocumentLink, LinkKind};
use crate::links::target::target_of;
use crate::links::wiki_target::{WikiTarget, wiki_target};
use crate::project_index::ProjectIndex;

/// The warning's code.
pub(crate) const CODE: &str = "broken-link";

/// The document's broken links, in its order: its links, then its references without a definition.
pub(crate) fn broken_links(asked: &Asked) -> Vec<Diagnostic> {
    let Some(processor) = asked.processor else {
        return Vec::new();
    };
    let mut found: Vec<Diagnostic> = links(asked)
        .iter()
        .filter_map(|link| {
            let message = broken(link, asked)?;
            Some(warning(
                asked.encoding.span(asked.text, &link.position),
                message,
            ))
        })
        .collect();
    found.extend(
        processor
            .unresolved_references()
            .into_iter()
            .map(|reference| {
                let range = asked.encoding.span(asked.text, &reference.position);
                warning(
                    range,
                    format!("no definition defines the label `{}`", reference.label),
                )
            }),
    );
    found
}

/// What is broken about `link`, when something is: its file isn't there, or the heading its anchor names or its
/// passage isn't in it.
fn broken(link: &DocumentLink, asked: &Asked) -> Option<String> {
    if let LinkKind::Wiki { target } = &link.kind {
        return broken_wiki(target, asked);
    }
    let url = link.url()?;
    let target = target_of(url, asked.path, &root(asked))?;
    // The file as the link writes it, without its anchor.
    let written = url.split('#').next().unwrap_or(url);
    let file = match &target.file {
        Some(file) if file.as_path() != asked.path => {
            if !file.exists() {
                return Some(format!("no file `{written}` is in the project"));
            }
            file.clone()
        }
        _ => asked.path.to_path_buf(),
    };
    if let Some(passage) = target.passage {
        let text = text_of(&file, asked)?;
        if passage.find(&text).is_some() {
            return None;
        }
        let place = if file == asked.path {
            "the document".to_owned()
        } else {
            format!("`{written}`")
        };
        return Some(format!(
            "nothing in {place} matches the passage `{}`",
            passage.start()
        ));
    }
    let anchor = target.anchor?;
    let headings = headings_of(&file, asked)?;
    if headings.iter().any(|heading| heading.anchor == anchor) {
        return None;
    }
    Some(if file == asked.path {
        format!("the document has no heading `#{anchor}`")
    } else {
        format!("`{written}` has no heading `#{anchor}`")
    })
}

/// What is broken about a wiki link to `target`, when something is: no file has its page's name, or the heading it
/// names isn't in it.
fn broken_wiki(target: &str, asked: &Asked) -> Option<String> {
    let WikiTarget { page, heading } = wiki_target(target);
    let Some(file) = wiki_file_of(&page, asked.path, asked) else {
        let read = asked.index.is_some_and(ProjectIndex::is_read);
        return read.then(|| format!("no page `{page}` is in the project"));
    };
    let heading = heading?;
    headings_of(&file, asked)?;
    if wiki_heading(&heading, &file, asked).is_some() {
        return None;
    }
    Some(if file == asked.path {
        format!("the document has no heading `{heading}`")
    } else {
        format!("`{page}` has no heading `{heading}`")
    })
}

/// Whether `diagnostic` is a broken-link warning.
pub(crate) fn is_broken_link(diagnostic: &Diagnostic) -> bool {
    diagnostic.code == Some(NumberOrString::String(CODE.to_owned()))
}

/// The broken-link warning `message` at `range`.
fn warning(range: Range, message: String) -> Diagnostic {
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::WARNING),
        code: Some(NumberOrString::String(CODE.to_owned())),
        source: Some(SOURCE.to_owned()),
        message,
        ..Diagnostic::default()
    }
}

#[cfg(test)]
mod tests;
