//! The document tree (spec §12): mdast nodes for CommonMark and GFM, and CompMD's own nodes.
//!
//! Every type serializes to the JSON of spec §12.4 with `serde`, so `serde_json::to_value(&root)` is the tree
//! `conformance/compmd-tree.schema.json` defines as `#/$defs/parseRoot`.

mod components;
mod definitions;
mod shift;

pub use components::{Container, Interpolation, Leaf, Param, ParamValue, Slot, StringPart};
pub use definitions::{
    Declaration, DeclaredType, DefaultValue, Definition, DefinitionBody, Headings, Signature,
    SignatureFence, SlotDeclaration, TypeName,
};
pub(crate) use shift::{Move, Moved, Shift};

use crate::diagnostic::{Diagnostic, Location};
use serde::Serialize;
use serde_json::Value;

/// A place in a document (unist `point`): lines and columns are 1-based, offsets 0-based, and columns and offsets
/// count Unicode code points (spec §10.5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Point {
    /// The line, from 1.
    pub line: usize,
    /// The column, from 1, in code points.
    pub column: usize,
    /// The code points before the point, from the start of the document.
    pub offset: usize,
}

/// Where a node is in its document (unist `position`), from `start` up to `end`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize)]
pub struct Position {
    /// The first place of the node.
    pub start: Point,
    /// The place just past the node.
    pub end: Point,
}

/// Where a node at `position` starts, as diagnostics name places; the document's start when it has no position.
pub(crate) fn start_of(position: Option<Position>) -> Location {
    position.map_or(Location::new(1, 1), |position| position.start.into())
}

/// The root of a document tree.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(tag = "type", rename = "root")]
pub struct Root {
    /// The document's blocks.
    pub children: Vec<Node>,
    /// The whole document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A node of the document tree, serialized with its `type`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Node {
    /// A block quote; a GitHub alert when `alert` is set (spec §2.3).
    Blockquote(Blockquote),
    /// A hard line break.
    Break(Void),
    /// A fenced or indented code block.
    Code(Code),
    /// A container, `::: name` … `::: /name` (spec §4.1).
    ComponentContainer(Container),
    /// A component definition, a fenced code block with `component=` (spec §7).
    ComponentDefinition(Definition),
    /// An inline component, `:name[fallback]{params}` (spec §4.4).
    ComponentInline(Leaf),
    /// A leaf, `:: name[fallback] {params}` (spec §4.3).
    ComponentLeaf(Leaf),
    /// A signature fence, ```` ```compmd component=name signature ```` (spec §7.1).
    ComponentSignature(SignatureFence),
    /// A named slot of a container, `:: #name` (spec §4.2).
    ComponentSlot(Slot),
    /// A link reference definition, `[label]: url`.
    Definition(LinkDefinition),
    /// GFM strikethrough.
    Delete(Parent),
    /// Emphasis.
    Emphasis(Parent),
    /// A GFM footnote definition, `[^label]: …`.
    FootnoteDefinition(FootnoteDefinition),
    /// A GFM footnote reference, `[^label]`.
    FootnoteReference(Reference),
    /// A heading.
    Heading(Heading),
    /// Raw HTML, a block or inline.
    Html(Literal),
    /// An image.
    Image(Image),
    /// An image by reference, `![alt][label]`.
    ImageReference(ImageReference),
    /// A code span.
    InlineCode(Literal),
    /// Inline math, `$…$` (spec §2.3).
    InlineMath(Literal),
    /// An interpolation, `{{path}}` (spec §8.1).
    Interpolation(Interpolation),
    /// A link.
    Link(Link),
    /// A link by reference, `[text][label]`.
    LinkReference(LinkReference),
    /// A list.
    List(List),
    /// An item of a list.
    ListItem(ListItem),
    /// Display math, `$$…$$` or a ```` ```math ```` fence (spec §2.3).
    Math(Math),
    /// A paragraph.
    Paragraph(Parent),
    /// Strong emphasis.
    Strong(Parent),
    /// A GFM table.
    Table(Table),
    /// A cell of a table row.
    TableCell(Parent),
    /// A row of a table.
    TableRow(Parent),
    /// Text.
    Text(Literal),
    /// A thematic break.
    ThematicBreak(Void),
    /// A wiki link, `[[target|label]]`, when the document opts in (spec §2.3).
    WikiLink(WikiLink),
    /// The frontmatter (spec §2.1).
    Yaml(Yaml),
}

