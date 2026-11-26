//! The quick fix of an import of one of CompMD's own packs its site can't load ([the language server], decision 4 and
//! Q4): adding their registry to the configuration.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use lsp_types::{CodeAction, CodeActionKind, Range};

use super::Asked;
use crate::own_packs::Unregistered;

/// The fix of each such import's warning that meets `range`.
pub(super) fn fixes(range: Range, asked: &Asked) -> Vec<CodeAction> {
    let Some(Unregistered {
        add: Some(add),
        warnings,
    }) = asked.unregistered
    else {
        return Vec::new();
    };
    warnings
        .iter()
        .filter(|warning| warning.range.start <= range.end && range.start <= warning.range.end)
        .map(|warning| CodeAction {
            title: add.title(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: Some(vec![warning.clone()]),
            edit: Some(add.edit.clone()),
            is_preferred: Some(true),
            ..CodeAction::default()
        })
        .collect()
}
