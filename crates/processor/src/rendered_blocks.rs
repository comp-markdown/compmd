//! The code blocks code renderers render (spec §7.7), for an editor: each one's language and where its text is, so an
//! editor hands the requests inside it to the writer's language server for that language (content languages in
//! editors, decision 3).

use crate::line_index::LineIndex;
use crate::lines::Lines;
use crate::resolve::{Renderer, Resolved, code_blocks};
use crate::tree::{Point, Position};
use serde::Serialize;

/// A code block a code renderer renders, whatever its `render` shows: its text is in the renderer's language either
/// way.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RenderedBlock {
    /// Its language, its fence's as written (`mermaid`): one its renderer `renders`.
    pub language: String,
    /// The code renderer that renders it, by the name the document gives it.
    pub component: String,
    /// Where its text is: from its first line after the fence, at the fence's column, to the end of its last line.
    /// For a block with no text, empty, at the start of the line after the fence.
    pub range: Position,
}

/// The code blocks of `resolved`'s processed document that a code renderer renders, in written order: in containers,
/// slots, lists and quotes, and in the bodies of the definitions it holds, which are its text too. A block whose
/// language two renderers render renders neither (F023), and isn't one.
pub(crate) fn of(resolved: &Resolved) -> Vec<RenderedBlock> {
    let document = resolved.document(resolved.main);
    let lines = document.lines();
    let mut blocks = Vec::new();
    code_blocks(&document.root.children, &mut blocks);
    blocks
        .into_iter()
        .filter_map(|code| {
            let language = code.lang.as_deref()?;
            let fence = code.position?.start;
            let Some(Renderer::Component(component, _)) = document.renderers.get(language) else {
                return None;
            };
            Some(RenderedBlock {
                language: language.to_string(),
                component: component.clone(),
                range: text_range(&lines, &document.line_starts, fence, &code.value),
            })
        })
        .collect()
}

/// Where the text `value` of the fenced block whose fence starts at `fence` is, in the document of `lines`.
fn text_range(lines: &Lines<'_>, starts: &LineIndex, fence: Point, value: &str) -> Position {
    let first = fence.line + 1;
    let at = |line: usize, column: usize| Point {
        line,
        column,
        offset: starts.start(line).unwrap_or_default() + column - 1,
    };
    if value.is_empty() {
        let start = at(first, 1);
        return Position { start, end: start };
    }
    let length = |line: usize| lines.line(line).map_or(0, |text| text.chars().count());
    let last = fence.line + value.split('\n').count();
    Position {
        start: at(first, fence.column.min(length(first) + 1)),
        end: at(last, length(last) + 1),
    }
}

#[cfg(test)]
mod tests {
    use super::RenderedBlock;
    use crate::context::RenderContext;
    use crate::expand::Options;
    use crate::tree::{Point, Position};
    use crate::{Ambient, ParseOptions, Processor};
    use std::rc::Rc;

    /// A code renderer for `shout` and `yell`, before what each test adds.
    const SHOUT: &str = "```compmd component=shout\nrenders: [shout, yell]\n---\n{{code}}\n```\n\n";

    /// The rendered blocks of `text`.
    fn blocks(text: &str) -> Vec<RenderedBlock> {
        let context = RenderContext {
            now: 0,
            tz: "UTC".into(),
            time: Rc::new(()),
            inputs: serde_json::Map::default(),
        };
        Processor::new("a.comp.md", text)
            .parse(&ParseOptions::default())
            .resolve((), (), &Ambient::default())
            .validate(&[])
            .load(context, (), ())
            .expand(&Options::default())
            .rendered_blocks()
    }

    /// The text of `text` in `range`: what the range says the block's text is.
    fn spanned(text: &str, range: Position) -> &str {
        &text[range.start.offset..range.end.offset]
    }

    #[test]
    fn a_block_has_its_language_its_renderer_and_its_texts_range() {
        let text = format!("{SHOUT}```yell {{render=source}}\nhi\nthere\n```\n\n```sh\nno\n```\n");
        let [block] = blocks(&text).try_into().expect("the yell block alone");
        assert_eq!(
            (block.language.as_str(), block.component.as_str()),
            ("yell", "shout")
        );
        assert_eq!(spanned(&text, block.range), "hi\nthere");
        let (start, end) = (block.range.start, block.range.end);
        assert_eq!(
            ((start.line, start.column), (end.line, end.column)),
            ((8, 1), (9, 6))
        );
    }

    #[test]
    fn a_blocks_text_starts_at_its_fences_column() {
        let text = format!(
            "{SHOUT}- A list:\n\n  ```shout\n  loud\n  ```\n\n> ```shout\n> quiet\n> ```\n"
        );
        let found = blocks(&text);
        let texts: Vec<&str> = found
            .iter()
            .map(|block| spanned(&text, block.range))
            .collect();
        assert_eq!(texts, ["loud", "quiet"]);
        let starts: Vec<(usize, usize)> = found
            .iter()
            .map(|block| (block.range.start.line, block.range.start.column))
            .collect();
        assert_eq!(starts, [(10, 3), (14, 3)]);
    }

    #[test]
    fn blocks_are_found_in_containers_and_definitions() {
        let text = format!(
            "{SHOUT}````compmd component=twice\n---\n```yell\nin a body\n```\n````\n\n::: box\n```shout\nin a box\n```\n::: /box\n"
        );
        let texts: Vec<String> = blocks(&text)
            .into_iter()
            .map(|block| spanned(&text, block.range).to_string())
            .collect();
        assert_eq!(texts, ["in a body", "in a box"]);
    }

    #[test]
    fn a_block_with_no_text_is_an_empty_range_after_its_fence() {
        let text = format!("{SHOUT}```shout\n```\n");
        let [block] = blocks(&text).try_into().expect("one");
        assert_eq!(block.range.start, block.range.end);
        let fence_ends = text.find("```shout\n").expect("the block") + "```shout\n".len();
        assert_eq!(
            block.range.start,
            Point {
                line: 8,
                column: 1,
                offset: fence_ends
            }
        );
    }

    #[test]
    fn a_language_two_renderers_render_has_no_block() {
        let text = format!(
            "{SHOUT}```compmd component=other\nrenders: yell\n---\n{{{{code}}}}\n```\n\n```yell\nhi\n```\n"
        );
        assert_eq!(blocks(&text), []);
    }
}
