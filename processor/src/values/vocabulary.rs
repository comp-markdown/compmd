//! The fixed vocabulary every document can write (spec §5.1, §9): the common params, the attributes and `box`'s own
//! params, each with what it takes. `conformance/params.json` is its JSON, which a renderer checks itself against
//! and an editor can list values from.

use super::{Form, Kind};
use crate::parse::ParseOptions;
use crate::processor::Processor;
use crate::resolve::CORE_TEXT;
use crate::tree::Node;
use crate::validate::universal::Universal;
use serde::Serialize;

/// A param of the fixed vocabulary, and what it takes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Known {
    /// The param's key.
    pub name: String,
    /// What it takes: [`Kind`]'s fields.
    pub tokens: Vec<String>,
    /// The forms it takes besides its tokens.
    pub forms: Vec<Form>,
    /// Values of its forms.
    pub examples: Vec<&'static str>,
    /// How many space-separated values it takes.
    pub up_to: usize,
    /// Whether a flag is one of its values.
    pub flag: bool,
    /// Whether a quoted string may hold it.
    pub quoted: bool,
    /// Whether it may carry a breakpoint (spec §9.3).
    pub breakpoints: bool,
}

impl Known {
    fn new(name: &str, kind: &Kind, breakpoints: bool) -> Self {
        Self {
            name: name.to_string(),
            tokens: kind.tokens.iter().map(ToString::to_string).collect(),
            forms: kind.forms.to_vec(),
            examples: kind.examples.to_vec(),
            up_to: kind.up_to,
            flag: kind.flag,
            quoted: kind.quoted,
            breakpoints,
        }
    }
}

/// The fixed vocabulary, in the order the spec lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Vocabulary {
    /// The common params, which style a component's box (spec §9.2).
    pub common: Vec<Known>,
    /// `class`, `id`, `lang` and `dir` (spec §5.1).
    pub attributes: Vec<Known>,
    /// `box`'s own params, from its signature in `@compmd/core` (spec §9.1).
    pub layout: Vec<Known>,
}

/// The fixed vocabulary.
///
/// ```
/// let vocabulary = compmd::values::vocabulary();
/// let border = vocabulary.common.iter().find(|param| param.name == "border").unwrap();
/// assert_eq!(border.tokens, ["none", "subtle", "strong"]);
/// assert!(vocabulary.layout.iter().any(|param| param.name == "gap"));
/// ```
#[must_use]
pub fn vocabulary() -> Vocabulary {
    let (common, attributes) = Universal::ALL
        .iter()
        .map(|param| {
            (
                param.is_common(),
                Known::new(param.name(), &param.kind(), param.takes_breakpoint()),
            )
        })
        .partition::<Vec<_>, _>(|(common, _)| *common);
    let core = Processor::new("core.comp.md", CORE_TEXT).parse(&ParseOptions::default());
    let layout = core
        .tree()
        .children
        .iter()
        .find_map(|node| match node {
            Node::ComponentSignature(fence) if fence.name == "box" => fence.signature.as_ref(),
            _ => None,
        })
        .map(|signature| {
            signature
                .params
                .iter()
                .map(|declared| {
                    Known::new(&declared.name, &Kind::declared(&declared.declared), true)
                })
                .collect()
        })
        .unwrap_or_default();
    Vocabulary {
        common: common.into_iter().map(|(_, known)| known).collect(),
        attributes: attributes.into_iter().map(|(_, known)| known).collect(),
        layout,
    }
}
