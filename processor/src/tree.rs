//! The document tree (spec §12): mdast nodes for CommonMark and GFM, and CompMD's own nodes.
//!
//! Every type serializes to the JSON of spec §12.4 with `serde`, so `serde_json::to_value(&root)` is the tree
//! `conformance/compmd-tree.schema.json` defines as `#/$defs/parseRoot`.

mod shift;

pub(crate) use shift::{Move, Shift};

use crate::named::named;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
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

/// An interpolation, `{{path}}`.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Interpolation {
    /// The names joined by `.`, as written.
    pub path: String,
    /// Whether it stands alone in its paragraph, which it replaces (spec §12.2).
    pub block: bool,
    /// Where the interpolation is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A part of a quoted param, a URL or a data import's `from` (spec §12.1, §12.2).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum StringPart {
    /// Text, with escapes resolved.
    Text(Literal),
    /// An interpolation.
    Interpolation(Interpolation),
}

/// A container, `::: name {params}` … `::: /name`.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Container {
    /// The component's name.
    pub name: String,
    /// The params of the opener and its continuation lines.
    pub params: Vec<Param>,
    /// The default slot.
    pub children: Vec<Node>,
    /// The named slots, in source order.
    pub slots: Vec<Slot>,
    /// Where the opener is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    /// Where the closer is; absent when recovery closed the container.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_position: Option<Position>,
    /// Whether recovery closed the container (spec §10.3).
    #[serde(skip_serializing_if = "is_false")]
    pub auto_closed: bool,
    /// Whether a recovery rule made it, such as params that broke (spec §10.3).
    #[serde(skip_serializing_if = "is_false")]
    pub recovered: bool,
}

/// A named slot of a container.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(tag = "type", rename = "componentSlot")]
pub struct Slot {
    /// The slot's name.
    pub name: String,
    /// The slot's content.
    pub children: Vec<Node>,
    /// Where the slot line is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A leaf or an inline component.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Leaf {
    /// The component's name.
    pub name: String,
    /// The fallback, as inline content; empty when a leaf has none.
    pub fallback: Vec<Node>,
    /// The params, with a leaf's continuation lines.
    pub params: Vec<Param>,
    /// Where the component is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    /// Whether a recovery rule made it, such as params that broke (spec §10.3).
    #[serde(skip_serializing_if = "is_false")]
    pub recovered: bool,
}

/// A param as written (spec §12.1).
#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    /// The key; `None` for a class or an id.
    pub key: Option<String>,
    /// The breakpoint prefix, `md` in `md:gap=l`.
    pub breakpoint: Option<String>,
    /// The value and its kind.
    pub value: ParamValue,
    /// Where the param is.
    pub position: Option<Position>,
}

/// A param's value as written, by its kind (spec §3.1, §12.1).
#[derive(Clone, Debug, PartialEq)]
pub enum ParamValue {
    /// `icon=rocket`: the text, typed by the signature later.
    Bare(String),
    /// `title="Step {{n}}"`: text and interpolation parts, escapes resolved.
    String(Vec<StringPart>),
    /// `tags=["a"]`: the JSON text.
    Json(String),
    /// `count=(size(rows))`: the expression without its outer parentheses.
    Expr(String),
    /// `open`: a key with no value.
    Flag,
    /// `.wide`: the class name.
    Class(String),
    /// `#intro`: the id.
    Id(String),
}

impl Param {
    /// Whether it is `display=contents`, at any breakpoint. Only a bare one on a `compmd` definition fence takes the
    /// component's box away (spec §9.2); anywhere else it is F009.
    pub(crate) fn is_display_contents(&self) -> bool {
        self.key.as_deref() == Some("display") && self.value == ParamValue::Bare("contents".into())
    }
}

impl ParamValue {
    /// The kind's name in the tree: `bare`, `string`, `json`, `expr`, `flag`, `class` or `id`.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Bare(_) => "bare",
            Self::String(_) => "string",
            Self::Json(_) => "json",
            Self::Expr(_) => "expr",
            Self::Flag => "flag",
            Self::Class(_) => "class",
            Self::Id(_) => "id",
        }
    }

    fn serialize_value<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
        match self {
            Self::Bare(text)
            | Self::Json(text)
            | Self::Expr(text)
            | Self::Class(text)
            | Self::Id(text) => map.serialize_entry("value", text),
            Self::String(parts) => map.serialize_entry("value", parts),
            Self::Flag => map.serialize_entry("value", &()),
        }
    }
}

impl Serialize for Param {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("key", &self.key)?;
        map.serialize_entry("breakpoint", &self.breakpoint)?;
        map.serialize_entry("kind", self.value.kind())?;
        self.value.serialize_value(&mut map)?;
        if let Some(position) = &self.position {
            map.serialize_entry("position", position)?;
        }
        map.end()
    }
}

