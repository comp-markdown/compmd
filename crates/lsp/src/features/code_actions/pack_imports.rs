//! The quick fix of an unknown component (F002) that one of the project's packs exports: the import that names it.

use compmd::diagnostic::Code;
use compmd::hints::Subject;
use lsp_types::{CodeAction, Range};

use super::{Asked, diagnostics_at, quick_fix};
use crate::features::import_edit::adding_import;

/// When `range` starts on a name that stands for no component, the imports of the packs that export one of that name.
pub(super) fn imports(range: Range, asked: &Asked) -> Vec<CodeAction> {
    let Some(processor) = asked.processor else {
        return Vec::new();
    };
    let Some(hint) = processor.hint_at(asked.offset(range.start)) else {
        return Vec::new();
    };
    let Some(Subject::Component {
        name,
        component: None,
    }) = hint.subject
    else {
        return Vec::new();
    };
    let name_range = asked.encoding.range(asked.text, &hint.range);
    let Some(unknown) =
        diagnostics_at(name_range, asked).find(|diagnostic| diagnostic.code == Code::F002)
    else {
        return Vec::new();
    };
    asked
        .pack_components
        .iter()
        .filter(|component| component.name == name)
        .filter_map(|component| {
            let edit = adding_import(
                asked.text,
                processor.tree(),
                &component.from,
                &component.name,
            )?;
            let title = format!("Import `{}` from `{}`", component.name, component.from);
            Some(quick_fix(title, unknown, edit, asked))
        })
        .collect()
}
