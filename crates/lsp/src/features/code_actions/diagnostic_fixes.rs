//! The quick fix of a diagnostic that suggests one (its `fix`, spec §10.5): a "did you mean" made.

use lsp_types::{CodeAction, Range, TextEdit};

use super::{Asked, diagnostics_at, quick_fix};

/// The fixes of the diagnostics at `range` that have one.
pub(super) fn fixes(range: Range, asked: &Asked) -> Vec<CodeAction> {
    diagnostics_at(range, asked)
        .filter_map(|diagnostic| {
            let fix = diagnostic.fix.as_ref()?;
            let edit = TextEdit {
                range: asked.encoding.range(asked.text, &fix.range),
                new_text: fix.text.clone(),
            };
            let mut action =
                quick_fix(format!("Change to `{}`", fix.text), diagnostic, edit, asked);
            action.is_preferred = Some(true);
            Some(action)
        })
        .collect()
}
