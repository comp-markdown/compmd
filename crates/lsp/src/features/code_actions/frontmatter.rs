//! The frontmatter rewritten (spec §2.1): read as written, changed, and written again in YAML or TOML, with the
//! comments on lines of their own kept above their keys. The organize actions and the change of syntax share it; the
//! frontmatter they write is laid out as these writers lay it out.

mod comments;
#[cfg(test)]
mod testing;
mod toml_writer;
mod yaml_writer;

use std::collections::HashMap;

use compmd::frontmatter::{Syntax, Written};
use compmd::tree::Node;
use lsp_types::{Position, Range, TextEdit};
use serde_json::{Map, Value};

use super::Asked;
use comments::Comments;
pub(super) use toml_writer::Unwritable;

/// A document's frontmatter, as written, with its comments and where it is.
pub(super) struct Frontmatter {
    /// What it is written in.
    pub(super) syntax: Syntax,
    /// Its value, each mapping's keys in written order.
    pub(super) value: Map<String, Value>,
    /// The line each key is on, from 0 as the client counts, by JSON pointer.
    pub(super) key_lines: HashMap<String, u32>,
    comments: Comments,
    /// The whole of it, with its fences.
    range: Range,
}

impl Frontmatter {
    /// The frontmatter of the document `asked` is about; `None` when it has none, or one that isn't a mapping or
    /// doesn't read.
    pub(super) fn of(asked: &Asked) -> Option<Self> {
        let position = match asked.processor?.tree().children.first()? {
            Node::Yaml(frontmatter) | Node::Toml(frontmatter) => frontmatter.position?,
            _ => return None,
        };
        let written = Written::of(asked.text)?;
        let comments = Comments::of(
            asked.text,
            &written,
            position.start.line + 1,
            position.end.line - 1,
        );
        let Value::Object(value) = written.value else {
            return None;
        };
        let key_lines = written
            .key_lines
            .into_iter()
            .filter_map(|(pointer, line)| Some((pointer, u32::try_from(line - 1).ok()?)))
            .collect();
        Some(Self {
            syntax: written.syntax,
            value,
            key_lines,
            comments,
            range: asked.encoding.span(asked.text, &position),
        })
    }

    /// Whether `position` is in it, its fences included.
    pub(super) fn holds(&self, position: Position) -> bool {
        self.range.start <= position && position <= self.range.end
    }

    /// The first line of its text, after its opening fence.
    pub(super) fn first_line(&self) -> u32 {
        self.range.start.line + 1
    }

    /// The line of its closing fence.
    pub(super) fn closing_line(&self) -> u32 {
        self.range.end.line
    }

    /// The edit that writes `value` in `syntax` in its place, with its comments; an error for a value `syntax` can't
    /// hold.
    pub(super) fn rewritten(
        &self,
        syntax: Syntax,
        value: &Map<String, Value>,
    ) -> Result<TextEdit, Unwritable> {
        let (fence, text) = match syntax {
            Syntax::Yaml => ("---", yaml_writer::write(value, &self.comments)),
            Syntax::Toml => ("+++", toml_writer::write(value, &self.comments)?),
        };
        Ok(TextEdit {
            range: self.range,
            new_text: format!("{fence}\n{text}{fence}"),
        })
    }
}

#[cfg(test)]
mod tests;
