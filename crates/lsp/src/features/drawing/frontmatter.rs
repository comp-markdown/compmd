//! The frontmatter ([a document drawn in the editor], decision 11): its `md` key set back, where the processor's tokens
//! say it is.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tokens::TokenKind;
use compmd::tree::Yaml;
use lsp_types::Range;

use super::line_span::{LineSpan, lsp_line};
use super::planner::Planner;
use super::source::Source;
use super::style;

/// Draws `frontmatter`'s `md` key.
pub(crate) fn draw(frontmatter: &Yaml, planner: &mut Planner) {
    let Some(at) = frontmatter.position else {
        return;
    };
    let lines = LineSpan::of(Source::place(at.start).0, Source::place(at.end).0);
    let keys: Vec<Range> = planner
        .tokens()
        .iter()
        .filter(|token| token.kind == TokenKind::FrontmatterKey)
        .filter(|token| lines.contains(lsp_line(token.range.start.line - 1)))
        .map(|token| {
            let (line, from) = (token.range.start.line - 1, token.range.start.column - 1);
            let to = token.range.end.map_or(from, |end| end.column - 1);
            planner.source().range(line, from, to)
        })
        .collect();
    for key in keys {
        planner.style(key, style::FRONTMATTER_MD, false);
    }
}
