//! The quick fix of an unknown component (F002) that one of the project's packs exports: the import that names it.

use compmd::Processor;
use compmd::diagnostic::{Code, Diagnostic};
use compmd::hints::Subject;
use compmd::processor::Expanded;
use lsp_types::{CodeAction, Range};

use super::{Asked, diagnostics_at, quick_fix};
use crate::features::import_edit::adding_import;

/// When `range` starts on a name that stands for no component, the imports of the packs that export one of that name.
pub(super) fn imports(range: Range, asked: &Asked) -> Vec<CodeAction> {
    let Some((name, unknown)) = unknown_at(range, asked) else {
        return Vec::new();
    };
    let tree = asked.processor.map(Processor::<Expanded>::tree);
    asked
        .pack_components
        .iter()
        .filter(|component| component.name == name)
        .filter_map(|component| {
            let edit = adding_import(asked.text, tree?, &component.from, &name)?;
            let title = format!("Import `{name}` from `{}`", component.from);
            Some(quick_fix(title, unknown, edit, asked))
        })
        .collect()
}

/// The name `range` starts on, when it stands for no component, with its F002.
fn unknown_at<'a>(range: Range, asked: &'a Asked) -> Option<(String, &'a Diagnostic)> {
    let hint = asked.processor?.hint_at(asked.offset(range.start))?;
    let Some(Subject::Component {
        name,
        component: None,
    }) = hint.subject
    else {
        return None;
    };
    let name_range = asked.encoding.range(asked.text, &hint.range);
    let unknown =
        diagnostics_at(name_range, asked).find(|diagnostic| diagnostic.code == Code::F002)?;
    Some((name, unknown))
}