/// A component definition (spec §7).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Definition {
    /// The component's name, from `component=`.
    pub name: String,
    /// The source language, the info string's first word.
    pub language: String,
    /// The header of a `compmd` definition; `None` for a host language, whose signature comes from a signature
    /// fence.
    pub signature: Option<Signature>,
    /// The param list that ends the info string, which styles the component's box.
    pub params: Vec<Param>,
    /// The body.
    #[serde(flatten)]
    pub body: DefinitionBody,
    /// Where the fence is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// The body of a component definition.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DefinitionBody {
    /// A `compmd` definition's body, parsed as a document of its own.
    Compmd {
        /// The body's blocks.
        children: Vec<Node>,
    },
    /// A host-language definition's source.
    Host {
        /// The code.
        source: String,
        /// The `runtime=` attribute.
        runtime: Option<String>,
    },
}

/// A signature fence.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SignatureFence {
    /// The component it gives a signature to.
    pub name: String,
    /// The signature; `None` when the fence isn't valid YAML.
    pub signature: Option<Signature>,
    /// Where the fence is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A signature (spec §7.3).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Signature {
    /// The declared params, in written order.
    pub params: Vec<Declaration>,
    /// The declared slots; `None` when the signature has no `slots` key, so the component takes no body.
    pub slots: Option<Vec<SlotDeclaration>>,
    /// How headings in a `compmd` body are leveled.
    pub headings: Headings,
}

/// A slot of a signature.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SlotDeclaration {
    /// A slot by its name; `default` is the default slot.
    Name(String),
    /// A template slot, rendered once per element of the array param `each`, bound to the name param `as`
    /// (spec §7.5).
    Template {
        /// The slot's name.
        name: String,
        /// The array param it iterates.
        each: String,
        /// The `name` param that names the element.
        #[serde(rename = "as")]
        binding: String,
    },
}

/// How headings in a `compmd` definition's body are leveled (spec §7.2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Headings {
    /// Shifted to fit under the heading the invocation is under.
    #[default]
    Relative,
    /// Kept at their written level.
    Absolute,
}

/// A declared param (spec §7.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Declaration {
    /// The param's name.
    pub name: String,
    /// Its type.
    #[serde(rename = "type")]
    pub declared: DeclaredType,
    /// Whether its type ends in `?`.
    pub optional: bool,
    /// Its default value.
    pub default: Option<DefaultValue>,
    /// The long form's `doc`.
    pub doc: Option<String>,
}

named! {
    /// A type name of a declaration (spec §7.3), and `enum`, which `input<enum>` binds.
    pub enum TypeName {
        /// Text.
        String = "string",
        /// A number, a CEL `double`.
        Number = "number",
        /// `true` or `false`.
        Boolean = "boolean",
        /// A day, `YYYY-MM-DD`.
        Date = "date",
        /// A list of any values.
        Array = "array",
        /// A map from strings to any values.
        Object = "object",
        /// A length: `px`, `rem`, `em` or `%` (spec §9.4).
        Length = "length",
        /// A space: `none`, `s`, `m`, `l` or a length (spec §9.4).
        Space = "space",
        /// A CSS color (spec §9.4).
        Color = "color",
        /// Grid tracks: lengths, `fr`, `auto`, `minmax()`, `repeat()` … (spec §9.4).
        Tracks = "tracks",
        /// A background: a token or a CSS color (spec §9.4).
        Surface = "surface",
        /// A border: a token or a CSS border (spec §9.4).
        Border = "border",
        /// A corner radius: a token or a length (spec §9.4).
        Radius = "radius",
        /// A name the author binds, written as is: never read as an expression.
        Name = "name",
        /// Any enum, in `input<enum>`.
        Enum = "enum",
    }
}

impl Serialize for TypeName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

/// A declared type (spec §7.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DeclaredType {
    /// A type name: `string`, `length`, ….
    Name(TypeName),
    /// A name that isn't one of the type names, as written: any value fits it, as `dyn`.
    Other(String),
    /// An enum, `a | b`.
    Enum {
        /// The members, in written order.
        #[serde(rename = "enum")]
        members: Vec<String>,
    },
    /// A bound input, `input<type>`.
    Input {
        /// The input's type.
        input: Box<DeclaredType>,
    },
}

impl DeclaredType {
    /// The type a name stands for: one of the type names, or another name as written.
    pub(crate) fn named(name: &str) -> Self {
        TypeName::from_name(name).map_or_else(|| Self::Other(name.to_string()), Self::Name)
    }

    /// The type as messages write it: `a number`, `one of a, b`, `an input of a date`.
    pub(crate) fn written(&self) -> String {
        match self {
            Self::Name(name) => format!("a {}", name.name()),
            Self::Other(name) => format!("a {name}"),
            Self::Enum { members } => format!("one of {}", members.join(", ")),
            Self::Input { input } => format!("an input of {}", input.written()),
        }
    }
}

/// A declared default (spec §12.1).
#[derive(Clone, Debug, PartialEq)]
pub enum DefaultValue {
    /// A short form's default, a param value.
    Param(ParamValue),
    /// A long form's default, a YAML value.
    Yaml(Value),
}

impl Serialize for DefaultValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        match self {
            Self::Param(value) => {
                map.serialize_entry("kind", value.kind())?;
                value.serialize_value(&mut map)?;
            }
            Self::Yaml(value) => {
                map.serialize_entry("kind", "yaml")?;
                map.serialize_entry("value", value)?;
            }
        }
        map.end()
    }
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
