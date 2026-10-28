//! Inline nodes, and CompMD's inline forms (spec §3.2, §4.4, §8.1): inline components with their fallback and params
//! (F011, F017, W107), interpolations and `{{` that isn't one (W106), wiki links, and `urlParts` on URLs that hold
//! an interpolation.

use super::Converter;
use super::line::matching_bracket;
use super::params::{self, interpolation_at, is_space, name_len, string_parts};
use super::source::right;
use crate::diagnostic::Code;
use crate::tree::{
    Image, ImageReference, Interpolation, Leaf, Link, LinkReference, Literal, Node, Parent, Point,
    Position, Reference, ReferenceType, StringPart, Void, WikiLink,
};
use markdown::mdast;
use std::iter::Peekable;

impl Converter<'_, '_> {
    /// The inline children of a paragraph, heading, cell or other phrasing parent; in a table cell, `\|` in an
    /// inline component or a wiki link is a `|` (spec §3.2 rule 6).
    pub(super) fn inlines(&mut self, nodes: Vec<mdast::Node>, in_table: bool) -> Vec<Node> {
        let mut out = Vec::new();
        let mut nodes = nodes.into_iter().peekable();
        while let Some(node) = nodes.next() {
            match node {
                mdast::Node::CompmdRaw(raw) => self.raw_inline(raw, &mut nodes, in_table, &mut out),
                other => out.extend(self.inline(other, in_table)),
            }
        }
        merge_texts(out)
    }

    /// An inline node; `None` for a node the fork makes only for syntax CompMD leaves out, such as MDX.
    pub(super) fn inline(&mut self, node: mdast::Node, in_table: bool) -> Option<Node> {
        let parent =
            |this: &mut Self, children, position: Option<markdown::unist::Position>| Parent {
                children: this.inlines(children, in_table),
                position: this.source.position(position.as_ref()),
            };
        Some(match node {
            mdast::Node::Text(text) => Node::Text(self.literal(text.value, text.position.as_ref())),
            mdast::Node::InlineCode(code) => {
                Node::InlineCode(self.literal(code.value, code.position.as_ref()))
            }
            mdast::Node::InlineMath(math) => {
                Node::InlineMath(self.literal(math.value, math.position.as_ref()))
            }
            mdast::Node::Html(html) => Node::Html(self.literal(html.value, html.position.as_ref())),
            mdast::Node::Emphasis(node) => {
                Node::Emphasis(parent(self, node.children, node.position))
            }
            mdast::Node::Strong(node) => Node::Strong(parent(self, node.children, node.position)),
            mdast::Node::Delete(node) => Node::Delete(parent(self, node.children, node.position)),
            mdast::Node::Break(node) => Node::Break(Void {
                position: self.source.position(node.position.as_ref()),
            }),
            mdast::Node::Link(link) => Node::Link(Link {
                children: self.inlines(link.children, in_table),
                url_parts: self.url_parts(&link.url),
                url: link.url,
                title: link.title,
                position: self.source.position(link.position.as_ref()),
            }),
            mdast::Node::Image(image) => Node::Image(Image {
                alt: image.alt,
                url_parts: self.url_parts(&image.url),
                url: image.url,
                title: image.title,
                position: self.source.position(image.position.as_ref()),
            }),
            mdast::Node::LinkReference(link) => Node::LinkReference(LinkReference {
                children: self.inlines(link.children, in_table),
                reference_type: reference_type(link.reference_kind),
                identifier: link.identifier,
                label: link.label,
                position: self.source.position(link.position.as_ref()),
            }),
            mdast::Node::ImageReference(image) => Node::ImageReference(ImageReference {
                alt: image.alt,
                reference_type: reference_type(image.reference_kind),
                identifier: image.identifier,
                label: image.label,
                position: self.source.position(image.position.as_ref()),
            }),
            mdast::Node::FootnoteReference(footnote) => Node::FootnoteReference(Reference {
                identifier: footnote.identifier,
                label: footnote.label,
                position: self.source.position(footnote.position.as_ref()),
            }),
            _ => return None,
        })
    }

    fn literal(&self, value: String, position: Option<&markdown::unist::Position>) -> Literal {
        Literal {
            value,
            position: self.source.position(position),
        }
    }

