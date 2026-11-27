//! A container as a box ([a document drawn in the editor], decision 11): its opener as a top border titled with its
//! name and its params, set back; each slot as a divider titled with the slot's name; its closer as the bottom border;
//! and its other lines with `│ ` before them, a `prefix` for a box at a line's start, an `insert` at its column for a
//! box further in (in a list, a quote). A container recovery closed has no bottom border.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::{Container, Node};
use lsp_types::Position;

use super::box_border::{BoxBorder, Titled};
use super::line_span::LineSpan;
use super::mark::Drawn;
use super::param_spans;
use super::planner::Planner;
use super::primitive::Primitive;
use super::source::Source;
use super::style;

/// Draws `container`'s box.
pub(crate) fn draw(container: &Container, planner: &mut Planner) {
    let Some(opener) = container.position else {
        return;
    };
    let ((first, column), (opener_end, _)) =
        (Source::place(opener.start), Source::place(opener.end));
    let opener_last = param_spans::last_line(&container.params, opener_end);
    let closer = container
        .close_position
        .map(|closer| Source::place(closer.start));
    let last = closer.map_or_else(|| content_end(container).max(opener_last), |(line, _)| line);
    let element = LineSpan::of(first, last);
    let opener_lines = LineSpan::of(first, opener_last);
    draw_top(container, planner, (opener_lines, column), element);
    let slot_lines = draw_dividers(container, planner, element);
    let inside_end = match closer {
        Some(closer) => {
            draw_bottom(planner, closer, element);
            closer.0
        }
        None => last + 1,
    };
    draw_sides(planner, (opener_last + 1, inside_end), &slot_lines, column);
}

/// Draws the top border in place of the opener's `lines`, starting at `column`: the name, then the params.
fn draw_top(
    container: &Container,
    planner: &mut Planner,
    (lines, column): (LineSpan, usize),
    element: LineSpan,
) {
    let (first, last) = (lines.first as usize, lines.last as usize);
    let params = if planner.settings().param.on {
        style::PARAM
    } else {
        style::CONTAINER_BORDER
    };
    let name = named(
        planner.source(),
        (first, column),
        &container.name,
        style::COMPONENT,
    );
    let marker_end = name.from;
    let written = param_spans::on_lines(planner.tokens(), planner.source(), (first, last));
    let title = std::iter::once(name)
        .chain(written.into_iter().map(|(line, from, to)| Titled {
            line,
            from,
            to,
            style: params,
        }))
        .collect();
    let top = BoxBorder {
        lines,
        column,
        marker_end,
        corner: "box.top-left",
        title,
        line_style: style::CONTAINER_TOP,
        element,
    };
    top.draw(planner);
}

/// Draws a divider in place of each slot's line, titled with its name; gives the slots' lines.
fn draw_dividers(container: &Container, planner: &mut Planner, element: LineSpan) -> Vec<usize> {
    let mut slot_lines = Vec::new();
    for slot in &container.slots {
        let Some(at) = slot.position else { continue };
        let (line, column) = Source::place(at.start);
        slot_lines.push(line);
        let name = named(planner.source(), (line, column), &slot.name, style::SLOT);
        let divider = BoxBorder {
            lines: LineSpan::one(line),
            column,
            marker_end: name.from,
            corner: "box.left",
            title: vec![name],
            line_style: style::CONTAINER_DIVIDER,
            element,
        };
        divider.draw(planner);
    }
    slot_lines
}

/// Draws the bottom border in place of the closer at `column` of `line`.
fn draw_bottom(planner: &mut Planner, (line, column): (usize, usize), element: LineSpan) {
    let bottom = BoxBorder {
        lines: LineSpan::one(line),
        column,
        marker_end: planner.source().trimmed_len(line),
        corner: "box.bottom-left",
        title: Vec::new(),
        line_style: style::CONTAINER_BOTTOM,
        element,
    };
    bottom.draw(planner);
}

/// Where `name` is written on `line` after the marker that starts at `column` (`:::`, `:: #`), as a title's part.
fn named(
    source: &Source,
    (line, column): (usize, usize),
    name: &str,
    style: &'static str,
) -> Titled {
    let from = column
        + source
            .line(line)
            .chars()
            .skip(column)
            .take_while(|c| matches!(c, ':' | '#' | ' ' | '\t'))
            .count();
    Titled {
        line,
        from,
        to: from + name.chars().count(),
        style,
    }
}

/// The last line of what a container recovery closed holds.
fn content_end(container: &Container) -> usize {
    let nodes = container
        .children
        .iter()
        .chain(container.slots.iter().flat_map(|slot| &slot.children));
    nodes
        .filter_map(Node::position)
        .map(|at| Source::place(at.end).0)
        .max()
        .unwrap_or_default()
}

/// Draws `│ ` before each line from `from` up to `to` but the slots' lines, at `column`.
fn draw_sides(planner: &mut Planner, (from, to): (usize, usize), slots: &[usize], column: usize) {
    let text = format!("{} ", planner.glyph("box.vertical"));
    let mut run: Option<(usize, usize)> = None;
    for line in from..=to {
        let inside = line < to && !slots.contains(&line);
        match (inside, run) {
            (true, Some((start, _))) => run = Some((start, line)),
            (true, None) => run = Some((line, line)),
            (false, Some(lines)) => {
                draw_run(planner, lines, column, &text);
                run = None;
            }
            (false, None) => {}
        }
    }
}

/// Draws `text` before the lines `first` to `last`.
fn draw_run(planner: &mut Planner, (first, last): (usize, usize), column: usize, text: &str) {
    if column == 0 && planner.has(Primitive::Prefix) {
        planner.prefix(
            LineSpan::of(first, last),
            text.to_owned(),
            style::CONTAINER_BORDER,
        );
        return;
    }
    for line in first..=last {
        let position: Position = planner
            .source()
            .position(line, column.min(planner.source().len(line)));
        planner.decorate(Drawn::Insert {
            position,
            text: text.to_owned(),
            style: style::CONTAINER_BORDER.to_owned(),
        });
    }
}
