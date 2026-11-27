//! Where a component's params are written on a line, from the processor's tokens (decision 1): from the first param's
//! token to the last's, with the braces around them.

use compmd::tokens::{SyntaxToken, TokenKind};
use compmd::tree::Param;

use super::source::Source;

/// The last line a component's line runs to: `last`, or its params' last continuation line.
pub(crate) fn last_line(params: &[Param], last: usize) -> usize {
    params
        .iter()
        .filter_map(|param| param.position)
        .map(|at| Source::place(at.end).0)
        .fold(last, usize::max)
}

/// Whether a token of `kind` is part of a param, an interpolation in a quoted value being inside the value's token.
const fn is_param(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::ParamKey
            | TokenKind::Breakpoint
            | TokenKind::ParamClass
            | TokenKind::ParamId
            | TokenKind::ParamString
            | TokenKind::ParamBare
            | TokenKind::ExpressionParen
    )
}

/// The columns of `line` its params are written on, braces and all; `None` when it has none.
pub(crate) fn on_line(
    tokens: &[SyntaxToken],
    source: &Source,
    line: usize,
) -> Option<(usize, usize)> {
    let mut spans = tokens
        .iter()
        .filter(|token| is_param(token.kind) && token.range.start.line == line + 1)
        .map(|token| {
            let start = token.range.start.column - 1;
            let end = token.range.end.map_or(start, |end| end.column - 1);
            (start, end)
        });
    let (mut from, mut to) = spans.next()?;
    for (_, end) in spans {
        to = to.max(end);
    }
    let before = source.line(line).chars().take(from).collect::<String>();
    if before.trim_end().ends_with('{') {
        from = before.trim_end().chars().count() - 1;
    }
    let after = source.skip_spaces(line, to);
    if source.char_at(line, after) == Some('}') {
        to = after + 1;
    }
    Some((from, to))
}

/// The params of each of the lines `first` to `last`, each a line and its columns.
pub(crate) fn on_lines(
    tokens: &[SyntaxToken],
    source: &Source,
    (first, last): (usize, usize),
) -> Vec<(usize, usize, usize)> {
    (first..=last)
        .filter_map(|line| on_line(tokens, source, line).map(|(from, to)| (line, from, to)))
        .collect()
}
