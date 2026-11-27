//! A block quote ([a document drawn in the editor], decision 11): each line's `>` replaced by the quote glyph, `▋`, in
//! the style of its depth (`compmd.quote.1`, …), or of its callout for a GitHub alert; styled where the client draws no
//! `replace`. A lazy line, without its `>`, has nothing to replace.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::Blockquote;

use super::callout;
use super::line_span::LineSpan;
use super::planner::{Part, Planner};
use super::primitive::Primitive;
use super::source::Source;
use super::style;
use super::walk::{self, Nesting};

/// Draws `quote`, and what it holds, at `nesting`.
pub(crate) fn draw(quote: &Blockquote, planner: &mut Planner, nesting: Nesting) {
    let depth = nesting.quotes + 1;
    if let Some(at) = quote.position {
        let ((first, column), (last, _)) = (Source::place(at.start), Source::place(at.end));
        let element = LineSpan::of(first, last);
        let alert = quote.alert.filter(|_| planner.settings().callout.on);
        if planner.settings().quote.on {
            let style = alert.map_or_else(|| style::quote(depth), style::callout);
            draw_bars(planner, (element, column), &style);
        }
        if let Some(alert) = alert {
            callout::draw(alert, (first, column), element, planner);
        }
    }
    let inner = Nesting {
        quotes: depth,
        ..nesting
    };
    walk::draw(&quote.children, planner, inner);
}

/// Draws the `>` at `column` of each of `element`'s lines in `style`.
fn draw_bars(planner: &mut Planner, (element, column): (LineSpan, usize), style: &str) {
    let glyph = planner.glyph("quote");
    for line in element.first as usize..=element.last as usize {
        if planner.source().char_at(line, column) != Some('>') {
            continue;
        }
        let range = planner.source().range(line, column, column + 1);
        let part = Part {
            own: LineSpan::one(line),
            element,
        };
        if planner.has(Primitive::Replace) {
            planner.replace(range, glyph.clone(), style, part);
        } else {
            planner.style(range, style, false);
        }
    }
}