impl Node {
    /// The nodes inside this one: a paragraph's text, a list's items, a container's default slot, a leaf's or an
    /// inline component's fallback, a `compmd` definition's body. A container's named slots are in its `slots`.
    #[must_use]
    pub fn children(&self) -> &[Node] {
        match self {
            Self::Blockquote(Blockquote { children, .. })
            | Self::Delete(Parent { children, .. })
            | Self::Emphasis(Parent { children, .. })
            | Self::Paragraph(Parent { children, .. })
            | Self::Strong(Parent { children, .. })
            | Self::TableCell(Parent { children, .. })
            | Self::TableRow(Parent { children, .. })
            | Self::FootnoteDefinition(FootnoteDefinition { children, .. })
            | Self::Heading(Heading { children, .. })
            | Self::Link(Link { children, .. })
            | Self::LinkReference(LinkReference { children, .. })
            | Self::List(List { children, .. })
            | Self::ListItem(ListItem { children, .. })
            | Self::Table(Table { children, .. })
            | Self::ComponentContainer(Container { children, .. })
            | Self::ComponentSlot(Slot { children, .. })
            | Self::ComponentDefinition(Definition {
                body: DefinitionBody::Compmd { children },
                ..
            }) => children,
            Self::ComponentLeaf(leaf) | Self::ComponentInline(leaf) => &leaf.fallback,
            _ => &[],
        }
    }

    /// Where the node is: for a container, its opener.
    #[must_use]
    pub fn position(&self) -> Option<Position> {
        match self {
            Self::Blockquote(n) => n.position,
            Self::Code(n) => n.position,
            Self::ComponentContainer(n) => n.position,
            Self::ComponentDefinition(n) => n.position,
            Self::ComponentInline(n) | Self::ComponentLeaf(n) => n.position,
            Self::ComponentSignature(n) => n.position,
            Self::ComponentSlot(n) => n.position,
            Self::Definition(n) => n.position,
            Self::Delete(n)
            | Self::Emphasis(n)
            | Self::Paragraph(n)
            | Self::Strong(n)
            | Self::TableCell(n)
            | Self::TableRow(n) => n.position,
            Self::FootnoteDefinition(n) => n.position,
            Self::FootnoteReference(n) => n.position,
            Self::Heading(n) => n.position,
            Self::Html(n) | Self::InlineCode(n) | Self::InlineMath(n) | Self::Text(n) => n.position,
            Self::Image(n) => n.position,
            Self::ImageReference(n) => n.position,
            Self::Interpolation(n) => n.position,
            Self::Link(n) => n.position,
            Self::LinkReference(n) => n.position,
            Self::List(n) => n.position,
            Self::ListItem(n) => n.position,
            Self::Math(n) => n.position,
            Self::Table(n) => n.position,
            Self::Break(n) | Self::ThematicBreak(n) => n.position,
            Self::WikiLink(n) => n.position,
            Self::Yaml(n) => n.position,
        }
    }
}

/// A node with children and nothing else. Like every mdast node with children, it is generic over its children's
/// type, so the expanded tree (spec §12.3) shares it.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Parent<N = Node> {
    /// The node's content.
    pub children: Vec<N>,
    /// Where the node is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A node with a text value and nothing else.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Literal {
    /// The text.
    pub value: String,
    /// Where the node is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A node with no content.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Void {
    /// Where the node is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A block quote.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Blockquote<N = Node> {
    /// The quote's content, without an alert's `[!TYPE]` line.
    pub children: Vec<N>,
    /// The alert type, for `> [!NOTE]` and the like.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<Alert>,
    /// Where the quote is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// The type of a GitHub alert (spec §2.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Alert {
    /// `[!NOTE]`
    Note,
    /// `[!TIP]`
    Tip,
    /// `[!IMPORTANT]`
    Important,
    /// `[!WARNING]`
    Warning,
    /// `[!CAUTION]`
    Caution,
}

/// A code block.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Code {
    /// The first word of the info string.
    pub lang: Option<String>,
    /// The rest of the info string.
    pub meta: Option<String>,
    /// The code.
    pub value: String,
    /// Where the block is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    /// What a code renderer is given when it renders the block (spec §7.7); not in the tree, since a block is content
    /// until a code renderer takes it.
    #[serde(skip)]
    pub(crate) info: Option<Info>,
}

/// A code block as a code renderer's invocation (spec §7.7): its params, the block's text as `code` first, then the
/// info string's; its text alternative; and what was wrong with the info string's params, reported only when a code
/// renderer renders the block.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Info {
    pub(crate) params: Vec<Param>,
    /// The `alt` param as written, whose interpolations are filled in where the block renders (spec §8.1).
    pub(crate) alt: Option<Param>,
    pub(crate) problems: Vec<Diagnostic>,
}

