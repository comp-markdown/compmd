//! CompMD's own nodes in a document (spec §12.1, §12.2): interpolations, components, their slots and their params.

use super::{Literal, Node, Position, is_false};
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

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

    /// Writes the value into `map` as its `value` entry, as a param and a declared default have it.
    pub(super) fn serialize_value<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
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
