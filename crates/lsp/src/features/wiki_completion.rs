//! Completion of wiki links ([the language server], *Links and headings*, `links.completion` and `links.wiki`): after
//! `[[`, the project's pages by name, each once; after `#`, the headings of the page before it, or of the document,
//! by their text. Only in a document that has wiki links.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use std::path::PathBuf;

use lsp_types::{CompletionItem, CompletionItemKind, Position, Range};

use super::Asked;
use super::link_completion::{item, written_range};
use super::link_targets::{headings_of, root, wiki_file_of};
use crate::links::wiki_target::page_name;

/// The completions of a wiki link's target, `written` so far up to `position`; `None` where the document has no wiki
/// links, or while `links.wiki` is off.
pub(super) fn wiki_completions(
    written: &str,
    position: Position,
    asked: &Asked,
) -> Option<Vec<CompletionItem>> {
    if !asked.links.follows_wiki() || !asked.processor?.has_wiki_links() {
        return None;
    }
    Some(match written.split_once('#') {
        Some((page, heading)) => {
            headings(page.trim(), written_range(heading, position, asked), asked)
        }
        None => pages(written_range(written, position, asked), asked),
    })
}

/// The names of the project's files but the document, each once, with the path of the first.
fn pages(range: Range, asked: &Asked) -> Vec<CompletionItem> {
    let Some(index) = asked.index else {
        return Vec::new();
    };
    let root = root(asked);
    let files: Vec<PathBuf> = index
        .files()
        .keys()
        .filter(|file| *file != asked.path)
        .cloned()
        .collect();
    let mut items: Vec<CompletionItem> = Vec::new();
    for file in files {
        let name = page_name(&file);
        if items.iter().any(|item| item.label == name) {
            continue;
        }
        let path = file
            .strip_prefix(&root)
            .unwrap_or(&file)
            .display()
            .to_string();
        items.push(item(name, CompletionItemKind::FILE, Some(path), range));
    }
    items
}

/// The headings of the page `page` names, or of the document's when it is empty, by their text.
fn headings(page: &str, range: Range, asked: &Asked) -> Vec<CompletionItem> {
    wiki_file_of(page, asked.path, asked)
        .and_then(|file| headings_of(&file, asked))
        .unwrap_or_default()
        .into_iter()
        .map(|heading| {
            let detail = format!(
                "{} {}",
                "#".repeat(usize::from(heading.depth)),
                heading.text
            );
            item(
                heading.text,
                CompletionItemKind::REFERENCE,
                Some(detail),
                range,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests;
