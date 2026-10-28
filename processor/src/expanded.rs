//! The expanded tree (spec §12.3), what stage 4b makes of a document: content only, with every `compmd` definition,
//! `if`, `each`, `value` and interpolation replaced by what it produces. Components stay as nodes only where the
//! processor renders them itself: `box`, the input components, and host-language components, with their params
//! evaluated.
//!
//! It shares the mdast node types of the [parse tree](crate::tree), with expanded children, and serializes to the
//! JSON `conformance/compmd-tree.schema.json` defines as `#/$defs/expandedRoot`.

use crate::cel;
use crate::tree::{
    Blockquote, Code, FootnoteDefinition, Heading, Image, ImageReference, Link, LinkDefinition,
    LinkReference, List, ListItem, Literal, Math, Parent, Point, Reference, Table, Void, WikiLink,
};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_json::Value;

/// A value as the expanded tree writes it: as JSON, with a whole `double` written as an integer (`5`, not `5.0`),
/// as JavaScript writes numbers.
pub(crate) fn json(value: &cel::Value) -> Value {
    /// The largest integer every `double` up to it holds exactly.
    const EXACT: f64 = 9_007_199_254_740_992.0;
    match value {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "the double is whole and within ±2^53, so exact"
        )]
        cel::Value::Double(d) if d.fract() == 0.0 && d.abs() <= EXACT => Value::from(*d as i64),
        cel::Value::List(items) => Value::Array(items.iter().map(json).collect()),
        cel::Value::Map(entries) => Value::Object(
            entries
                .iter()
                .map(|(key, value)| (key.to_value().text(), json(value)))
                .collect(),
        ),
        other => other.to_json(),
    }
}

/// The root of an expanded tree.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(tag = "type", rename = "root")]
pub struct Root {
    /// The document's content.
    pub children: Vec<Node>,
    /// The document's inputs, in declared order, with the values the render used; absent when it has none.
    #[serde(skip_serializing_if = "Inputs::is_empty")]
    pub inputs: Inputs,
}

/// A document's inputs, serialized as a map from each name to its state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Inputs(pub(crate) Vec<Input>);

impl Inputs {
    /// Whether the document declares no input.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The inputs, in declared order.
    pub fn iter(&self) -> std::slice::Iter<'_, Input> {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a Inputs {
    type Item = &'a Input;
    type IntoIter = std::slice::Iter<'a, Input>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Serialize for Inputs {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for input in self {
            map.serialize_entry(&input.name, input)?;
        }
        map.end()
    }
}

/// An input of the document (spec §8.7), as input components read it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Input {
    /// Its name.
    #[serde(skip)]
    pub name: String,
    /// Its type: a type name, or `enum`.
    #[serde(rename = "type")]
    pub kind: String,
    /// An enum's members, its choices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<String>>,
    /// The value the render used: the one given, or the default.
    pub value: Value,
}

/// A node of the expanded tree.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Node {
    /// What the node is.
    #[serde(flatten)]
    pub content: Content,
    /// The range of the processed document that made it (the `origins` feature, spec §12.3).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
}

impl Node {
    /// A node without an origin.
    pub(crate) fn new(content: Content) -> Self {
        Self {
            content,
            origin: None,
        }
    }
}

