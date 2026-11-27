//! Every place a document reads a metadata path, `md.doc.due_date` (spec §8.1), and the edits that rename its key: for
//! a host that lets a writer rename a field its documents read, and for an editor's references and rename. The reads
//! are found in the document's expressions as the processor parses them ([`crate::written_expressions`]), so text that
//! only looks like one, in a code block, in prose or inside a CEL string, isn't one.

use std::fmt;

use crate::cel::Program;
use crate::parse::TextEdit;
use crate::resolve::Document;
use crate::written_expressions::{self, WrittenExpression};

/// A metadata path's key and the path it is under: `md.doc.due_date` is the key `due_date` under `md.doc`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataPath {
    parent: String,
    key: String,
}

impl MetadataPath {
    /// The path `path`: `md`, then two names or more, each a CEL identifier (`md.doc.due_date`, `md.input.week`,
    /// `md.data.sales`, `md.doc.address.city`).
    ///
    /// # Errors
    ///
    /// [`PathError::NotAPath`] for anything else.
    pub fn new(path: &str) -> Result<Self, PathError> {
        let segments: Vec<&str> = path.split('.').collect();
        let valid = segments.len() >= 3
            && segments[0] == "md"
            && segments.iter().all(|segment| is_name(segment));
        match segments.split_last() {
            Some((key, parent)) if valid => Ok(Self {
                parent: parent.join("."),
                key: (*key).to_string(),
            }),
            _ => Err(PathError::NotAPath(path.to_string())),
        }
    }

    /// The path it is under: `md.doc`.
    #[must_use]
    pub fn parent(&self) -> &str {
        &self.parent
    }

    /// Its key: `due_date`.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }
}

/// Why a path or a key can't be found or renamed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathError {
    /// Not a metadata path: `md`, then two names or more, each a CEL identifier.
    NotAPath(String),
    /// Not a key an expression can select by dot: a name the CEL parser reads after a `.`.
    NotAKey(String),
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAPath(path) => {
                write!(f, "`{path}` isn't a metadata path such as `md.doc.title`")
            }
            Self::NotAKey(key) => write!(
                f,
                "`{key}` isn't a key an expression can select, such as `due`"
            ),
        }
    }
}

impl std::error::Error for PathError {}

/// Where a key is read, in code points of the document's text, as the trees and [`TextEdit`] count them: the first
/// code point and the one after the last.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyRead {
    /// The first code point.
    pub start: usize,
    /// The code point after the last.
    pub end: usize,
}

/// Every place a document reads a path's key.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PathReferences {
    /// The reads of the key, each where its name is written: a selection by dot (`md.doc.due_date`, in
    /// `has(md.doc.due_date)` too), or the text of a string literal indexing the path (`md.doc["due_date"]`). In the
    /// order they are written.
    pub reads: Vec<KeyRead>,
    /// What may read the key and no edit can rename: an index of the path whose key is computed when it runs
    /// (`md.doc[name]`), at its `[`, and a string literal naming the key with escapes, at the literal.
    pub unrewritable: Vec<KeyRead>,
}

/// The edits that rename a path's key, and what may read it that they can't rename.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyRename {
    /// The edits, each over a read's name only, in the order they are written; applied from the last, or all at once.
    pub edits: Vec<TextEdit>,
    /// As [`PathReferences::unrewritable`]: left as they are, for the caller to report.
    pub unrewritable: Vec<KeyRead>,
}

/// Every place `document` reads `path`'s key.
pub(crate) fn find_references(document: &Document, path: &MetadataPath) -> PathReferences {
    let mut found = PathReferences::default();
    for expression in written_expressions::expressions_of(document) {
        reads_in(&expression, path, &mut found);
    }
    found
}

/// Whether `key` is one an expression can select by dot, so a rename may write it.
///
/// # Errors
///
/// [`PathError::NotAKey`] when it isn't.
pub(crate) fn check_key(key: &str) -> Result<(), PathError> {
    if is_key(key) {
        Ok(())
    } else {
        Err(PathError::NotAKey(key.to_string()))
    }
}

/// The edits that rename `path`'s key in `document` to `key`, a key [`check_key`] admits.
pub(crate) fn rename_references(document: &Document, path: &MetadataPath, key: &str) -> KeyRename {
    let PathReferences {
        reads,
        unrewritable,
    } = find_references(document, path);
    let edits = reads
        .into_iter()
        .map(|read| TextEdit {
            start: read.start,
            end: read.end,
            text: key.to_string(),
        })
        .collect();
    KeyRename {
        edits,
        unrewritable,
    }
}

/// The reads of `path`'s key in `expression`, added to `found`. An expression that doesn't parse reads nothing.
fn reads_in(expression: &WrittenExpression, path: &MetadataPath, found: &mut PathReferences) {
    let Ok(program) = Program::parse(&expression.source) else {
        return;
    };
    let source: Vec<char> = expression.source.chars().collect();
    let at = |range: (usize, usize)| KeyRead {
        start: expression.start + range.0,
        end: expression.start + range.1,
    };
    for selection in program.selections(&path.parent) {
        if selection.name == path.key {
            found
                .reads
                .push(at((selection.range.start, selection.range.end)));
        }
    }
    for indexing in program.indexings(&path.parent) {
        match indexing.key {
            Some(key) if key == path.key => match literal_text(&source, indexing.at, &key) {
                Some(text) => found.reads.push(at(text)),
                None => found.unrewritable.push(at((indexing.at, indexing.at + 1))),
            },
            Some(_) => {}
            None => found.unrewritable.push(at((indexing.at, indexing.at + 1))),
        }
    }
    found.reads.sort();
    found.unrewritable.sort();
}

/// Where the text of the string literal at `at` of `source` is, when it is `key` as written (no escapes): past its
/// prefix (`r`) and its quotes, single or triple.
fn literal_text(source: &[char], at: usize, key: &str) -> Option<(usize, usize)> {
    let mut start = at
        + source[at..]
            .iter()
            .take_while(|c| matches!(c, 'r' | 'R'))
            .count();
    let quote = *source.get(start).filter(|c| matches!(c, '\'' | '"'))?;
    let quotes = if source[start..].starts_with(&[quote; 3]) {
        3
    } else {
        1
    };
    start += quotes;
    let end = start + key.chars().count();
    let written: String = source.get(start..end)?.iter().collect();
    let closed = source.get(end..end + quotes) == Some(&vec![quote; quotes][..]);
    (written == key && closed).then_some((start, end))
}

/// Whether `name` is a CEL identifier: a letter or `_`, then letters, digits and `_`.
fn is_name(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Whether an expression can select `key` by dot: a CEL identifier its parser reads as a field's name.
fn is_key(key: &str) -> bool {
    is_name(key)
        && Program::parse(&format!("x.{key}")).is_ok_and(|program| {
            program
                .selections("x")
                .first()
                .is_some_and(|selection| selection.name == key)
        })
}

#[cfg(test)]
mod tests;
