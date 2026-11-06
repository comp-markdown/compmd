//! Component definitions and their signatures in a document (spec §7, §12.1): the fences, the declared params and
//! slots, and their types.

use super::{Node, Param, ParamValue, Position};
use crate::diagnostic::Location;
use crate::named::named;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_json::Value;

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
    /// For a code renderer, the languages whose code blocks it renders, names of one language (spec §7.7); its
    /// `params` then include `code`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub renders: Option<Vec<String>>,
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

impl Signature {
    /// Whether it declares the slot `name`, as a slot line writes it.
    pub(crate) fn declares_slot(&self, name: &str) -> bool {
        self.slots.iter().flatten().any(|slot| slot.name() == name)
    }
}

impl SlotDeclaration {
    /// The slot's name.
    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Name(name) | Self::Template { name, .. } => name,
        }
    }
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
    /// Where its type is written when that isn't a type name (F037), in the document that declares it.
    #[serde(skip)]
    pub(crate) unknown_at: Option<Location>,
    /// Where its type is written, in the document that declares it, when known.
    #[serde(skip)]
    pub(crate) type_at: Option<Location>,
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
        /// Grid areas: quoted rows of area names, all as long (spec §9.4).
        Areas = "areas",
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

    /// The name it is written with when that isn't a type name: `numbr`, alone or in `input<numbr>`.
    pub(crate) fn unknown(&self) -> Option<&str> {
        match self {
            Self::Other(name) => Some(name),
            Self::Input { input } => input.unknown(),
            Self::Name(_) | Self::Enum { .. } => None,
        }
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