/// What an expanded node is, serialized with its `type`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Content {
    /// A block quote.
    Blockquote(Blockquote<Node>),
    /// A hard line break.
    Break(Void),
    /// A code block.
    Code(Code),
    /// A `box`, or a host-language component that takes a body.
    ComponentContainer(Container),
    /// An inline component the processor renders: an input component, or a host-language one.
    ComponentInline(Leaf),
    /// A leaf the processor renders: an input component, or a host-language one.
    ComponentLeaf(Leaf),
    /// A link reference definition.
    Definition(LinkDefinition),
    /// GFM strikethrough.
    Delete(Parent<Node>),
    /// Emphasis.
    Emphasis(Parent<Node>),
    /// A GFM footnote definition.
    FootnoteDefinition(FootnoteDefinition<Node>),
    /// A GFM footnote reference.
    FootnoteReference(Reference),
    /// A heading, at its computed depth.
    Heading(Heading<Node>),
    /// Raw HTML.
    Html(Literal),
    /// An image.
    Image(Image),
    /// An image by reference.
    ImageReference(ImageReference),
    /// A code span.
    InlineCode(Literal),
    /// Inline math.
    InlineMath(Literal),
    /// A link.
    Link(Link<Node>),
    /// A link by reference.
    LinkReference(LinkReference<Node>),
    /// A list.
    List(List<Node>),
    /// An item of a list.
    ListItem(ListItem<Node>),
    /// Display math.
    Math(Math),
    /// A paragraph.
    Paragraph(Parent<Node>),
    /// Strong emphasis.
    Strong(Parent<Node>),
    /// A GFM table.
    Table(Table<Node>),
    /// A cell of a table row.
    TableCell(Parent<Node>),
    /// A row of a table.
    TableRow(Parent<Node>),
    /// Text.
    Text(Literal),
    /// A thematic break.
    ThematicBreak(Void),
    /// A wiki link.
    WikiLink(WikiLink),
}

impl Content {
    /// Its children: a parent's content, a container's default slot, a leaf's fallback.
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
            | Self::ComponentContainer(Container { children, .. }) => children,
            Self::ComponentLeaf(leaf) | Self::ComponentInline(leaf) => &leaf.fallback,
            _ => &[],
        }
    }

    /// Its children, to change them.
    pub(crate) fn children_mut(&mut self) -> Option<&mut Vec<Node>> {
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
            | Self::ComponentContainer(Container { children, .. }) => Some(children),
            Self::ComponentLeaf(leaf) | Self::ComponentInline(leaf) => Some(&mut leaf.fallback),
            _ => None,
        }
    }
}

/// A component container the processor renders.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Container {
    /// The component's name: `box`, or the host-language component's.
    pub name: String,
    /// Its params, evaluated.
    pub params: Vec<Param>,
    /// The default slot, expanded; empty for a template slot, whose content is in `instances`.
    pub children: Vec<Node>,
    /// A template default slot's content, once per element (spec §7.5).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instances: Option<Vec<Instance>>,
    /// The named slots, expanded.
    pub slots: Vec<Slot>,
}

/// A named slot of a component container.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(tag = "type", rename = "componentSlot")]
pub struct Slot {
    /// The slot's name.
    pub name: String,
    /// Its content, expanded; empty for a template slot.
    pub children: Vec<Node>,
    /// A template slot's content, once per element.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instances: Option<Vec<Instance>>,
    /// The range of the processed document that made it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<Origin>,
}

/// A template slot's content for one element (spec §7.5).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Instance {
    /// The element.
    pub value: Value,
    /// The slot's content, expanded with the element bound.
    pub children: Vec<Node>,
}

/// A leaf or an inline component the processor renders.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Leaf {
    /// The component's name.
    pub name: String,
    /// Its params, evaluated.
    pub params: Vec<Param>,
    /// The fallback, expanded.
    pub fallback: Vec<Node>,
}

/// An evaluated param (spec §12.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Param {
    /// The key; `class` for all the classes, `id` for `#name`.
    pub key: String,
    /// The breakpoint it applies from.
    pub breakpoint: Option<String>,
    /// The value, as JSON; whole numbers are written without a fraction.
    pub value: Value,
    /// For a param bound to an input, the input's name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
}

/// The range of the processed document that made an expanded node, and how many nodes before it in the same list
/// have the same range (spec §12.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct Origin {
    /// Where the range starts.
    pub start: Point,
    /// Where it ends.
    pub end: Point,
    /// How many nodes before this one, in the same list of children, have the same range.
    pub occurrence: usize,
}
