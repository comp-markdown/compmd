//! A fenced code block ([a document drawn in the editor], decisions 3 and 11): its fences hidden, its language on a
//! line above (`code.language: above`) or on its opening fence (`fence`, and where the client draws no `line`), and its
//! lines in the code style to the window's edge. An indented code block has no fences, and is left as written.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::Code;

use super::line_span::LineSpan;
use super::mark::Segment;
use super::planner::{Part, Planner};
use super::primitive::Primitive;
use super::source::Source;
use super::style;
use crate::settings::LanguagePlace;

/// Draws `code`, when it is fenced.
pub(crate) fn draw(code: &Code, planner: &mut Planner) {
    let Some(at) = code.position else { return };
    let ((first, column), (last, _)) = (Source::place(at.start), Source::place(at.end));
    let source = planner.source();
    let Some(fence) = source
        .char_at(first, column)
        .filter(|c| matches!(c, '`' | '~'))
    else {
        return;
    };
    let fence_len = source
        .line(first)
        .chars()
        .skip(column)
        .take_while(|c| *c == fence)
        .count();
    let element = LineSpan::of(first, last);
    let opening = Part {
        own: LineSpan::one(first),
        element,
    };
    planner.style(source.to_end((first, column), last), style::CODE, true);
    let opener = planner
        .source()
        .range(first, column, planner.source().len(first));
    let place = planner.settings().code.options.language;
    match code.lang.as_deref() {
        Some(language) if place != LanguagePlace::Off => {
            let text = language_text(planner, language);
            if place == LanguagePlace::Above && planner.has(Primitive::Line) {
                planner.hide(opener, opening);
                let segments = vec![Segment::new(text, style::CODE_LANGUAGE)];
                let fill = Segment::new(" ", style::CODE);
                planner.line((first, true), segments, Some(fill), opening);
            } else if planner.has(Primitive::Replace) {
                planner.replace(opener, text, style::CODE_LANGUAGE, opening);
            } else {
                let marker = planner.source().range(first, column, column + fence_len);
                planner.hide(marker, opening);
            }
        }
        _ => planner.hide(opener, opening),
    }
    if let Some(closing) = closing_fence(planner.source(), (last, column), (fence, fence_len))
        && last > first
    {
        let range = planner
            .source()
            .range(last, closing, planner.source().len(last));
        let part = Part {
            own: LineSpan::one(last),
            element,
        };
        planner.hide(range, part);
    }
}

/// The column where the closing fence of a block whose last line is `last` starts, when that line is one: the block's
/// fence character, at least as many as its opening fence has.
fn closing_fence(
    source: &Source,
    (last, column): (usize, usize),
    (fence, fence_len): (char, usize),
) -> Option<usize> {
    let start = source.skip_spaces(last, column.min(source.len(last)));
    let rest = source.slice(last, start, source.len(last));
    let rest = rest.trim_end();
    (rest.chars().count() >= fence_len && rest.chars().all(|c| c == fence)).then_some(start)
}

/// A language as its line or its fence shows it: its icon first, when the glyphs have one.
fn language_text(planner: &Planner, language: &str) -> String {
    let icon = planner.glyph(&format!("language.{language}"));
    if icon.is_empty() {
        language.to_owned()
    } else {
        format!("{icon} {language}")
    }
}
