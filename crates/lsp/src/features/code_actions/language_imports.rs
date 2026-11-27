//! The action on a fenced code block whose language no code renderer the document names renders, but one a known
//! pack exports does ([the language server], IN-1): *Import `<pack>` to render this block*. No diagnostic comes with
//! it: such a block is valid CompMD and shows as code. When the site can't load the pack, the import's warning then
//! offers its registry (Q4).
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#templates-code-actions-refactorings-in-1

use compmd::diagnostic::{Location, Range as SourceRange};
use compmd::hints::Subject;
use compmd::tree::{Code, Node};
use lsp_types::{CodeAction, CodeActionKind, Range};

use super::{Asked, editing};
use crate::features::import_edit::adding_import;

/// The imports that render the code block `range` starts in, one for each pack component rendering its language.
pub(super) fn imports(range: Range, asked: &Asked) -> Vec<CodeAction> {
    let Some(language) = unrendered(range, asked) else {
        return Vec::new();
    };
    let Some(processor) = asked.processor else {
        return Vec::new();
    };
    asked
        .pack_components
        .iter()
        .filter(|component| component.languages().contains(&language))
        .filter_map(|component| {
            let edit = adding_import(
                asked.text,
                processor.tree(),
                &component.from,
                &component.name,
            )?;
            Some(CodeAction {
                title: format!("Import `{}` to render this block", component.from),
                kind: Some(CodeActionKind::QUICKFIX),
                edit: Some(editing(asked, vec![edit])),
                ..CodeAction::default()
            })
        })
        .collect()
}

/// The language of the code block `range` starts in, when it has one and no code renderer the document names
/// renders it.
fn unrendered(range: Range, asked: &Asked) -> Option<String> {
    let processor = asked.processor?;
    let line = usize::try_from(range.start.line).ok()? + 1;
    let code = holding(&processor.tree().children, line)?;
    let start = code.position?.start;
    let opening = asked.text.split('\n').nth(start.line - 1)?;
    let column = language_column(opening, start.column);
    let at = asked.encoding.range(
        asked.text,
        &SourceRange {
            start: Location::new(start.line, column),
            end: None,
        },
    );
    let hint = processor.hint_at(asked.offset(at.start))?;
    match hint.subject? {
        Subject::Language {
            language,
            renderer: None,
            ..
        } if !language.is_empty() => Some(language),
        _ => None,
    }
}

/// The column, from 1, of the language on a code block's opening line `line` whose fence starts at `fence`: past
/// the fence and the white space after it.
fn language_column(line: &str, fence: usize) -> usize {
    let rest = line.chars().skip(fence - 1);
    let marks = rest.clone().take_while(|c| matches!(c, '`' | '~')).count();
    let spaces = rest.skip(marks).take_while(|c| c.is_whitespace()).count();
    fence + marks + spaces
}

/// The fenced code block among `nodes`, or inside them, whose lines hold `line` (from 1).
fn holding(nodes: &[Node], line: usize) -> Option<&Code> {
    nodes.iter().find_map(|node| {
        if let Node::Code(code) = node
            && code
                .position
                .is_some_and(|at| at.start.line <= line && line <= at.end.line)
        {
            return Some(code);
        }
        let slots = match node {
            Node::ComponentContainer(container) => container.slots.as_slice(),
            _ => &[],
        };
        holding(node.children(), line)
            .or_else(|| slots.iter().find_map(|slot| holding(&slot.children, line)))
    })
}
