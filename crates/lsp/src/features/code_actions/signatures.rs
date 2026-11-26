//! `refactor.rewrite`: *generate signature* and *update signature* for a host-language component ([the language
//! server], decision 11), defined in the document or imported from a component file: its props read by the component
//! reader, written as the signature fence's params and slots.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#templates-code-actions-refactorings-in-1

mod component_at;
mod declarations;
mod fence_body;

use compmd::analysis::ComponentProps;
use compmd::tree::{Node, SignatureFence, SlotDeclaration};
use compmd_components::ComponentReader;
use lsp_types::{CodeAction, CodeActionKind, Position, Range, TextEdit};

use super::{Asked, editing};
use component_at::{Component, component_at};
use declarations::Declared;

/// The action on the component `range` starts on (its definition fence, its file's import, or its signature fence):
/// generating its signature fence, or bringing the one it has up to date with its props when it isn't.
pub(super) fn signature_action(range: Range, asked: &Asked) -> Option<CodeAction> {
    let component = component_at(range.start, asked)?;
    let props = ComponentReader.props(component.language, &component.source)?;
    let declared = Declared::of(&props);
    let fence = asked
        .processor?
        .tree()
        .children
        .iter()
        .find_map(|node| match node {
            Node::ComponentSignature(fence) if fence.name == component.name => Some(fence),
            _ => None,
        });
    let name = &component.name;
    let (title, edit) = match fence {
        None => (
            format!("Generate the signature of `{name}`"),
            generated(&component, &declared),
        ),
        Some(fence) => (
            format!("Update the signature of `{name}`"),
            updated(fence, &declared, props.rest, asked)?,
        ),
    };
    Some(CodeAction {
        title,
        kind: Some(CodeActionKind::REFACTOR_REWRITE),
        edit: Some(editing(asked, vec![edit])),
        ..CodeAction::default()
    })
}

/// The edit that writes a signature fence for `declared` where `component`'s goes.
fn generated(component: &Component, declared: &Declared) -> TextEdit {
    let body = fence_body::generated(declared).join("\n");
    TextEdit {
        range: Range {
            start: component.after,
            end: component.after,
        },
        new_text: format!(
            "\n\n```compmd component={} signature\n{body}\n```",
            component.name
        ),
    }
}

/// The edit that brings `fence`'s text up to date with `declared`; none when it is.
fn updated(
    fence: &SignatureFence,
    declared: &Declared,
    rest: bool,
    asked: &Asked,
) -> Option<TextEdit> {
    let position = fence.position?;
    let lines: Vec<&str> = asked.text.split('\n').collect();
    // From 1, the fence's lines; its text is between them.
    let body: Vec<&str> = lines
        .get(position.start.line..position.end.line.checked_sub(1)?)?
        .iter()
        .map(|line| line.trim_end_matches('\r'))
        .collect();
    let slots: Vec<String> = fence
        .signature
        .as_ref()
        .and_then(|signature| signature.slots.as_ref())
        .into_iter()
        .flatten()
        .map(|slot| match slot {
            SlotDeclaration::Name(name) | SlotDeclaration::Template { name, .. } => name.clone(),
        })
        .collect();
    let new = fence_body::updated(&body, declared, rest, &slots);
    if new == body {
        return None;
    }
    let line = |line: usize| Position {
        line: u32::try_from(line).unwrap_or(u32::MAX),
        character: 0,
    };
    let mut new_text = new.join("\n");
    if !new.is_empty() {
        new_text.push('\n');
    }
    Some(TextEdit {
        range: Range {
            start: line(position.start.line),
            end: line(position.end.line - 1),
        },
        new_text,
    })
}

#[cfg(test)]
mod tests;
