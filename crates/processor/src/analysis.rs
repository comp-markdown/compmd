//! Component analysis (spec §7.1, §13.1): each host-language component checked against its signature fence, W104
//! where they disagree. The processor reads no JavaScript: the host gives what a component takes, read from its code
//! ([`ComponentProps`]), and only what the code states is checked: a prop's name always, its type where the code
//! gives one.
//!
//! ```
//! use compmd::analysis::{ComponentProps, Prop, PropType, Props};
//! use compmd::Language;
//!
//! /// A reader that knows one component: a badge taking a `label`.
//! struct Badge;
//!
//! impl ComponentProps for Badge {
//!     fn props(&self, _language: Language, code: &str) -> Option<Props> {
//!         let label = Prop { name: "label".into(), kind: Some(PropType::String), ..Prop::default() };
//!         code.contains("Badge").then(|| Props { props: vec![label], rest: false })
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};

use crate::resolve::Language;
use crate::tree::{DeclaredType, Signature, TypeName};

/// What a host-language component takes, read from its code by the host: a component reader.
pub trait ComponentProps {
    /// What the component of `language` whose code is `code` takes, or `None` when its props can't be read, which
    /// leaves it unchecked.
    fn props(&self, language: Language, code: &str) -> Option<Props>;
}

/// A reader a host shares.
impl<T: ComponentProps + ?Sized> ComponentProps for std::rc::Rc<T> {
    fn props(&self, language: Language, code: &str) -> Option<Props> {
        (**self).props(language, code)
    }
}

/// No reader: no component is analysed.
impl ComponentProps for () {
    fn props(&self, _language: Language, _code: &str) -> Option<Props> {
        None
    }
}

/// What a component takes. In JSON, `{"props": [{"name", "type"}], "rest"}`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Props {
    /// Its props, by name.
    pub props: Vec<Prop>,
    /// Whether it also takes props it doesn't name (a `...rest` with no type), so any param fits.
    #[serde(default)]
    pub rest: bool,
}

/// A prop a component takes.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prop {
    /// Its name, as the binding passes it: camelCase, `children` for the default slot (spec §7.6).
    pub name: String,
    /// Its type, when the code states one this check knows.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<PropType>,
    /// Whether the code lets it be left out: its type marks it optional (`label?: string`), or it has a default.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub optional: bool,
    /// Its default, when the code writes one as a literal: a string, a number or a boolean.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}

/// A prop's type, where the code states one a signature's type can be checked against. In JSON,
/// `{"kind": "string"}`, or `{"kind": "enum", "members": [...]}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PropType {
    /// Text.
    String,
    /// A number.
    Number,
    /// `true` or `false`.
    Boolean,
    /// A node or a snippet: what a binding gives for a slot.
    Slot,
    /// One of these strings.
    Enum {
        /// The strings.
        members: Vec<String>,
    },
}

/// A way a component disagrees with its signature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Mismatch {
    /// What is wrong.
    pub(crate) message: String,
    /// The declared param it is about, where the warning goes; the fence when there is none.
    pub(crate) param: Option<String>,
}

/// The binding's own prop, which no signature declares (spec §7.6).
const CONTEXT_PROP: &str = "compmd";

/// A param's or a slot's prop in the React and Svelte bindings: camelCase (spec §7.6).
fn prop_name(name: &str) -> String {
    let mut prop = String::with_capacity(name.len());
    let mut upper = false;
    for c in name.chars() {
        match c {
            '-' => {
                if upper {
                    prop.push('-');
                }
                upper = true;
            }
            c if upper && (c.is_ascii_lowercase() || c.is_ascii_digit()) => {
                prop.push(c.to_ascii_uppercase());
                upper = false;
            }
            c => {
                if upper {
                    prop.push('-');
                    upper = false;
                }
                prop.push(c);
            }
        }
    }
    if upper {
        prop.push('-');
    }
    prop
}