    /// An inline component, interpolation or wiki link the fork found.
    fn raw_inline(
        &mut self,
        raw: mdast::CompmdRaw,
        next: &mut Peekable<impl Iterator<Item = mdast::Node>>,
        in_table: bool,
        out: &mut Vec<Node>,
    ) {
        let position = self.source.position(raw.position.as_ref());
        let start = position.map_or(super::START, |position| position.start);
        match raw.kind {
            mdast::CompmdRawKind::ComponentInline => {
                out.extend(self.component(&raw.value, position, next.peek(), in_table));
            }
            mdast::CompmdRawKind::Interpolation => {
                let chars: Vec<char> = raw.value.chars().collect();
                let path = interpolation_at(&chars, 0)
                    .map(|(path, _)| path)
                    .unwrap_or_default();
                out.push(Node::Interpolation(Interpolation {
                    path,
                    block: false,
                    position,
                }));
            }
            mdast::CompmdRawKind::InterpolationLiteral => {
                self.diagnose(
                    Code::W106,
                    start,
                    "`{{` isn't followed by a path, so it is text; write `\\{{` for literal braces, and use `:value` for a computed value",
                );
                out.push(Node::Text(Literal {
                    value: raw.value,
                    position,
                }));
            }
            mdast::CompmdRawKind::WikiLink => {
                out.push(Node::WikiLink(wiki_link(&raw.value, position)));
            }
            mdast::CompmdRawKind::ComponentLine => out.push(Node::Text(Literal {
                value: raw.value,
                position,
            })),
        }
    }

    /// An inline component, `:name[fallback]{params}`, and the text after it when its params didn't parse.
    fn component(
        &mut self,
        raw: &str,
        position: Option<Position>,
        next: Option<&mdast::Node>,
        in_table: bool,
    ) -> Vec<Node> {
        let text: Vec<char> = raw.chars().collect();
        let start = position.map_or(super::START, |position| position.start);
        let name_end = 1 + name_len(&text, 1).unwrap_or(0);
        let close = matching_bracket(&text, name_end).unwrap_or(text.len().saturating_sub(1));
        let fallback_text: String = text[name_end + 1..close].iter().collect();
        let mut component = Leaf {
            name: text[1..name_end].iter().collect(),
            fallback: self.fallback(&fallback_text, right(start, name_end + 1)),
            params: Vec::new(),
            position,
            recovered: false,
        };
        if close + 1 < text.len() {
            let (chars, indexes) = unescape_pipes(&text[close + 1..], in_table);
            let place = |index: usize| right(start, close + 1 + indexes[index]);
            let list = params::braced(&chars, 0);
            if let Some(at) = list.broke_at {
                self.diagnose(Code::F011, place(at), BROKEN_INLINE);
                component.recovered = true;
                let rest = Literal {
                    value: text[close + 1..].iter().collect(),
                    position: position.map(|position| Position {
                        start: right(start, close + 1),
                        end: position.end,
                    }),
                };
                return vec![Node::ComponentInline(component), Node::Text(rest)];
            }
            self.add_params_at(&mut component.params, list, place);
        } else if let Some(mdast::Node::Text(after)) = next {
            self.params_after(&mut component, after);
        }
        vec![Node::ComponentInline(component)]
    }

    /// Text right after an inline component without params: `{` that starts no param list the fork took is F011
    /// (spec §10.3), and a space then a param list is W107 (spec §3.2 rule 5).
    fn params_after(&mut self, component: &mut Leaf, after: &mdast::Text) {
        let line: Vec<char> = after.value.lines().next().unwrap_or("").chars().collect();
        let from = after.position.as_ref().map_or(super::START, |position| {
            self.source.point(position.start.offset)
        });
        let braced_at = |at: usize| line.get(at) == Some(&'{') && line.get(at + 1) != Some(&'{');
        if braced_at(0) {
            let broke_at = params::braced(&line, 0).broke_at.unwrap_or(0);
            self.diagnose(Code::F011, right(from, broke_at), BROKEN_INLINE);
            component.recovered = true;
            return;
        }
        let spaces = line.iter().take_while(|c| is_space(Some(c))).count();
        if spaces > 0 && braced_at(spaces) && params::braced(&line, spaces).broke_at.is_none() {
            self.diagnose(
                Code::W107,
                right(from, spaces),
                format!(
                    "the space before `{{` makes this param list text; write `:{}[…]{{…}}` without a space",
                    component.name
                ),
            );
        }
    }

