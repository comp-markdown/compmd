//! The actions' tests: what they share, and the quick fixes'.

use lsp_types::{CodeAction, CodeActionKind, Position, Range, TextEdit};

use super::code_actions;
use crate::features::testing::{asking, processed};
use crate::pack_components::PackComponent;

/// The actions at `range` of `text`, with the packs' `components`, of the kinds `only` names.
pub(super) fn actions(
    text: &str,
    range: Range,
    components: &[PackComponent],
    only: Option<&[CodeActionKind]>,
) -> Vec<CodeAction> {
    let processor = processed(text);
    asking(text, &processor, components, |asked| {
        code_actions(range, only, asked)
    })
}

/// The actions of `kind` at `range` of `text`.
pub(super) fn actions_of(kind: &str, text: &str, range: Range) -> Vec<CodeAction> {
    actions(
        text,
        range,
        &[],
        Some(&[CodeActionKind::from(kind.to_owned())]),
    )
}

/// The edits `action` makes to the document.
pub(super) fn edits(action: &CodeAction) -> Vec<TextEdit> {
    action
        .edit
        .iter()
        .flat_map(|edit| edit.changes.iter().flat_map(|changes| changes.values()))
        .flatten()
        .cloned()
        .collect()
}

/// `text` with `action`'s one edit made (positions in a text of ASCII).
pub(super) fn applied(text: &str, action: &CodeAction) -> String {
    let [edit] = edits(action).try_into().expect("one edit");
    let offset = |position: Position| {
        let line_start: usize = text
            .split_inclusive('\n')
            .take(position.line as usize)
            .map(str::len)
            .sum();
        line_start + position.character as usize
    };
    let (start, end) = (offset(edit.range.start), offset(edit.range.end));
    format!("{}{}{}", &text[..start], edit.new_text, &text[end..])
}

pub(super) fn at(line: u32, character: u32) -> Position {
    Position { line, character }
}

/// The caret at `line` and `character`, as a range.
pub(super) fn caret_at(line: u32, character: u32) -> Range {
    Range {
        start: at(line, character),
        end: at(line, character),
    }
}

/// The quick fixes at `range` of `text`, with the packs' `components`.
fn quick_fixes(text: &str, range: Range, components: &[PackComponent]) -> Vec<CodeAction> {
    actions(text, range, components, Some(&[CodeActionKind::QUICKFIX]))
}

fn notes() -> Vec<PackComponent> {
    vec![PackComponent {
        name: "aside".into(),
        from: "@kit/notes@1".into(),
        ..PackComponent::default()
    }]
}

