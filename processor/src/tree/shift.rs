//! Moving positions: after an edit, the nodes after it keep their shape and move by the edit's change in lines and
//! offsets. Columns stay, since each node after an edit starts on a line of its own.

use super::{
    DefaultValue, DefinitionBody, Info, Leaf, Node, Param, ParamValue, Point, Position, Signature,
    StringPart,
};
use crate::diagnostic::{Diagnostic, Location, Range};

/// How far an edit moved what follows it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Move {
    /// Lines added, or removed when negative.
    pub(crate) lines: isize,
    /// Code points added, or removed when negative.
    pub(crate) offset: isize,
}

/// How an edit moved a document: what is at or after `from` in the old text moved `by`, what is before it stayed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Moved {
    pub(crate) from: Point,
    pub(crate) by: Move,
}

impl Moved {
    /// Nothing moved.
    pub(crate) const NOTHING: Self = Self {
        from: Point {
            line: usize::MAX,
            column: usize::MAX,
            offset: usize::MAX,
        },
        by: Move {
            lines: 0,
            offset: 0,
        },
    };

    /// Whether `point` of the old text moved.
    pub(crate) fn moves(&self, point: Point) -> bool {
        point.offset >= self.from.offset
    }

    /// `point`, moved if it is at or after `from`.
    pub(crate) fn point(&self, point: &mut Point) {
        if self.moves(*point) {
            point.shift(self.by);
        }
    }

    /// `location`, in the processed document, moved if it is on `from`'s line or after it: whatever is on that line
    /// is at or after `from`.
    pub(crate) fn location(&self, location: &mut Location) {
        if location.line >= self.from.line {
            location.line = location.line.saturating_add_signed(self.by.lines);
        }
    }

    /// The places of `diagnostic` in the processed document, each moved if it is on `from`'s line or after it.
    pub(crate) fn diagnostic(&self, diagnostic: &mut Diagnostic) {
        for range in main_ranges(diagnostic) {
            if range.start.line >= self.from.line {
                shift_range(range, self.by);
            }
        }
    }
}

/// What has positions that an edit before it moves.
pub(crate) trait Shift {
    fn shift(&mut self, by: Move);
}

/// A diagnostic's places in the processed document: its own and its fix's when it is there, and those of its related
/// places there.
fn main_ranges(diagnostic: &mut Diagnostic) -> impl Iterator<Item = &mut Range> {
    let here = diagnostic.file.is_none();
    let own = here.then_some(&mut diagnostic.range);
    let fix = diagnostic
        .fix
        .as_mut()
        .filter(|_| here)
        .map(|fix| &mut fix.range);
    own.into_iter().chain(fix).chain(
        diagnostic
            .related
            .iter_mut()
            .filter(|related| related.file.is_none())
            .map(|related| &mut related.range),
    )
}

fn shift_range(range: &mut Range, by: Move) {
    range.start.line = range.start.line.saturating_add_signed(by.lines);
    if let Some(end) = &mut range.end {
        end.line = end.line.saturating_add_signed(by.lines);
    }
}

/// A diagnostic's places in the processed document move; those in other documents stay.
impl Shift for Diagnostic {
    fn shift(&mut self, by: Move) {
        for range in main_ranges(self) {
            shift_range(range, by);
        }
    }
}

impl Shift for Point {
    fn shift(&mut self, by: Move) {
        self.line = self.line.saturating_add_signed(by.lines);
        self.offset = self.offset.saturating_add_signed(by.offset);
    }
}

impl Shift for Location {
    fn shift(&mut self, by: Move) {
        self.line = self.line.saturating_add_signed(by.lines);
    }
}

impl Shift for Position {
    fn shift(&mut self, by: Move) {
        self.start.shift(by);
        self.end.shift(by);
    }
}

impl<T: Shift> Shift for Option<T> {
    fn shift(&mut self, by: Move) {
        if let Some(value) = self {
            value.shift(by);
        }
    }
}

impl<T: Shift> Shift for [T] {
    fn shift(&mut self, by: Move) {
        for value in self {
            value.shift(by);
        }
    }
}

impl<T: Shift> Shift for Vec<T> {
    fn shift(&mut self, by: Move) {
        self.as_mut_slice().shift(by);
    }
}

impl Shift for StringPart {
    fn shift(&mut self, by: Move) {
        match self {
            Self::Text(text) => text.position.shift(by),
            Self::Interpolation(interpolation) => interpolation.position.shift(by),
        }
    }
}

impl Shift for ParamValue {
    fn shift(&mut self, by: Move) {
        if let Self::String(parts) = self {
            parts.shift(by);
        }
    }
}