    /// A fallback's text as inline content, placed where it is written.
    pub(super) fn fallback(&mut self, text: &str, origin: Point) -> Vec<Node> {
        let first = self.syntax.mdast(text, false).children.into_iter().next();
        let Some(mdast::Node::Paragraph(paragraph)) = first else {
            return Vec::new();
        };
        self.placed(text, origin).inlines(paragraph.children, false)
    }

    /// A URL's parts, when it holds an interpolation (spec §12.2).
    pub(super) fn url_parts(&self, url: &str) -> Option<Vec<StringPart>> {
        let parts = string_parts(url);
        let interpolated = parts
            .iter()
            .any(|part| matches!(part, StringPart::Interpolation(_)));
        (self.syntax.compmd && interpolated).then_some(parts)
    }
}

const BROKEN_INLINE: &str = "malformed param list, so the component keeps no params and the list is text: a quote or bracket that doesn't close, a character a bare value can't hold (quote the value, or write an expression), or `=` with no value";

/// In a table cell, the text with each `\|` as `|`, and for each of its characters the index it had.
fn unescape_pipes(text: &[char], in_table: bool) -> (Vec<char>, Vec<usize>) {
    let mut chars = Vec::new();
    let mut indexes = Vec::new();
    let mut at = 0;
    while at < text.len() {
        if in_table && text[at] == '\\' && text.get(at + 1) == Some(&'|') {
            at += 1;
        }
        chars.push(text[at]);
        indexes.push(at);
        at += 1;
    }
    indexes.push(text.len());
    (chars, indexes)
}

/// `[[target]]`, `[[target|label]]`, or in a table `[[target\|label]]`.
fn wiki_link(raw: &str, position: Option<Position>) -> WikiLink {
    let inner = raw
        .strip_prefix("[[")
        .and_then(|r| r.strip_suffix("]]"))
        .unwrap_or(raw);
    let (target, label) = inner
        .split_once("\\|")
        .or_else(|| inner.split_once('|'))
        .unwrap_or((inner, inner));
    WikiLink {
        target: target.to_string(),
        label: label.to_string(),
        position,
    }
}

fn reference_type(kind: mdast::ReferenceKind) -> ReferenceType {
    match kind {
        mdast::ReferenceKind::Shortcut => ReferenceType::Shortcut,
        mdast::ReferenceKind::Collapsed => ReferenceType::Collapsed,
        mdast::ReferenceKind::Full => ReferenceType::Full,
    }
}

/// Adjacent text nodes as one.
fn merge_texts(nodes: Vec<Node>) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::with_capacity(nodes.len());
    for node in nodes {
        if let (Node::Text(text), Some(Node::Text(previous))) = (&node, out.last_mut()) {
            previous.value.push_str(&text.value);
            previous.position =
                previous
                    .position
                    .zip(text.position)
                    .map(|(first, last)| Position {
                        start: first.start,
                        end: last.end,
                    });
            continue;
        }
        out.push(node);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(value: &str) -> Node {
        Node::Text(Literal {
            value: value.into(),
            position: None,
        })
    }

    #[test]
    fn adjacent_texts_merge() {
        let merged = merge_texts(vec![
            text("a"),
            text("b"),
            Node::Break(Void::default()),
            text("c"),
        ]);
        assert_eq!(
            merged,
            [text("ab"), Node::Break(Void::default()), text("c")]
        );
    }

    #[test]
    fn pipes_unescape_only_in_tables() {
        let chars: Vec<char> = r"{a=(x \|\| y)}".chars().collect();
        let (unescaped, indexes) = unescape_pipes(&chars, true);
        assert_eq!(unescaped.iter().collect::<String>(), "{a=(x || y)}");
        assert_eq!((indexes[6], indexes[7], indexes[12]), (7, 9, 14));
        assert_eq!(unescape_pipes(&chars, false).0, chars);
    }

    #[test]
    fn wiki_links_split_target_and_label() {
        let link = |raw: &str| {
            let link = wiki_link(raw, None);
            (link.target, link.label)
        };
        assert_eq!(link("[[Home]]"), ("Home".into(), "Home".into()));
        assert_eq!(link("[[Home|start]]"), ("Home".into(), "start".into()));
        assert_eq!(link(r"[[Home\|start]]"), ("Home".into(), "start".into()));
    }
}
