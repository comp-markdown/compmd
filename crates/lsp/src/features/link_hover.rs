//! The hover on a link ([the language server], *Links and headings*): its target's section, from its heading to the
//! next of its level or higher, or a file's start, its first lines as Markdown.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

use super::Asked;
use super::link_targets::{headings_of, line_of, link_at, reached, text_of};

/// The lines of a target a hover shows at most.
const MOST_LINES: usize = 12;

/// The hover of the link at `position`; `None` off a link, or for one that leads nowhere in the project.
pub(crate) fn link_hover(position: Position, asked: &Asked) -> Option<Hover> {
    let link = link_at(position, asked)?;
    let reached = reached(&link, asked)?;
    let text = text_of(&reached.file, asked)?;
    let lines: Vec<&str> = text.lines().collect();
    let (first, end) = match &reached.heading {
        Some(heading) => {
            let first = usize::try_from(line_of(&heading.position)).unwrap_or(usize::MAX);
            let next = headings_of(&reached.file, asked)
                .unwrap_or_default()
                .into_iter()
                .map(|other| {
                    (
                        usize::try_from(line_of(&other.position)).unwrap_or(usize::MAX),
                        other.depth,
                    )
                })
                .find(|(line, depth)| *line > first && *depth <= heading.depth)
                .map_or(lines.len(), |(line, _)| line);
            (first, next)
        }
        None => (0, lines.len()),
    };
    let shown: Vec<&str> = lines
        .iter()
        .skip(first)
        .take(end.saturating_sub(first).min(MOST_LINES))
        .copied()
        .collect();
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: shown.join("\n").trim_end().to_owned(),
        }),
        range: Some(asked.encoding.span(asked.text, &link.position)),
    })
}
