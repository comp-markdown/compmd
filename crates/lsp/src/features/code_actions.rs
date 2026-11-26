//! Code actions ([the language server], *Templates, code actions, refactorings*): what the server offers to do at a
//! range, each of a kind an editor groups, and only those of the kinds the client asks for.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#templates-code-actions-refactorings-in-1

mod diagnostic_fixes;
mod frontmatter;
mod frontmatter_keys;
mod frontmatter_syntax;
mod organize_frontmatter;
mod organize_imports;
mod pack_imports;
mod registry_fix;
mod signatures;

use std::collections::HashMap;

use compmd::diagnostic::Diagnostic;
use lsp_types::{CodeAction, CodeActionKind, Range, TextEdit, WorkspaceEdit};

use super::Asked;
use crate::diagnostics::lsp_diagnostic;

/// The kinds of the actions the server offers.
pub(crate) fn kinds() -> Vec<CodeActionKind> {
    vec![
        CodeActionKind::QUICKFIX,
        CodeActionKind::SOURCE_ORGANIZE_IMPORTS,
        CodeActionKind::new(organize_frontmatter::KIND),
        CodeActionKind::new(frontmatter_keys::KIND),
        CodeActionKind::REFACTOR_REWRITE,
    ]
}

/// The actions at `range` of the document `asked` is about, of the kinds `only` names (any kind when `None`).
pub(crate) fn code_actions(
    range: Range,
    only: Option<&[CodeActionKind]>,
    asked: &Asked,
) -> Vec<CodeAction> {
    let mut actions = diagnostic_fixes::fixes(range, asked);
    actions.extend(registry_fix::fixes(range, asked));
    actions.extend(pack_imports::imports(range, asked));
    actions.extend(organize_imports::organize(asked));
    actions.extend(organize_frontmatter::organize(asked));
    actions.extend(frontmatter_keys::offered(range, asked));
    actions.extend(frontmatter_syntax::convert(range, asked));
    actions.extend(signatures::signature_action(range, asked));
    actions
        .into_iter()
        .filter(|action| only.is_none_or(|only| asked_for(action, only)))
        .collect()
}

/// Whether `action`'s kind is one of `only`, or under one (`source.organizeImports` is under `source`).
fn asked_for(action: &CodeAction, only: &[CodeActionKind]) -> bool {
    let Some(kind) = &action.kind else {
        return false;
    };
    only.iter().any(|asked| {
        kind.as_str() == asked.as_str()
            || kind
                .as_str()
                .strip_prefix(asked.as_str())
                .is_some_and(|rest| rest.starts_with('.'))
    })
}

/// The diagnostics of the document itself (not of one it imports) that meet `range`: their own range, or the text
/// their fix replaces, which a diagnostic at a point (a name's start) is about.
fn diagnostics_at<'a>(range: Range, asked: &'a Asked) -> impl Iterator<Item = &'a Diagnostic> + 'a {
    let meets = move |at: &compmd::diagnostic::Range| {
        let at = asked.encoding.range(asked.text, at);
        at.start <= range.end && range.start <= at.end
    };
    asked
        .processor
        .into_iter()
        .flat_map(|processor| processor.diagnostics().iter())
        .filter(|diagnostic| diagnostic.file.is_none())
        .filter(move |diagnostic| {
            meets(&diagnostic.range) || diagnostic.fix.as_ref().is_some_and(|fix| meets(&fix.range))
        })
}

/// A quick fix titled `title`, fixing `diagnostic` with `edit`.
fn quick_fix(title: String, diagnostic: &Diagnostic, edit: TextEdit, asked: &Asked) -> CodeAction {
    CodeAction {
        title,
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![lsp_diagnostic(
            diagnostic,
            asked.uri,
            asked.text,
            asked.encoding,
        )]),
        edit: Some(editing(asked, vec![edit])),
        ..CodeAction::default()
    }
}

/// The workspace edit that makes `edits` to the document `asked` is about.
fn editing(asked: &Asked, edits: Vec<TextEdit>) -> WorkspaceEdit {
    WorkspaceEdit {
        changes: Some(HashMap::from([(asked.uri.clone(), edits)])),
        ..WorkspaceEdit::default()
    }
}

#[cfg(test)]
mod tests;
