//! The document tree (spec §12): mdast nodes for CommonMark and GFM, and CompMD's own nodes.
//!
//! Every type serializes to the JSON of spec §12.4 with `serde`, so `serde_json::to_value(&root)` is the tree
//! `conformance/compmd-tree.schema.json` defines as `#/$defs/parseRoot`.

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

/// A node with children and nothing else.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Parent {
    /// The node's content.
    pub children: Vec<Node>,
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
pub struct Blockquote {
    /// The quote's content, without an alert's `[!TYPE]` line.
    pub children: Vec<Node>,
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
pub struct Heading {
    /// The level as written, 1 to 6.
    pub depth: u8,
    /// The heading's text.
    pub children: Vec<Node>,
    /// Where the heading is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A list.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct List {
    /// Whether the list is numbered.
    pub ordered: bool,
    /// The first number of an ordered list.
    pub start: Option<u32>,
    /// Whether blank lines separate its items.
    pub spread: bool,
    /// The items.
    pub children: Vec<Node>,
    /// Where the list is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// An item of a list.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ListItem {
    /// Whether blank lines separate its children.
    pub spread: bool,
    /// A GFM task item's state.
    pub checked: Option<bool>,
    /// The item's content.
    pub children: Vec<Node>,
    /// Where the item is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
}

/// A GFM table.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Table {
    /// Each column's alignment, `None` when it has none.
    pub align: Vec<Option<Align>>,
    /// The rows, the header row first.
    pub children: Vec<Node>,
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
pub struct Link {
    /// The link text.
    pub children: Vec<Node>,
    /// The destination as written.
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
pub struct LinkReference {
    /// The link text.
    pub children: Vec<Node>,
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
pub struct FootnoteDefinition {
    /// The normalized label.
    pub identifier: String,
    /// The label as written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The note's content.
    pub children: Vec<Node>,
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

/// A declared type (spec §7.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DeclaredType {
    /// A type name: `string`, `length`, … as written.
    Name(String),
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
    use serde_json::json;

    fn text(value: &str) -> StringPart {
        StringPart::Text(Literal {
            value: value.to_string(),
            position: None,
        })
    }

    #[test]
    fn params_serialize_with_their_kind_and_value() {
        let params = [
            (ParamValue::Bare("rocket".into()), json!("rocket")),
            (
                ParamValue::String(vec![text("Setup")]),
                json!([{ "type": "text", "value": "Setup" }]),
            ),
            (ParamValue::Flag, json!(null)),
            (ParamValue::Class("wide".into()), json!("wide")),
        ];
        for (value, expected) in params {
            let kind = value.kind();
            let param = Param {
                key: Some("k".into()),
                breakpoint: None,
                value,
                position: None,
            };
            assert_eq!(
                serde_json::to_value(&param).unwrap(),
                json!({ "key": "k", "breakpoint": null, "kind": kind, "value": expected })
            );
        }
    }

    #[test]
    fn a_container_leaves_out_recovery_fields_it_does_not_have() {
        let container = Node::ComponentContainer(Container {
            name: "card".into(),
            ..Container::default()
        });
        assert_eq!(
            serde_json::to_value(&container).unwrap(),
            json!({ "type": "componentContainer", "name": "card", "params": [], "children": [], "slots": [] })
        );
    }

    #[test]
    fn definitions_serialize_their_body_flat() {
        let host = Node::ComponentDefinition(Definition {
            name: "chart".into(),
            language: "jsx".into(),
            signature: None,
            params: Vec::new(),
            body: DefinitionBody::Host {
                source: "x".into(),
                runtime: None,
            },
            position: None,
        });
        assert_eq!(
            serde_json::to_value(&host).unwrap(),
            json!({ "type": "componentDefinition", "name": "chart", "language": "jsx", "signature": null, "params": [], "source": "x", "runtime": null })
        );
    }

    #[test]
    fn declarations_serialize_types_and_defaults() {
        let signature = Signature {
            params: vec![
                Declaration {
                    name: "tone".into(),
                    declared: DeclaredType::Enum {
                        members: vec!["info".into(), "warn".into()],
                    },
                    optional: false,
                    default: Some(DefaultValue::Param(ParamValue::Bare("info".into()))),
                    doc: None,
                },
                Declaration {
                    name: "pick".into(),
                    declared: DeclaredType::Input {
                        input: Box::new(DeclaredType::Name("date".into())),
                    },
                    optional: true,
                    default: Some(DefaultValue::Yaml(json!({ "a": 1 }))),
                    doc: Some("Picks".into()),
                },
            ],
            slots: Some(vec![
                SlotDeclaration::Name("default".into()),
                SlotDeclaration::Template {
                    name: "rows".into(),
                    each: "items".into(),
                    binding: "row".into(),
                },
            ]),
            headings: Headings::Absolute,
        };
        assert_eq!(
            serde_json::to_value(&signature).unwrap(),
            json!({
                "params": [
                    { "name": "tone", "type": { "enum": ["info", "warn"] }, "optional": false, "default": { "kind": "bare", "value": "info" }, "doc": null },
                    { "name": "pick", "type": { "input": "date" }, "optional": true, "default": { "kind": "yaml", "value": { "a": 1 } }, "doc": "Picks" },
                ],
                "slots": ["default", { "name": "rows", "each": "items", "as": "row" }],
                "headings": "absolute",
            })
        );
    }
}
