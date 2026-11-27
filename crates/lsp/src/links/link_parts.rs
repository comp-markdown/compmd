//! Where a link's parts are written in its text ([the language server], *Links and headings*, rename): its
//! destination's path and fragment, and a reference's or a definition's label, in code points from the link's start.
//! The tree gives a link's destination and label, not where they are; rename edits them where they are.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#links-and-headings

use crate::links::document_links::{DocumentLink, LinkKind};

/// A part's place in a link's text: its first code point and the one after its last.
pub(crate) type Span = (usize, usize);

/// Where the destination's path is (before `#` and `?`) in `written`, `link`'s text: `None` for a reference, or a
/// destination not written as the tree has it (with escapes).
pub(crate) fn path_span(written: &str, link: &DocumentLink) -> Option<Span> {
    let (start, url) = destination(written, link)?;
    let end = url.find(['#', '?']).unwrap_or(url.len());
    Some((start, start + url[..end].chars().count()))
}

/// Where the destination's fragment is (after `#`) in `written`, `link`'s text, when it has one.
pub(crate) fn fragment_span(written: &str, link: &DocumentLink) -> Option<Span> {
    let (start, url) = destination(written, link)?;
    let hash = url.find('#')?;
    let from = start + url[..=hash].chars().count();
    Some((from, start + url.chars().count()))
}

/// Where a reference's or a definition's label is in `written`, its text: a definition's and a shortcut or collapsed
/// reference's first brackets, a full reference's last.
pub(crate) fn label_span(written: &str, link: &DocumentLink) -> Option<Span> {
    let text = written.strip_prefix('!').unwrap_or(written);
    let skipped = written.len() - text.len();
    let (open, close) = match link.kind {
        LinkKind::Inline { .. } => return None,
        LinkKind::Definition { .. } => (0, text.find(']')?),
        LinkKind::Reference { .. } => {
            let last = text.rfind('[')?;
            if last > 0 && text[..last].ends_with(']') && !text.ends_with("[]") {
                (last, text.len() - 1)
            } else {
                (0, text.find(']')?)
            }
        }
    };
    let chars = |bytes: usize| written[..skipped + bytes].chars().count();
    Some((chars(open + 1), chars(close)))
}

/// Where `link`'s destination starts in `written`, in code points, and the destination.
fn destination<'a>(written: &str, link: &'a DocumentLink) -> Option<(usize, &'a str)> {
    let url = link.url()?;
    // Past an opening `<`, where the destination is angle-bracketed.
    let opened = |at: usize| at + usize::from(written[at..].starts_with('<'));
    let after = match link.kind {
        LinkKind::Inline { .. } => written
            .rmatch_indices("](")
            .map(|(at, _)| opened(at + 2))
            .find(|&at| written[at..].starts_with(url)),
        LinkKind::Definition { .. } => written.find("]:").map(|at| {
            let rest = &written[at + 2..];
            opened(at + 2 + (rest.len() - rest.trim_start().len()))
        }),
        LinkKind::Reference { .. } => None,
    }?;
    written[after..]
        .starts_with(url)
        .then(|| (written[..after].chars().count(), url))
}

#[cfg(test)]
mod tests {
    use compmd::tree::{Point, Position};

    use super::*;

    fn link(kind: LinkKind) -> DocumentLink {
        let point = Point {
            line: 1,
            column: 1,
            offset: 0,
        };
        DocumentLink {
            kind,
            position: Position {
                start: point,
                end: point,
            },
        }
    }

    fn inline(url: &str) -> DocumentLink {
        link(LinkKind::Inline { url: url.into() })
    }

    fn part(written: &str, span: Option<Span>) -> Option<String> {
        span.map(|(from, to)| written.chars().skip(from).take(to - from).collect())
    }

    #[test]
    fn a_destinations_path_and_fragment() {
        let written = "[the é tides](../tides.md#high-water \"Tides\")";
        let tides = inline("../tides.md#high-water");
        assert_eq!(
            part(written, path_span(written, &tides)).as_deref(),
            Some("../tides.md")
        );
        assert_eq!(
            part(written, fragment_span(written, &tides)).as_deref(),
            Some("high-water")
        );
        let angled = "![a map](<maps/harbour map.png>)";
        let map = inline("maps/harbour map.png");
        assert_eq!(
            part(angled, path_span(angled, &map)).as_deref(),
            Some("maps/harbour map.png")
        );
        assert_eq!(fragment_span(angled, &map), None);
        let defined = "[tides]:   ./tides.md#low";
        let definition = link(LinkKind::Definition {
            identifier: "tides".into(),
            url: "./tides.md#low".into(),
        });
        assert_eq!(
            part(defined, path_span(defined, &definition)).as_deref(),
            Some("./tides.md")
        );
        assert_eq!(
            part(defined, label_span(defined, &definition)).as_deref(),
            Some("tides")
        );
    }

    #[test]
    fn a_references_label_is_its_last_brackets_when_it_is_full() {
        let reference = link(LinkKind::Reference {
            identifier: "tides".into(),
        });
        for (written, label) in [
            ("[the tides][Tides]", "Tides"),
            ("[Tides][]", "Tides"),
            ("[Tides]", "Tides"),
            ("![a map][Tides]", "Tides"),
        ] {
            assert_eq!(
                part(written, label_span(written, &reference)).as_deref(),
                Some(label),
                "{written}"
            );
        }
        assert_eq!(label_span("[a](b)", &inline("b")), None);
    }

    #[test]
    fn a_destination_written_otherwise_than_the_tree_has_it_isnt_found() {
        let written = r"[a](tides\_high.md)";
        assert_eq!(path_span(written, &inline("tides_high.md")), None);
    }
}