#[test]
fn a_did_you_mean_is_a_preferred_quick_fix() {
    let text = "# A\n\n::: gird\nText.\n:::\n";
    let [fix] = quick_fixes(text, caret_at(2, 4), &[])
        .try_into()
        .expect("one");
    assert_eq!(fix.title, "Change to `grid`");
    assert_eq!(fix.kind, Some(CodeActionKind::QUICKFIX));
    assert_eq!(fix.is_preferred, Some(true));
    assert_eq!(
        edits(&fix),
        [TextEdit {
            range: Range {
                start: at(2, 4),
                end: at(2, 8),
            },
            new_text: "grid".into(),
        }]
    );
    let diagnostics = fix.diagnostics.expect("its diagnostic");
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn only_the_diagnostics_at_the_range_are_fixed() {
    let text = "# A\n\n::: gird\nText.\n:::\n";
    let elsewhere = Range {
        start: at(0, 0),
        end: at(0, 3),
    };
    assert_eq!(quick_fixes(text, elsewhere, &[]), []);
    assert_eq!(quick_fixes(text, caret_at(2, 6), &[]).len(), 1);
}

#[test]
fn an_unknown_name_a_pack_exports_is_imported() {
    let text = "# A\n\n:: aside[A note.]\n";
    let [import] = quick_fixes(text, caret_at(2, 5), &notes())
        .try_into()
        .expect("one");
    assert_eq!(import.title, "Import `aside` from `@kit/notes@1`");
    assert_eq!(import.kind, Some(CodeActionKind::QUICKFIX));
    let [edit] = edits(&import).try_into().expect("one edit");
    assert_eq!(edit.range.start, at(0, 0));
    assert!(edit.new_text.contains("@kit/notes@1"), "{}", edit.new_text);
}

#[test]
fn a_pack_that_exports_no_such_name_offers_nothing() {
    let text = "# A\n\n:: margin[A note.]\n";
    assert_eq!(quick_fixes(text, caret_at(2, 5), &notes()), []);
    // Away from the name, and on a name that stands for a component.
    let unknown = "# A\n\n:: aside[A note.]\n";
    assert_eq!(quick_fixes(unknown, caret_at(0, 1), &notes()), []);
    let known = "# A\n\n::: grid\nText.\n:::\n";
    let grid = [PackComponent {
        name: "grid".into(),
        from: "@kit/notes@1".into(),
        ..PackComponent::default()
    }];
    assert_eq!(quick_fixes(known, caret_at(2, 5), &grid), []);
    // A frontmatter that writes `md` on one line, which an import can't extend.
    let one_line = "---\nmd: {lang: de}\n---\n\n:: aside[A note.]\n";
    assert_eq!(quick_fixes(one_line, caret_at(4, 5), &notes()), []);
}

/// Pack components rendering `mermaid`: two packs, and one rendering `dot`.
fn diagrams() -> Vec<PackComponent> {
    let rendering = |name: &str, from: &str, languages: &[&str]| {
        let mut signature = compmd::tree::Signature::default();
        signature.renders = Some(languages.iter().map(ToString::to_string).collect());
        PackComponent {
            name: name.into(),
            from: from.into(),
            signature: Some(signature),
            ..PackComponent::default()
        }
    };
    vec![
        rendering("mermaid", "@comp-markdown/mermaid@0.3", &["mermaid"]),
        rendering("diagram", "@kit/diagrams@1", &["mermaid"]),
        rendering("graph", "@kit/graphs@1", &["dot"]),
    ]
}

#[test]
fn a_code_block_a_pack_renders_offers_its_import_anywhere_in_it() {
    let text = "# A\n\n::: box\n```mermaid\ngraph TD\n```\n::: /box\n";
    let titles = |range| -> Vec<String> {
        quick_fixes(text, range, &diagrams())
            .into_iter()
            .map(|action| action.title)
            .collect()
    };
    let both = [
        "Import `@comp-markdown/mermaid@0.3` to render this block",
        "Import `@kit/diagrams@1` to render this block",
    ];
    assert_eq!(titles(caret_at(3, 5)), both, "on its language");
    assert_eq!(titles(caret_at(4, 2)), both, "in its code");
    assert_eq!(titles(caret_at(0, 1)), Vec::<String>::new(), "outside it");
    let [import, _] = quick_fixes(text, caret_at(4, 2), &diagrams())
        .try_into()
        .expect("two");
    assert_eq!(import.diagnostics, None, "no diagnostic: it shows as code");
    assert!(
        applied(text, &import).starts_with(
            "---\nmd:\n  import:\n    - from: \"@comp-markdown/mermaid@0.3\"\n      names: [mermaid]\n---\n\n# A"
        ),
        "{}",
        applied(text, &import)
    );
    let toml = "+++\ntitle = \"A\"\n+++\n\n```dot\na -> b\n```\n";
    let [graph] = quick_fixes(toml, caret_at(5, 1), &diagrams())
        .try_into()
        .expect("one");
    assert!(
        applied(toml, &graph)
            .contains("[[md.import]]\nfrom = \"@kit/graphs@1\"\nnames = [\"graph\"]\n+++")
    );
}

#[test]
fn a_rendered_unknown_or_unnamed_language_offers_no_import() {
    let shout = "```compmd component=shout\nrenders: [mermaid]\n---\n{{code}}\n```\n\n";
    let rendered = format!("{shout}```mermaid\ngraph TD\n```\n");
    assert_eq!(quick_fixes(&rendered, caret_at(7, 1), &diagrams()), []);
    assert_eq!(
        quick_fixes("```sh\nls\n```\n", caret_at(1, 1), &diagrams()),
        []
    );
    assert_eq!(
        quick_fixes("```\nls\n```\n", caret_at(1, 1), &diagrams()),
        []
    );
}

#[test]
fn only_the_kinds_asked_for_are_offered() {
    let text = "# A\n\n::: gird\nText.\n:::\n";
    let everything = actions(text, caret_at(2, 5), &[], None);
    let titles: Vec<&str> = everything
        .iter()
        .map(|action| action.title.as_str())
        .collect();
    assert_eq!(titles, ["Change to `grid`", "Open preview"]);
    let sources = actions_of("source", text, caret_at(2, 5));
    let titles: Vec<&str> = sources.iter().map(|action| action.title.as_str()).collect();
    assert_eq!(titles, ["Open preview"]);
}

#[test]
fn open_preview_is_offered_on_any_line_with_the_documents_uri() {
    let text = "# A\n\nText.\n";
    let [preview] = actions_of("source.compmd.preview", text, caret_at(0, 0))
        .try_into()
        .expect("one");
    let command = preview.command.expect("a command");
    assert_eq!(command.command, "compmd.preview");
    assert_eq!(
        command.arguments,
        Some(vec![serde_json::json!("file:///site/a.comp.md")])
    );
    assert_eq!(preview.edit, None);
}

#[test]
fn a_kind_under_one_asked_for_is_offered() {
    let action = CodeAction {
        kind: Some(CodeActionKind::SOURCE_ORGANIZE_IMPORTS),
        ..CodeAction::default()
    };
    assert!(super::asked_for(&action, &[CodeActionKind::SOURCE]));
    assert!(!super::asked_for(&action, &[CodeActionKind::new("sour")]));
    assert!(!super::asked_for(
        &CodeAction::default(),
        &[CodeActionKind::SOURCE]
    ));
}
