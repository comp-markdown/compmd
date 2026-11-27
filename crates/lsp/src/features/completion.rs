//! Completion ([the language server], *Completion*): the processor's hint's completions at a place, each replacing
//! the hint's range, of a kind by what the hint is about, a function's or a param's with its documentation.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#completion-hover-signature-help-go-to-definition

use compmd::hints::{Completion, PositionalHint, Subject};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionTextEdit, Documentation, MarkupContent,
    MarkupKind, Range, TextEdit,
};

use super::{Asked, import_edit, markdown};
use crate::pack_components::PackComponent;

/// The completions of `hint`, in the document `asked` is about; on a component's name, with the components the
/// project's packs and the suggested ones export that it can't name yet, and on a code block's language, with the
/// languages their code renderers render that none the document names does, each importing its pack.
pub(crate) fn completions(hint: &PositionalHint, asked: &Asked) -> Vec<CompletionItem> {
    let range = asked.encoding.range(asked.text, &hint.range);
    let mut items = hinted(hint, range);
    let offered = |label: &str| {
        hint.completions
            .iter()
            .any(|completion| completion.label == label)
    };
    let imported: Vec<(&str, &PackComponent)> = match hint.subject {
        Some(Subject::Component { .. }) => asked
            .pack_components
            .iter()
            .map(|component| (component.name.as_str(), component))
            .collect(),
        Some(Subject::Language { .. }) => asked
            .pack_components
            .iter()
            .flat_map(|component| {
                component
                    .languages()
                    .iter()
                    .map(move |language| (language.as_str(), component))
            })
            .collect(),
        _ => Vec::new(),
    };
    items.extend(
        imported
            .into_iter()
            .filter(|(label, _)| !offered(label))
            .filter_map(|(label, component)| importing(label, component, hint, asked, range)),
    );
    items
}

/// The completion writing `label` at `range`, which imports `component` from its pack; `None` when the document's
/// frontmatter can't take the import.
fn importing(
    label: &str,
    component: &PackComponent,
    hint: &PositionalHint,
    asked: &Asked,
    range: Range,
) -> Option<CompletionItem> {
    let processor = asked.processor?;
    let edit = import_edit::adding_import(
        asked.text,
        processor.tree(),
        &component.from,
        &component.name,
    )?;
    let (kind, detail) = if matches!(hint.subject, Some(Subject::Language { .. })) {
        (
            CompletionItemKind::VALUE,
            format!("imports `{}` from `{}`", component.name, component.from),
        )
    } else {
        (
            CompletionItemKind::CLASS,
            format!("imports it from `{}`", component.from),
        )
    };
    Some(CompletionItem {
        label: label.to_owned(),
        kind: Some(kind),
        detail: Some(detail),
        documentation: Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: markdown::pack_component(component),
        })),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range,
            new_text: label.to_owned(),
        })),
        additional_text_edits: Some(vec![edit]),
        ..CompletionItem::default()
    })
}

/// The hint's own completions, each replacing `range`.
fn hinted(hint: &PositionalHint, range: Range) -> Vec<CompletionItem> {
    hint.completions
        .iter()
        .map(|completion| {
            let (kind, documentation) = described(hint.subject.as_ref(), completion);
            CompletionItem {
                label: completion.label.clone(),
                kind: Some(kind),
                detail: completion.detail.clone(),
                documentation: documentation.map(|value| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value,
                    })
                }),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: completion.text.clone(),
                })),
                ..CompletionItem::default()
            }
        })
        .collect()
}

/// What `completion` is where `subject` is: its kind, and its documentation when there is more to say than its
/// detail.
fn described(
    subject: Option<&Subject>,
    completion: &Completion,
) -> (CompletionItemKind, Option<String>) {
    match subject {
        Some(Subject::Component { .. }) => (CompletionItemKind::CLASS, None),
        Some(Subject::Language { .. }) => (CompletionItemKind::VALUE, None),
        Some(Subject::Params { params, .. }) => {
            let param = params.iter().find(|param| param.name == completion.label);
            (CompletionItemKind::PROPERTY, param.map(markdown::param))
        }
        Some(Subject::Param { value: Some(_), .. }) => (CompletionItemKind::ENUM_MEMBER, None),
        Some(Subject::Param { .. } | Subject::Frontmatter { .. }) => {
            (CompletionItemKind::PROPERTY, None)
        }
        Some(Subject::Slot { .. }) => (CompletionItemKind::FIELD, None),
        Some(Subject::Type { .. }) => (CompletionItemKind::TYPE_PARAMETER, None),
        Some(Subject::Function { name, .. }) => {
            let prefix = if name.starts_with("md.") { "md." } else { "" };
            function(&format!("{prefix}{}", completion.label))
        }
        Some(Subject::Path { path, .. }) => {
            // A step after `md.` may be an `md.` function; after a value, a method on it.
            let prefix = if path.starts_with("md.") && path.matches('.').count() == 1 {
                "md."
            } else {
                ""
            };
            match function(&format!("{prefix}{}", completion.label)) {
                (_, None) => (CompletionItemKind::FIELD, None),
                found => found,
            }
        }
        None => (CompletionItemKind::TEXT, None),
    }
}

/// The function written `name`, with its documentation, when the catalog has it.
fn function(name: &str) -> (CompletionItemKind, Option<String>) {
    let documentation = markdown::function_named(name).map(markdown::function);
    (CompletionItemKind::FUNCTION, documentation)
}

#[cfg(test)]
mod tests;
