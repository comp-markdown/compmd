//! The frontmatter ([a document drawn in the editor], decision 11): its fences as rules, each giving way while the
//! cursor is on it (`frontmatter.rules`), and its lines set back while the cursor is outside it (`frontmatter.dim`),
//! both under `render.frontmatter`; its `md` key set back
//! too (`render.param`), where the processor's tokens say it is.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tokens::TokenKind;
use compmd::tree::Yaml;
use lsp_types::Range;

use super::line_span::{LineSpan, lsp_line};
use super::mark::Drawn;
use super::planner::{Part, Planner};
use super::rule::draw_rule_line;
use super::source::Source;
use super::style;

/// Draws `frontmatter`.
pub(crate) fn draw(frontmatter: &Yaml, planner: &mut Planner) {
    let Some(at) = frontmatter.position else {
        return;
    };
    let (first, last) = (Source::place(at.start).0, Source::place(at.end).0);
    let lines = LineSpan::of(first, last);
    let frontmatter = planner.settings().frontmatter;
    if frontmatter.on {
        let whole = Part {
            own: lines,
            element: lines,
        };
        if frontmatter.options.rules {
            for fence in [first, last] {
                let own = Part {
                    own: LineSpan::one(fence),
                    element: lines,
                };
                draw_rule_line(fence, 0, own, planner);
            }
        }
        if frontmatter.options.dim && last > first + 1 {
            let range = planner
                .source()
                .whole_lines(LineSpan::of(first + 1, last - 1));
            planner.draw(
                Drawn::Style {
                    range,
                    style: style::FRONTMATTER.into(),
                    whole_line: true,
                },
                whole,
            );
        }
    }
    if planner.settings().param.on {
        draw_md_key(lines, planner);
    }
}

/// Sets back the `md` key on `lines`.
fn draw_md_key(lines: LineSpan, planner: &mut Planner) {
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