/// A slot's prop: `children` for the default slot.
fn slot_prop(slot: &str) -> String {
    if slot == "default" {
        "children".to_owned()
    } else {
        prop_name(slot)
    }
}

/// A declared type as a prop's type, when there is one to compare with: `input<…>` and others are passed unchecked.
fn expected(declared: &DeclaredType) -> Option<PropType> {
    match declared {
        DeclaredType::Name(TypeName::Number) => Some(PropType::Number),
        DeclaredType::Name(TypeName::Boolean) => Some(PropType::Boolean),
        DeclaredType::Name(
            TypeName::String
            | TypeName::Length
            | TypeName::Space
            | TypeName::Color
            | TypeName::Tracks
            | TypeName::Areas
            | TypeName::Surface
            | TypeName::Border
            | TypeName::Radius
            | TypeName::Name,
        ) => Some(PropType::String),
        DeclaredType::Enum { members } => Some(PropType::Enum {
            members: members.clone(),
        }),
        DeclaredType::Name(_) | DeclaredType::Other(_) | DeclaredType::Input { .. } => None,
    }
}

/// A type as messages write it.
fn written(kind: &PropType) -> String {
    match kind {
        PropType::String => "a string".to_owned(),
        PropType::Number => "a number".to_owned(),
        PropType::Boolean => "a boolean".to_owned(),
        PropType::Slot => "a slot".to_owned(),
        PropType::Enum { members } => format!("one of {}", members.join(", ")),
    }
}

/// Whether a prop of type `actual` takes every value of the declared `wanted`.
fn fits(wanted: &PropType, actual: &PropType) -> bool {
    match (wanted, actual) {
        (PropType::String | PropType::Enum { .. }, PropType::String) => true,
        (PropType::Enum { members: wanted }, PropType::Enum { members: actual }) => {
            wanted.iter().all(|member| actual.contains(member))
        }
        (_, PropType::Enum { .. } | PropType::String) => false,
        (wanted, actual) => wanted == actual,
    }
}

/// How `props`, what the component in `file` takes, disagrees with its `signature`: a declared param or slot it has
/// no prop for, a prop whose type doesn't take the declared one, and a prop the signature doesn't declare.
pub(crate) fn mismatches(file: &str, signature: &Signature, props: &Props) -> Vec<Mismatch> {
    let mut found = Vec::new();
    let mut declared = vec![CONTEXT_PROP.to_owned()];
    let prop = |name: &str| props.props.iter().find(|prop| prop.name == name);
    for param in &signature.params {
        let name = prop_name(&param.name);
        declared.push(name.clone());
        let Some(prop) = prop(&name) else {
            if !props.rest {
                found.push(Mismatch {
                    param: Some(param.name.clone()),
                    message: format!(
                        "the signature declares `{}`, which {file} has no prop for (`{name}`)",
                        param.name
                    ),
                });
            }
            continue;
        };
        if let (Some(wanted), Some(actual)) = (expected(&param.declared), &prop.kind)
            && !fits(&wanted, actual)
        {
            found.push(Mismatch {
                param: Some(param.name.clone()),
                message: format!(
                    "the signature declares `{}` as {}, which {file}'s prop takes as {}",
                    param.name,
                    written(&wanted),
                    written(actual)
                ),
            });
        }
    }
    for slot in signature.slots.iter().flatten() {
        let name = slot.name();
        let slot_name = slot_prop(name);
        if prop(&slot_name).is_none() && !props.rest {
            found.push(Mismatch {
                param: None,
                message: format!("the signature declares the slot `{name}`, which {file} has no prop for (`{slot_name}`)"),
            });
        }
        declared.push(slot_name);
    }
    for prop in &props.props {
        if !declared.contains(&prop.name) {
            found.push(Mismatch {
                param: None,
                message: format!(
                    "{file} has the prop `{}`, which the signature doesn't declare",
                    prop.name
                ),
            });
        }
    }
    found
}

#[cfg(test)]
mod tests;
