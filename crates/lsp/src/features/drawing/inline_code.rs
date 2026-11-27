//! Inline code ([a document drawn in the editor], decision 11): its span, backticks and all, in the inline code style.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::Literal;
use lsp_types::Range;

use super::planner::Planner;
use super::source::Source;
use super::style;

/// Draws the code span `code`.
pub(crate) fn draw(code: &Literal, planner: &mut Planner) {
    let Some(at) = code.position else { return };
    let ((first, from), (last, to)) = (Source::place(at.start), Source::place(at.end));
    let source = planner.source();
    let range = Range {
        start: source.position(first, from),
        end: source.position(last, to),
    };
    planner.style(range, style::CODE_INLINE, false);
}
