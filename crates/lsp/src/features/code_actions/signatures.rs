//! `refactor.rewrite`: *generate signature* and *update signature* for a host-language component defined in the
//! document ([the language server], decision 11): its props read by the component reader, written as the signature
//! fence's params and slots.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#templates-code-actions-refactorings-in-1

mod declarations;
mod fence_body;

use compmd::Language;
use compmd::analysis::ComponentProps;
use compmd::tree::{Definition, DefinitionBody, Node, SignatureFence, SlotDeclaration};
use compmd_components::ComponentReader;
use lsp_types::{CodeAction, CodeActionKind, Position, Range, TextEdit};

use super::{Asked, editing};
use declarations::Declared;

/// The action on the host-language definition fence `range` starts in: generating its signature fence, or bringing
/// the one it has up to date with its props when it isn't.
pub(super) fn signature_action(range: Range, asked: &Asked) -> Option<CodeAction> {
    let children = &asked.processor?.tree().children;
    let (definition, source) = children.iter().find_map(|node| match node {
        Node::ComponentDefinition(definition) => match &definition.body {
            DefinitionBody::Host { source, .. } if holds(definition, range.start) => {
                Some((definition, source))
            }
            _ => None,
        },
        _ => None,
    })?;
    let props = ComponentReader.props(Language::from_name(&definition.language)?, source)?;
    let declared = Declared::of(&props);
    let fence = children.iter().find_map(|node| match node {
        Node::ComponentSignature(fence) if fence.name == definition.name => Some(fence),
        _ => None,
    });
    let name = &definition.name;
    let (title, edit) = match fence {
        None => (
            format!("Generate the signature of `{name}`"),
            generated(definition, &declared, asked)?,
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

/// Whether `position` is on one of the lines of `definition`'s fence.
fn holds(definition: &Definition, position: Position) -> bool {
    definition.position.is_some_and(|fence| {
        let line = position.line as usize + 1;
        fence.start.line <= line && line <= fence.end.line
    })
}

/// The edit that writes a signature fence for `declared` after `definition`'s fence.
fn generated(definition: &Definition, declared: &Declared, asked: &Asked) -> Option<TextEdit> {
    let after = asked.encoding.span(asked.text, &definition.position?).end;
    let body = fence_body::generated(declared).join("\n");
    Some(TextEdit {
        range: Range {
            start: after,
            end: after,
        },
        new_text: format!(
            "\n\n```compmd component={} signature\n{body}\n```",
            definition.name
        ),
    })
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