impl Shift for Param {
    fn shift(&mut self, by: Move) {
        self.value.shift(by);
        self.position.shift(by);
    }
}

impl Shift for Signature {
    fn shift(&mut self, by: Move) {
        for declaration in &mut self.params {
            if let Some(DefaultValue::Param(value)) = &mut declaration.default {
                value.shift(by);
            }
            declaration.unknown_at.shift(by);
            declaration.type_at.shift(by);
        }
    }
}

impl Shift for Info {
    fn shift(&mut self, by: Move) {
        self.params.shift(by);
        self.problems.shift(by);
    }
}

impl Shift for Leaf {
    fn shift(&mut self, by: Move) {
        self.fallback.shift(by);
        self.params.shift(by);
        self.position.shift(by);
    }
}

impl Shift for Node {
    fn shift(&mut self, by: Move) {
        match self {
            Self::Blockquote(quote) => {
                quote.children.shift(by);
                quote.position.shift(by);
            }
            Self::Break(void) | Self::ThematicBreak(void) => void.position.shift(by),
            Self::Code(code) => {
                code.info.shift(by);
                code.position.shift(by);
            }
            Self::ComponentContainer(container) => {
                container.params.shift(by);
                container.children.shift(by);
                for slot in &mut container.slots {
                    slot.children.shift(by);
                    slot.position.shift(by);
                }
                container.position.shift(by);
                container.close_position.shift(by);
            }
            Self::ComponentDefinition(definition) => {
                definition.signature.shift(by);
                definition.params.shift(by);
                if let DefinitionBody::Compmd { children } = &mut definition.body {
                    children.shift(by);
                }
                definition.position.shift(by);
            }
            Self::ComponentInline(leaf) | Self::ComponentLeaf(leaf) => leaf.shift(by),
            Self::ComponentSignature(fence) => {
                fence.signature.shift(by);
                fence.position.shift(by);
            }
            Self::ComponentSlot(slot) => {
                slot.children.shift(by);
                slot.position.shift(by);
            }
            Self::Definition(definition) => {
                definition.url_parts.shift(by);
                definition.position.shift(by);
            }
            Self::Delete(parent)
            | Self::Emphasis(parent)
            | Self::Paragraph(parent)
            | Self::Strong(parent)
            | Self::TableCell(parent)
            | Self::TableRow(parent) => {
                parent.children.shift(by);
                parent.position.shift(by);
            }
            Self::FootnoteDefinition(footnote) => {
                footnote.children.shift(by);
                footnote.position.shift(by);
            }
            Self::FootnoteReference(reference) => reference.position.shift(by),
            Self::Heading(heading) => {
                heading.children.shift(by);
                heading.position.shift(by);
            }
            Self::Html(literal)
            | Self::InlineCode(literal)
            | Self::InlineMath(literal)
            | Self::Text(literal) => literal.position.shift(by),
            Self::Image(image) => {
                image.url_parts.shift(by);
                image.position.shift(by);
            }
            Self::ImageReference(reference) => reference.position.shift(by),
            Self::Interpolation(interpolation) => interpolation.position.shift(by),
            Self::Link(link) => {
                link.children.shift(by);
                link.url_parts.shift(by);
                link.position.shift(by);
            }
            Self::LinkReference(reference) => {
                reference.children.shift(by);
                reference.position.shift(by);
            }
            Self::List(list) => {
                list.children.shift(by);
                list.position.shift(by);
            }
            Self::ListItem(item) => {
                item.children.shift(by);
                item.position.shift(by);
            }
            Self::Math(math) => math.position.shift(by),
            Self::Table(table) => {
                table.children.shift(by);
                table.position.shift(by);
            }
            Self::WikiLink(link) => link.position.shift(by),
            Self::Yaml(yaml) => yaml.position.shift(by),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Code, Location, Related};
    use crate::{ParseOptions, Processor};

    fn blocks(text: &str) -> Vec<Node> {
        Processor::new("a.comp.md", text)
            .parse(&ParseOptions::default())
            .tree()
            .children
            .clone()
    }

    #[test]
    fn every_position_moves_by_the_lines_and_offsets() {
        let body = "::: card {title=\"Step {{n}}\" .wide}\n:: #foot\n> **a** [b](./{{x}}.html) ![c](d.png) ~~e~~ `f`\n::: /card\n\n:: chart[A [link][l] and :badge[x]{tone=\"{{t}}\"}] {rows=(r)}\n\n```compmd component=note {padding=s}\nparams:\n  title: string = \"Hi {{x}}\"\n---\n# {{title}}\n\n- [ ] item\n```\n\n```js component=chip\nexport default 1\n```\n\n```compmd component=chip signature\nparams:\n  tone: string = \"a\"\n```\n\n| a | b |\n|---|:-:|\n| $x$ | <i>y</i> |\n\n[l]: ./{{y}}\n\n[^1]: Note  \nend\n\n$$\nx\n$$\n\n***\n\n![r][l] [^1] {{z}}\n\n1. one\n";
        let mut moved = blocks(&format!("x\n\n{body}"));
        moved.remove(0);
        let mut shifted = blocks(body);
        shifted.shift(Move {
            lines: 2,
            offset: 3,
        });
        assert_eq!(shifted, moved);
        shifted.shift(Move {
            lines: -2,
            offset: -3,
        });
        assert_eq!(shifted, blocks(body));
    }

    #[test]
    fn wiki_links_and_slots_move() {
        let text = "---\nmd:\n  syntax: [wiki-links]\n---\n\nSee [[Home]].\n";
        let mut nodes = blocks(text);
        let Node::Paragraph(paragraph) = &nodes[1] else {
            panic!("a paragraph");
        };
        let mut slot = Node::ComponentSlot(super::super::Slot {
            name: "foot".into(),
            children: paragraph.children.clone(),
            position: paragraph.position,
        });
        let by = Move {
            lines: 1,
            offset: 1,
        };
        nodes.shift(by);
        slot.shift(by);
        let Node::Paragraph(moved) = &nodes[1] else {
            panic!("a paragraph");
        };
        assert!(matches!(moved.children[1], Node::WikiLink(_)));
        assert_eq!(moved.children[1].position().unwrap().start.line, 7);
        assert_eq!(slot.position().unwrap().start.line, 7);
        assert_eq!(slot.children()[1], moved.children[1]);
    }

    fn place(line: usize) -> Range {
        Range {
            start: Location::new(line, 2),
            end: Some(Location::new(line + 1, 1)),
        }
    }

    /// A diagnostic at line 3, with related places at line 5 of the processed document and of another.
    fn diagnostic(file: Option<&str>) -> Diagnostic {
        let mut diagnostic = Diagnostic::new(Code::F004, Location::new(3, 2), "x");
        diagnostic.file = file.map(String::from);
        diagnostic.range = place(3);
        diagnostic.related = vec![
            Related {
                file: None,
                range: place(5),
                message: "here".into(),
            },
            Related {
                file: Some("other.comp.md".into()),
                range: place(5),
                message: "there".into(),
            },
        ];
        diagnostic
    }

    #[test]
    fn diagnostics_move_with_their_ends_and_related_places() {
        let by = Move {
            lines: 2,
            offset: 9,
        };
        let mut moved = diagnostic(None);
        moved.shift(by);
        assert_eq!(moved.range, place(5));
        assert_eq!(moved.related[0].range, place(7));
        assert_eq!(
            moved.related[1].range,
            place(5),
            "another file's place stays"
        );
        let mut elsewhere = diagnostic(Some("other.comp.md"));
        elsewhere.shift(by);
        assert_eq!(
            elsewhere.range,
            place(3),
            "a diagnostic of another file stays"
        );
        assert_eq!(elsewhere.related[0].range, place(7));
    }

    #[test]
    fn only_what_is_after_the_start_of_a_move_moves() {
        let from = Point {
            line: 4,
            column: 1,
            offset: 20,
        };
        let moved = Moved {
            from,
            by: Move {
                lines: 1,
                offset: 3,
            },
        };
        let mut diagnostic = diagnostic(None);
        moved.diagnostic(&mut diagnostic);
        assert_eq!(diagnostic.range, place(3), "before the move");
        assert_eq!(diagnostic.related[0].range, place(6), "after it");
        let mut before = Point { offset: 19, ..from };
        moved.point(&mut before);
        assert_eq!(before.offset, 19);
        let mut at = from;
        moved.point(&mut at);
        assert_eq!((at.line, at.offset), (5, 23));
        let mut far = from;
        Moved::NOTHING.point(&mut far);
        assert_eq!(far, from);
    }

    #[test]
    fn frontmatter_moves_too() {
        let mut yaml = blocks("---\na: 1\n---\n");
        yaml.shift(Move {
            lines: 1,
            offset: 2,
        });
        let position = yaml[0].position().unwrap();
        assert_eq!((position.start.line, position.start.offset), (2, 2));
        assert_eq!((position.end.line, position.end.column), (4, 4));
    }
}
