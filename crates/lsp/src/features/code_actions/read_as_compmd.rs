//! *Read as CompMD*, on a `.md` file without an `md` key, which is plain CommonMark and GFM ([a document drawn in the
//! editor], decision 13; spec section 2, which lets a processor offer it): the key added, `md.version: 1`, at the end
//! of the frontmatter in its own syntax, or in a new YAML frontmatter.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::frontmatter::Syntax;
use compmd::tree::Node;
use lsp_types::{CodeAction, CodeActionKind, Position, Range, TextEdit};

use super::frontmatter::Frontmatter;
use super::{Asked, editing};

/// The action on a plain Markdown file, when its frontmatter, if it has one, reads.
pub(super) fn read_as_compmd(asked: &Asked) -> Option<CodeAction> {
    let processor = asked.processor?;
    let (line, text) = match Frontmatter::of(asked) {
        Some(frontmatter) => (
            frontmatter.closing_line(),
            match frontmatter.syntax {
                Syntax::Yaml => "md:\n  version: 1\n",
                Syntax::Toml => "[md]\nversion = 1\n",
            },
        ),
        None if matches!(
            processor.tree().children.first(),
            Some(Node::Yaml(_) | Node::Toml(_))
        ) =>
        {
            return None;
        }
        None => (0, "---\nmd:\n  version: 1\n---\n\n"),
    };
    let at = Position { line, character: 0 };
    let edit = TextEdit {
        range: Range { start: at, end: at },
        new_text: text.to_owned(),
    };
    Some(CodeAction {
        title: "Read as CompMD".to_owned(),
        kind: Some(CodeActionKind::REFACTOR_REWRITE),
        edit: Some(editing(asked, vec![edit])),
        ..CodeAction::default()
    })
}

#[cfg(test)]
mod tests;
