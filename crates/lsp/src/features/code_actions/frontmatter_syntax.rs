//! `refactor.rewrite`: the frontmatter from YAML to TOML and back (spec §2.1 reads both alike), with its comments.
//! A value TOML can't hold (YAML's empty value) makes the action disabled, saying which.

use compmd::frontmatter::Syntax;
use lsp_types::{CodeAction, CodeActionDisabled, CodeActionKind, Range};

use super::frontmatter::{Frontmatter, Unwritable};
use super::{Asked, editing};

/// The action that writes the frontmatter in the other syntax, when `range` starts in it.
pub(super) fn convert(range: Range, asked: &Asked) -> Option<CodeAction> {
    let frontmatter = Frontmatter::of(asked)?;
    if !frontmatter.holds(range.start) {
        return None;
    }
    let (other, name) = match frontmatter.syntax {
        Syntax::Yaml => (Syntax::Toml, "TOML"),
        Syntax::Toml => (Syntax::Yaml, "YAML"),
    };
    let mut action = CodeAction {
        title: format!("Write the frontmatter in {name}"),
        kind: Some(CodeActionKind::REFACTOR_REWRITE),
        ..CodeAction::default()
    };
    match frontmatter.rewritten(other, &frontmatter.value) {
        Ok(edit) => action.edit = Some(editing(asked, vec![edit])),
        Err(Unwritable(pointer)) => {
            action.disabled = Some(CodeActionDisabled {
                reason: format!(
                    "{name} has no value for `{}`, which is empty or too large",
                    pointer.trim_start_matches('/').replace('/', ".")
                ),
            });
        }
    }
    Some(action)
}

#[cfg(test)]
mod tests;
