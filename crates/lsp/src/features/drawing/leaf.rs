//! A leaf component's line ([a document drawn in the editor], decision 11): its name in the component style
//! (`component`), its params set back (`param`). Nothing is hidden, so nothing gives way.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::Leaf;

use super::param_spans;
use super::planner::Planner;
use super::source::Source;
use super::style;

/// Draws `leaf`'s lines.
pub(crate) fn draw(leaf: &Leaf, planner: &mut Planner) {
    let Some(at) = leaf.position else { return };
    let ((first, column), (last, _)) = (Source::place(at.start), Source::place(at.end));
    let last = param_spans::last_line(&leaf.params, last);
    let settings = planner.settings();
    if settings.component.on {
        let source = planner.source();
        let marker = source
            .line(first)
            .chars()
            .skip(column)
            .take_while(|c| *c == ':')
            .count();
        let from = source.skip_spaces(first, column + marker);
        let range = source.range(first, from, from + leaf.name.chars().count());
        planner.style(range, style::COMPONENT, false);
    }
    if settings.param.on {
        for (line, from, to) in
            param_spans::on_lines(planner.tokens(), planner.source(), (first, last))
        {
            let range = planner.source().range(line, from, to);
            planner.style(range, style::PARAM, false);
        }
    }
}
