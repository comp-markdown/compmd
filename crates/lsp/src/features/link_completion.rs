//! Completion of links ([the language server], *Links and headings*, `links.completion`): after `](` and in a reference
//! definition's target, the project's files relative to the document; after `#`, the anchors of the file before it, or
//! of the document; after `[text][`, the document's reference labels. Each replaces what is written of its part.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use std::path::Path;

use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionTextEdit, Position, Range, TextEdit,
};

use super::Asked;
use super::link_targets::{headings_of, links, root};
use crate::links::document_links::LinkKind;
use crate::links::relative_path::relative_path;
use crate::links::target::target_of;
use crate::links::typed_link::{TypedLink, typed_link};

/// The completions of the link being typed at `position`; `None` when no link is.
pub(super) fn link_completions(position: Position, asked: &Asked) -> Option<Vec<CompletionItem>> {
    let start = asked.offset(Position {
        line: position.line,
        character: 0,
    });
    let before: String = asked
        .text
        .chars()
        .skip(start)
        .take(asked.offset(position) - start)
        .collect();
    let items = match typed_link(&before)? {
        TypedLink::Target(written) => match written.split_once('#') {
            Some((path, anchor)) => anchors(path, written_range(anchor, position, asked), asked),
            None => files(written_range(&written, position, asked), asked),
        },
        TypedLink::Label(written) => labels(written_range(&written, position, asked), asked),
    };
    Some(items)
}

/// The range of `written`, which ends at `position`.
fn written_range(written: &str, position: Position, asked: &Asked) -> Range {
    let start = Position {
        line: position.line,
        character: position
            .character
            .saturating_sub(asked.encoding.units(written)),
    };
    Range {
        start,
        end: position,
    }
}

/// The project's indexed files but the document, each written relative to the document.
fn files(range: Range, asked: &Asked) -> Vec<CompletionItem> {
    let Some(index) = asked.index else {
        return Vec::new();
    };
    let folder = asked.path.parent().unwrap_or_else(|| Path::new(""));
    index
        .files()
        .keys()
        .filter(|file| *file != asked.path)
        .map(|file| {
            item(
                relative_path(file, folder),
                CompletionItemKind::FILE,
                None,
                range,
            )
        })
        .collect()
}

/// The anchors of the headings of the file `path` names, or of the document's when it is empty.
fn anchors(path: &str, range: Range, asked: &Asked) -> Vec<CompletionItem> {
    let file = if path.is_empty() {
        Some(asked.path.to_path_buf())
    } else {
        target_of(path, asked.path, &root(asked)).and_then(|target| target.file)
    };
    file.and_then(|file| headings_of(&file, asked))
        .unwrap_or_default()
        .into_iter()
        .map(|heading| {
            let detail = format!(
                "{} {}",
                "#".repeat(usize::from(heading.depth)),
                heading.text
            );
            item(
                heading.anchor,
                CompletionItemKind::REFERENCE,
                Some(detail),
                range,
            )
        })
        .collect()
}

/// The labels the document's reference definitions define, each once, with its destination.
fn labels(range: Range, asked: &Asked) -> Vec<CompletionItem> {
    let mut items: Vec<CompletionItem> = Vec::new();
    for link in links(asked) {
        if let LinkKind::Definition { identifier, url } = link.kind
            && !items.iter().any(|item| item.label == identifier)
        {
            items.push(item(
                identifier,
                CompletionItemKind::REFERENCE,
                Some(url),
                range,
            ));
        }
    }
    items
}

/// The completion writing `label` over `range`.
fn item(
    label: String,
    kind: CompletionItemKind,
    detail: Option<String>,
    range: Range,
) -> CompletionItem {
    CompletionItem {
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range,
            new_text: label.clone(),
        })),
        label,
        kind: Some(kind),
        detail,
        ..CompletionItem::default()
    }
}

#[cfg(test)]
mod tests;
