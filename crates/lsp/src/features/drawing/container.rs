//! A container ([a document drawn in the editor], decision 11): a gutter, `│ `, before the lines inside it, its slots'
//! lines too; its opener, slots and closer left as written, their markers in the border's style, its name in the
//! component style, its params set back, each slot's name in the slot style. The gutter is a `prefix` for a container
//! at a line's start, an `insert` at its column for one further in (in a list, a quote). Nothing is hidden, so nothing
//! gives way. A container recovery closed has its gutter to its last line.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::{Container, Node};
use lsp_types::Position;

use super::line_span::LineSpan;
use super::mark::Drawn;
use super::param_spans;
use super::planner::Planner;
use super::primitive::Primitive;
use super::source::Source;
use super::style;

/// Draws `container`'s gutter and styles its opener, slots and closer.
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
    draw_opener(container, planner, (first, opener_last), column);
    for slot in &container.slots {
        if let Some(at) = slot.position {
            draw_slot(planner, Source::place(at.start), &slot.name);
        }
    }
    let inside_end = match closer {
        Some((line, at)) => {
            let closer = planner
                .source()
                .range(line, at, planner.source().trimmed_len(line));
            planner.style(closer, style::CONTAINER_BORDER, false);
            line
        }
        None => content_end(container).max(opener_last) + 1,
    };
    if planner.settings().container.options.gutter && opener_last + 1 < inside_end {
        draw_gutter(planner, (opener_last + 1, inside_end - 1), column);
    }
}

/// Styles the opener's lines `first` to `last`, written from `column`: the markers, the name and the params.
fn draw_opener(
    container: &Container,
    planner: &mut Planner,
    (first, last): (usize, usize),
    column: usize,
) {
    for line in first..=last {
        let start = if line == first { column } else { 0 };
        let end = marker_end(planner.source(), line, start, &[':', '+']);
        if end > start {
            let marker = planner.source().range(line, start, end);
            planner.style(marker, style::CONTAINER_BORDER, false);
        }
    }
    if planner.settings().component.on {
        let from = planner
            .source()
            .skip_spaces(first, marker_end(planner.source(), first, column, &[':']));
        let name = planner
            .source()
            .range(first, from, from + container.name.chars().count());
        planner.style(name, style::COMPONENT, false);
    }
    if planner.settings().param.on {
        for (line, from, to) in
            param_spans::on_lines(planner.tokens(), planner.source(), (first, last))
        {
            let range = planner.source().range(line, from, to);
            planner.style(range, style::PARAM, false);
        }
    }
}

/// Styles a slot's line at `column` of `line`: its marker, `:: #`, and its name.
fn draw_slot(planner: &mut Planner, (line, column): (usize, usize), name: &str) {
    let end = marker_end(planner.source(), line, column, &[':', '#']);
    let marker = planner.source().range(line, column, end);
    planner.style(marker, style::CONTAINER_BORDER, false);
    let from = planner.source().skip_spaces(line, end);
    let name = planner
        .source()
        .range(line, from, from + name.chars().count());
    planner.style(name, style::SLOT, false);
}

/// Where the marker written from `column` of `line` ends: its run of `characters`, spaces between them.
fn marker_end(source: &Source, line: usize, column: usize, characters: &[char]) -> usize {
    let mut end = column;
    for (offset, character) in source.line(line).chars().skip(column).enumerate() {
        if characters.contains(&character) {
            end = column + offset + 1;
        } else if !matches!(character, ' ' | '\t') {
            break;
        }
    }
    end
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

/// Draws `│ ` before the lines `first` to `last`, at `column`.
fn draw_gutter(planner: &mut Planner, (first, last): (usize, usize), column: usize) {
    let text = format!("{} ", planner.glyph("box.vertical"));
    if column == 0 && planner.has(Primitive::Prefix) {
        planner.prefix(LineSpan::of(first, last), text, style::CONTAINER_BORDER);
        return;
    }
    for line in first..=last {
        let position: Position = planner
            .source()
            .position(line, column.min(planner.source().len(line)));
        planner.decorate(Drawn::Insert {
            position,
            text: text.clone(),
            style: style::CONTAINER_BORDER.to_owned(),
        });
    }
}
