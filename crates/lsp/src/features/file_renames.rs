//! A file or a folder renamed or moved (`workspace/willRenameFiles`, [the language server], *Links and headings*,
//! `links.rename`): every indexed file's links to it given its new path, and a moved file's own relative links
//! written again from its new folder. A link from the root (`/notes/tides.md`) stays one; a `./` the link wrote
//! stays. The edits are of the files as they are before the rename.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use lsp_types::{TextEdit, WorkspaceEdit};

use super::link_targets::uri_of;
use crate::encoding::Encoding;
use crate::files::OpenTexts;
use crate::links::document_links::DocumentLink;
use crate::links::link_parts::path_span;
use crate::links::relative_path::relative_path;
use crate::links::target::target_of;
use crate::links::text_span::{range_of, spanned};
use crate::project_index::ProjectIndex;

/// A rename: a file's or a folder's path before it, and after.
pub(crate) type Moved = (PathBuf, PathBuf);

/// The edits `renames` need across `indexes`, the open files' texts in `open`.
pub(crate) fn file_renames<'a>(
    renames: &[Moved],
    indexes: impl IntoIterator<Item = &'a ProjectIndex>,
    open: &OpenTexts,
    encoding: Encoding,
) -> WorkspaceEdit {
    // By file: one in two roots' indexes (a root inside another) is edited once.
    let mut by_file: BTreeMap<PathBuf, Vec<TextEdit>> = BTreeMap::new();
    for index in indexes {
        let files: Vec<(PathBuf, Vec<DocumentLink>)> = index
            .files()
            .iter()
            .map(|(file, indexed)| (file.clone(), indexed.links.clone()))
            .collect();
        for (file, links) in files {
            let Some(text) = open.text(&file).or_else(|| fs::read_to_string(&file).ok()) else {
                continue;
            };
            let edits: Vec<TextEdit> = links
                .iter()
                .filter_map(|link| {
                    let (span, new_path) = rewritten(link, &file, &text, (index.root(), renames))?;
                    Some(TextEdit {
                        range: range_of(&text, encoding, span),
                        new_text: new_path,
                    })
                })
                .collect();
            if !edits.is_empty() {
                by_file.insert(file, edits);
            }
        }
    }
    WorkspaceEdit {
        changes: Some(
            by_file
                .into_iter()
                .filter_map(|(file, edits)| Some((uri_of(&file)?, edits)))
                .collect(),
        ),
        ..WorkspaceEdit::default()
    }
}

/// Where `link`'s path is in `text`, `file`'s, and the path it takes after `renames` under `root`; `None` when it
/// stays.
fn rewritten(
    link: &DocumentLink,
    file: &Path,
    text: &str,
    (root, renames): (&Path, &[Moved]),
) -> Option<((usize, usize), String)> {
    let url = link.url()?;
    let target = target_of(url, file, root)?.file?;
    let moved_target = moved(&target, renames);
    let moved_file = moved(file, renames);
    let from_root = url.starts_with('/');
    if moved_target.is_none() && (moved_file.is_none() || from_root) {
        return None;
    }
    let target = moved_target.unwrap_or(target);
    let path = if from_root {
        format!("/{}", relative_path(&target, root))
    } else {
        let file = moved_file.unwrap_or_else(|| file.to_path_buf());
        let folder = file.parent().unwrap_or(root);
        let path = relative_path(&target, folder);
        if url.starts_with("./") && !path.starts_with("../") {
            format!("./{path}")
        } else {
            path
        }
    };
    let written = spanned(text, &link.position);
    let (from, to) = path_span(&written, link)?;
    let unchanged: String = written.chars().skip(from).take(to - from).collect();
    let start = link.position.start.offset;
    (unchanged != path).then_some(((start + from, start + to), path))
}

/// Where `path` is after `renames`: the new path of it or of the folder it is in; `None` when it stays.
fn moved(path: &Path, renames: &[Moved]) -> Option<PathBuf> {
    renames.iter().find_map(|(old, new)| {
        path.strip_prefix(old).ok().map(|inside| {
            if inside.as_os_str().is_empty() {
                new.clone()
            } else {
                new.join(inside)
            }
        })
    })
}

#[cfg(test)]
mod tests;