/// Display math.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Math {
    /// What follows `$$` on the opening line.
    pub meta: Option<String>,
    /// The TeX source.
    pub value: String,
    /// Where the math is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A heading.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Heading<N = Node> {
    /// The level, 1 to 6: as written in the parse tree, computed in the expanded tree (spec §7.2).
    pub depth: u8,
    /// The heading's text.
    pub children: Vec<N>,
    /// Where the heading is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A list.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct List<N = Node> {
    /// Whether the list is numbered.
    pub ordered: bool,
    /// The first number of an ordered list.
    pub start: Option<u32>,
    /// Whether blank lines separate its items.
    pub spread: bool,
    /// The items.
    pub children: Vec<N>,
    /// Where the list is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// An item of a list.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ListItem<N = Node> {
    /// Whether blank lines separate its children.
    pub spread: bool,
    /// A GFM task item's state.
    pub checked: Option<bool>,
    /// The item's content.
    pub children: Vec<N>,
    /// Where the item is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A GFM table.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Table<N = Node> {
    /// Each column's alignment, `None` when it has none.
    pub align: Vec<Option<Align>>,
    /// The rows, the header row first.
    pub children: Vec<N>,
    /// Where the table is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// The alignment of a table column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    /// `:--`
    Left,
    /// `--:`
    Right,
    /// `:-:`
    Center,
}

/// A link.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Link<N = Node> {
    /// The link text.
    pub children: Vec<N>,
    /// The destination: as written in the parse tree, with its interpolations in the expanded tree.
    pub url: String,
    /// The title.
    pub title: Option<String>,
    /// The destination's text and interpolation parts, when it holds an interpolation (spec §12.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_parts: Option<Vec<StringPart>>,
    /// Where the link is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// An image.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// The alternative text.
    pub alt: String,
    /// The source as written.
    pub url: String,
    /// The title.
    pub title: Option<String>,
    /// The source's text and interpolation parts, when it holds an interpolation (spec §12.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_parts: Option<Vec<StringPart>>,
    /// Where the image is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A link reference definition.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkDefinition {
    /// The normalized label that references match.
    pub identifier: String,
    /// The label as written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The destination as written.
    pub url: String,
    /// The title.
    pub title: Option<String>,
    /// The destination's text and interpolation parts, when it holds an interpolation (spec §12.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_parts: Option<Vec<StringPart>>,
    /// Where the definition is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// How a reference is written.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReferenceType {
    /// `[label]`
    #[default]
    Shortcut,
    /// `[label][]`
    Collapsed,
    /// `[text][label]`
    Full,
}

/// A link by reference.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkReference<N = Node> {
    /// The link text.
    pub children: Vec<N>,
    /// How the reference is written.
    pub reference_type: ReferenceType,
    /// The normalized label.
    pub identifier: String,
    /// The label as written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Where the reference is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// An image by reference.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageReference {
    /// The alternative text.
    pub alt: String,
    /// How the reference is written.
    pub reference_type: ReferenceType,
    /// The normalized label.
    pub identifier: String,
    /// The label as written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Where the reference is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A footnote definition.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct FootnoteDefinition<N = Node> {
    /// The normalized label.
    pub identifier: String,
    /// The label as written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The note's content.
    pub children: Vec<N>,
    /// Where the definition is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A footnote reference.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Reference {
    /// The normalized label.
    pub identifier: String,
    /// The label as written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Where the reference is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// The frontmatter.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Yaml {
    /// The YAML text.
    pub value: String,
    /// The frontmatter as parsed YAML, each data import's `from` as text and interpolation parts; absent when
    /// CompMD processing doesn't apply (spec §12.2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parsed: Option<Value>,
    /// Where the frontmatter is, with its `---` lines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A wiki link.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct WikiLink {
    /// The page it links to.
    pub target: String,
    /// The text it shows: the part after `|`, or the target.
    pub label: String,
    /// Where the link is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde's skip_serializing_if passes a reference"
)]
fn is_false(value: &bool) -> bool {
    !value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn children_are_a_nodes_content() {
        let paragraph = Node::Paragraph(Parent {
            children: vec![Node::Text(Literal::default())],
            position: None,
        });
        let slot = Node::ComponentSlot(Slot {
            name: "footer".into(),
            children: vec![paragraph.clone()],
            position: None,
        });
        assert_eq!(slot.children(), std::slice::from_ref(&paragraph));
        assert_eq!(paragraph.children().len(), 1);
        assert_eq!(Node::Text(Literal::default()).children(), []);
    }
}
