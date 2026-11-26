use lsp_types::{CompletionItemKind, CompletionTextEdit, Documentation};

use super::*;
use crate::features::testing::{asking, caret, processed};
use crate::pack_components::PackComponent;

/// The completions at the caret `|` of `marked`, the project's packs exporting `components`.
fn completed_with(marked: &str, components: &[PackComponent]) -> Vec<CompletionItem> {
    let (text, at) = caret(marked);
    let processor = processed(&text);
    let Some(hint) = processor.hint_at(at) else {
        return Vec::new();
    };
    asking(&text, &processor, components, |asked| {
        completions(&hint, asked)
    })
}

/// The completions at the caret `|` of `marked`.
fn completed(marked: &str) -> Vec<CompletionItem> {
    completed_with(marked, &[])
}

/// The completion labelled `label` at the caret of `marked`.
fn item(marked: &str, label: &str) -> CompletionItem {
    completed(marked)
        .into_iter()
        .find(|item| item.label == label)
        .unwrap_or_else(|| panic!("`{label}` at {marked:?}"))
}

fn documentation(item: &CompletionItem) -> String {
    match &item.documentation {
        Some(Documentation::MarkupContent(markup)) => markup.value.clone(),
        _ => String::new(),
    }
}

#[test]
fn a_component_name_replaces_what_is_written() {
    let card = item("::: bo|\n", "box");
    assert_eq!(card.kind, Some(CompletionItemKind::CLASS));
    let Some(CompletionTextEdit::Edit(edit)) = card.text_edit else {
        panic!("an edit");
    };
    assert_eq!(
        (
            edit.range.start.character,
            edit.range.end.character,
            edit.new_text.as_str()
        ),
        (4, 6, "box")
    );
}

#[test]
fn a_packs_component_is_offered_with_its_import_unless_the_document_names_it() {
    let notes = [
        PackComponent {
            name: "note".into(),
            from: "@site/components@1".into(),
            description: Some("The site's own.".into()),
            ..PackComponent::default()
        },
        PackComponent {
            name: "box".into(),
            from: "@site/components@1".into(),
            ..PackComponent::default()
        },
    ];
    let items = completed_with("::: no|\n", &notes);
    let note = items
        .iter()
        .find(|item| item.label == "note")
        .expect("the pack's `note`");
    assert_eq!(
        note.detail.as_deref(),
        Some("imports it from `@site/components@1`")
    );
    let Some(Documentation::MarkupContent(documentation)) = &note.documentation else {
        panic!("its documentation");
    };
    assert_eq!(
        documentation.value,
        "**`note`**, from `@site/components@1`\n\nThe site's own."
    );
    let edit = &note.additional_text_edits.as_ref().expect("its import")[0];
    assert!(
        edit.new_text
            .contains("- from: \"@site/components@1\"\n      names: [note]"),
        "{}",
        edit.new_text
    );
    assert_eq!(
        items.iter().filter(|item| item.label == "box").count(),
        1,
        "`box` is core's already"
    );
    let flow = completed_with("---\nmd: {lang: en}\n---\n::: no|\n", &notes);
    assert!(
        flow.iter().all(|item| item.label != "note"),
        "no edit to make"
    );
    assert!(
        completed_with("::: box {bor|}\n::: /box\n", &notes)
            .iter()
            .all(|item| item.label != "note"),
        "not a name"
    );
}

#[test]
fn params_with_their_documentation_and_values() {
    let border = item("::: box {|}\n::: /box\n", "border");
    assert_eq!(border.kind, Some(CompletionItemKind::PROPERTY));
    assert!(
        documentation(&border).starts_with("`border`: "),
        "{border:?}"
    );
    assert_eq!(
        item("::: box {bor|}\n::: /box\n", "border").kind,
        Some(CompletionItemKind::PROPERTY)
    );
    assert_eq!(
        item("::: box {border=s|}\n::: /box\n", "subtle").kind,
        Some(CompletionItemKind::ENUM_MEMBER)
    );
}

#[test]
fn paths_and_functions_with_the_catalogs_documentation() {
    let doc = item("{{ md.d| }}\n", "doc");
    assert_eq!(
        (doc.kind, doc.documentation),
        (Some(CompletionItemKind::FIELD), None)
    );
    let number = item("::: box {padding=(md.n|)}\n::: /box\n", "number");
    assert_eq!(number.kind, Some(CompletionItemKind::FUNCTION));
    assert!(documentation(&number).starts_with("```cel\n"), "{number:?}");
    let called = item("::: box {padding=(md.numb|er(1))}\n::: /box\n", "number");
    assert!(documentation(&called).contains("(spec §8.2)"));
    let method = item(
        "---\ntitle: Notes\n---\n\n::: box {padding=(md.doc.title.s|)}\n::: /box\n",
        "size",
    );
    assert_eq!(method.kind, Some(CompletionItemKind::FUNCTION));
}

#[test]
fn slots_frontmatter_keys_and_types() {
    assert!(
        completed("---\nmd:\n  la|\n---\n")
            .iter()
            .all(|item| item.kind == Some(CompletionItemKind::PROPERTY))
    );
    let types = completed("---\nmd:\n  inputs:\n    n: num|\n---\n");
    assert!(types.iter().any(
        |item| item.label == "number" && item.kind == Some(CompletionItemKind::TYPE_PARAMETER)
    ));
    let slots = completed("::: box\nA.\n\n:: #f|\nB.\n::: /box\n");
    assert!(
        slots
            .iter()
            .all(|item| item.kind == Some(CompletionItemKind::FIELD))
    );
    assert_eq!(completed("Just te|xt.\n"), Vec::new());
}
