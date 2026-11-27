//! A table ([a document drawn in the editor], decisions 3, 5, 7 and 11), drawn over its text, which the formatter
//! aligns: each `|` replaced by `│`, the delimiter row by `├─┼─┤`, and, with `table.borders`, a border above it and
//! below it, each a `line`. Its marks give way on all its lines, so a cell is edited raw. Where the client draws no
//! `replace`, the pipes and the delimiter row are styled; where it draws no `line`, there are no borders.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::Table;

use super::line_span::LineSpan;
use super::mark::Segment;
use super::planner::{Part, Planner};
use super::primitive::Primitive;
use super::source::Source;
use super::style;

/// The glyphs a rule across the table is drawn with: its left end, where a pipe crosses it, its right end.
type Ends = [&'static str; 3];

/// The delimiter row's.
const DELIMITER: Ends = ["box.left", "box.cross", "box.right"];
/// The top border's.
const TOP: Ends = ["box.top-left", "box.top", "box.top-right"];
/// The bottom border's.
const BOTTOM: Ends = ["box.bottom-left", "box.bottom", "box.bottom-right"];

/// Draws `table`.
pub(crate) fn draw(table: &Table, planner: &mut Planner) {
    let Some(at) = table.position else { return };
    let ((first, column), (last, _)) = (Source::place(at.start), Source::place(at.end));
    let lines = LineSpan::of(first, last);
    let part = Part {
        own: lines,
        element: lines,
    };
    if planner.settings().table.options.borders {
        for (line, above, ends) in [(first, true, TOP), (last, false, BOTTOM)] {
            let border = rule_across(planner, (line, column), ends);
            let text = format!("{}{border}", " ".repeat(column));
            let segments = vec![Segment::new(text, style::TABLE_BORDER)];
            planner.line((line, above), segments, None, part);
        }
    }
    for line in first..=last {
        if line == first + 1 {
            draw_delimiter(planner, (line, column), part);
        } else {
            draw_pipes(planner, (line, column), part);
        }
    }
}

/// The columns of the pipes of `line` from `column` on, an escaped one (`\|`) left out.
pub(crate) fn pipes(source: &Source, (line, column): (usize, usize)) -> Vec<usize> {
    let mut found = Vec::new();
    let mut escaped = false;
    for (at, c) in source.line(line).chars().enumerate().skip(column) {
        if c == '|' && !escaped {
            found.push(at);
        }
        escaped = c == '\\' && !escaped;
    }
    found
}

/// Draws each pipe of a row.
fn draw_pipes(planner: &mut Planner, (line, column): (usize, usize), part: Part) {
    let glyph = planner.glyph("box.vertical");
    for pipe in pipes(planner.source(), (line, column)) {
        let range = planner.source().range(line, pipe, pipe + 1);
        if planner.has(Primitive::Replace) {
            planner.replace(range, glyph.clone(), style::TABLE_BORDER, part);
        } else {
            planner.style(range, style::TABLE_BORDER, false);
        }
    }
}

/// Draws the delimiter row.
fn draw_delimiter(planner: &mut Planner, (line, column): (usize, usize), part: Part) {
    let source = planner.source();
    let range = source.range(line, column, source.trimmed_len(line));
    if planner.has(Primitive::Replace) {
        let rule = rule_across(planner, (line, column), DELIMITER);
        planner.replace(range, rule, style::TABLE_BORDER, part);
    } else {
        planner.style(range, style::TABLE_BORDER, false);
    }
}

/// A rule across `line` from `column` to its last character that isn't a space: each pipe a crossing (or an end, at
/// either end), anything else the horizontal glyph.
fn rule_across(
    planner: &Planner,
    (line, column): (usize, usize),
    [left, cross, right]: Ends,
) -> String {
    let source = planner.source();
    let pipes = pipes(source, (line, column));
    let end = source.trimmed_len(line);
    let horizontal = planner.glyph("box.horizontal");
    (column..end)
        .map(|at| {
            if !pipes.contains(&at) {
                horizontal.clone()
            } else if at == column {
                planner.glyph(left)
            } else if at + 1 == end {
                planner.glyph(right)
            } else {
                planner.glyph(cross)
            }
        })
        .collect()
}
