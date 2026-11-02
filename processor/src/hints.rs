//! Positional hints: what is at a place in a processed document's text, and what it takes. An editor shows a hint
//! beside the text and offers its completions; a language server makes its hovers and completions of the same hint.
//!
//! A hint is a lookup in what the processor keeps, never a stage run again: it finds the block a place is in, reads
//! the place's line again, and looks its names up. Reading the line again gives half-typed text a hint too, such as
//! `border=` or a list whose `}` isn't written yet.

mod expression;
mod frontmatter;
mod invocation;

pub use expression::{FunctionHint, NameHint};

use crate::components;
use crate::diagnostic::{Diagnostic, Location, Range};
use crate::parse::CODE;
use crate::resolve::{Component, Document, Resolved};
use crate::settings::Input;
use crate::tree::{
    Code, Declaration, DeclaredType, DefaultValue, Node, ParamValue, Signature, SlotDeclaration,
    StringPart, TypeName,
};
use crate::validate::params::binds;
use crate::validate::universal::Universal;
use crate::values::{Form, Kind};
use serde::Serialize;

/// What is at a place in a processed document's text.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PositionalHint {
    /// The text the hint is about: a name, a key, a value. A completion replaces it; it is empty where there is
    /// nothing written yet.
    pub range: Range,
    /// What the text is, and what it takes; `None` where only a diagnostic has something to say.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Subject>,
    /// What may be written at `range`, each replacing it.
    pub completions: Vec<Completion>,
    /// The diagnostics of the processed document whose range holds the place, as the processor gives them.
    pub diagnostics: Vec<Diagnostic>,
}

/// What the text at a place is.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Subject {
    /// A component's name, on its line or inline.
    Component {
        /// The name as written.
        name: String,
        /// The component it stands for; `None` when it stands for none.
        #[serde(skip_serializing_if = "Option::is_none")]
        component: Option<ComponentHint>,
    },
    /// A param list between params, or a key that isn't one of the component's: the params it takes, without those
    /// already written.
    Params {
        /// The component's name.
        component: String,
        /// The params.
        params: Vec<ParameterHint>,
    },
    /// A param's key, or its value.
    Param {
        /// The component's name.
        component: String,
        /// The param.
        param: ParameterHint,
        /// The value as written, when the place is in it; empty after an `=` with nothing yet.
        #[serde(skip_serializing_if = "Option::is_none")]
        value: Option<String>,
    },
    /// A slot line.
    Slot {
        /// The container's component.
        component: String,
        /// The slot's name as written.
        slot: String,
        /// The slots the component declares.
        slots: Vec<SlotDeclaration>,
    },
    /// A step of a path, in an interpolation or an expression.
    Path {
        /// The path up to the step, `md.data.sales`.
        path: String,
        /// What the step names.
        name: NameHint,
    },
    /// A function called in an expression.
    Function {
        /// Its name, `md.sortBy` for an `md.` function.
        name: String,
        /// What it is; `None` for a name that is no function.
        #[serde(skip_serializing_if = "Option::is_none")]
        hint: Option<FunctionHint>,
    },
    /// A key in the frontmatter, or a value CompMD reads.
    Frontmatter {
        /// The keys to it, outermost first: `["md", "data", "schema"]`.
        path: Vec<String>,
        /// What it holds; `None` for a key CompMD doesn't know.
        #[serde(skip_serializing_if = "Option::is_none")]
        about: Option<String>,
    },
    /// A type name in a declaration: an input's, a schema field's.
    Type {
        /// The name as written.
        name: String,
        /// What a value of the type is; `None` for a name that is no type.
        #[serde(skip_serializing_if = "Option::is_none")]
        takes: Option<KindHint>,
    },
}

/// A component a name stands for.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentHint {
    /// Its name in the file that defines it, when an import renames it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_as: Option<String>,
    /// What implements it: `compmd`, `processor`, or the host language as written (`jsx`, `svelte`, …).
    pub kind: String,
    /// Its signature; `None` for a host-language component without one, whose params go unchecked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
    /// The file that defines it, as diagnostics name it; `None` for the processed document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Whether the processor provides it: `@compmd/core` or an ambient pack.
    pub provided: bool,
    /// Where in that file it is defined.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined: Option<Range>,
}

/// Where a param comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamOrigin {
    /// The component's signature declares it.
    Declared,
    /// A common param, which styles the component's box (spec §9.2).
    Common,
    /// `class`, `id`, `lang` or `dir` (spec §5.1).
    Attribute,
}

/// A param: what it is and what it takes.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ParameterHint {
    /// Its key.
    pub name: String,
    /// Where it comes from.
    pub origin: ParamOrigin,
    /// What it takes.
    pub takes: KindHint,
    /// Whether an invocation must give it (F010 otherwise).
    pub required: bool,
    /// Its declared default, as written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    /// The declaration's `doc`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    /// Whether it may carry a breakpoint, `md:gap=l` (spec §9.3).
    pub breakpoints: bool,
}

/// What a value may be: [`Kind`]'s fields, and how a message says it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KindHint {
    /// The words it takes; for a bound input, the inputs it can bind.
    pub tokens: Vec<String>,
    /// The forms it takes besides its tokens.
    pub forms: Vec<Form>,
    /// Values of its forms.
    pub examples: Vec<String>,
    /// How many space-separated values it takes.
    pub up_to: usize,
    /// Whether a flag, written without a value, is one.
    pub flag: bool,
    /// Whether a quoted string may hold it.
    pub quoted: bool,
    /// What it takes, as F009 says it.
    pub description: String,
}

impl KindHint {
    fn of(kind: &Kind) -> Self {
        Self {
            tokens: kind.tokens.iter().map(ToString::to_string).collect(),
            forms: kind.forms.to_vec(),
            examples: kind.examples.iter().map(ToString::to_string).collect(),
            up_to: kind.up_to,
            flag: kind.flag,
            quoted: kind.quoted,
            description: kind.describe(),
        }
    }

    /// A param of type `declared`, which may bind one of `inputs`.
    fn declared(declared: &DeclaredType, inputs: &[Input]) -> Self {
        let DeclaredType::Input { input } = declared else {
            return Self::of(&Kind::declared(declared));
        };
        let bound: Vec<&str> = inputs
            .iter()
            .filter(|known| binds(input, &known.declaration.declared))
            .map(|known| known.declaration.name.as_str())
            .collect();
        Self {
            forms: vec![Form::Name],
            quoted: false,
            description: format!("the name of {} under `md.inputs`", declared.written()),
            ..Self::of(&Kind::one_of(bound))
        }
    }
}

/// What may be written at a hint's range.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Completion {
    /// What a list shows.
    pub label: String,
    /// What replaces the range.
    pub text: String,
    /// What it is, in a few words.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl Completion {
    fn new(text: &str, detail: Option<String>) -> Self {
        Self {
            label: text.to_string(),
            text: text.to_string(),
            detail,
        }
    }
}

/// The hint at code point `offset` of the processed document of `resolved`, whose `diagnostics` are those the
/// processor gives.
pub(crate) fn at(
    resolved: &Resolved,
    diagnostics: &[Diagnostic],
    offset: usize,
) -> Option<PositionalHint> {
    let document = resolved.document(resolved.main);
    let caret = document.line_starts.location(offset);
    let lines = document.lines();
    let text: Vec<char> = lines.line(caret.line).unwrap_or("").chars().collect();
    let place = Place {
        resolved,
        document,
        offset,
        line: caret.line,
        at: (caret.column - 1).min(text.len()),
        text,
    };
    let found = match surrounding(&document.root.children, offset) {
        Surrounding::Text => expression::interpolation(&place).or_else(|| invocation::hint(&place)),
        Surrounding::Frontmatter => frontmatter::hint(&place),
        Surrounding::Code(code) => invocation::code_block(&place, code),
        Surrounding::Opaque => None,
    };
    let caret = Location::new(place.line, place.at + 1);
    let diagnostics: Vec<Diagnostic> = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.file.is_none() && covers(&diagnostic.range, caret))
        .cloned()
        .collect();
    let hint = match found {
        Some(found) => PositionalHint {
            range: place.range(found.start, found.end),
            subject: Some(found.subject),
            completions: found.completions,
            diagnostics,
        },
        None if diagnostics.is_empty() => return None,
        None => PositionalHint {
            range: place.range(place.at, place.at),
            subject: None,
            completions: Vec::new(),
            diagnostics,
        },
    };
    Some(hint)
}

/// Whether `range` holds `at`. A range without an end runs to the end of its line.
fn covers(range: &Range, at: Location) -> bool {
    let end = range
        .end
        .unwrap_or(Location::new(range.start.line, usize::MAX));
    range.start <= at && at <= end
}

/// The caret's line in the processed document.
struct Place<'r> {
    resolved: &'r Resolved,
    document: &'r Document,
    /// The caret's code point in the document.
    offset: usize,
    /// The line's number.
    line: usize,
    /// The line's code points.
    text: Vec<char>,
    /// The caret's index in `text`.
    at: usize,
}

impl Place<'_> {
    /// The range of the line's code points `start..end`.
    fn range(&self, start: usize, end: usize) -> Range {
        Range {
            start: Location::new(self.line, start + 1),
            end: Some(Location::new(self.line, end + 1)),
        }
    }

    /// The component `name` stands for in the document.
    fn component(&self, name: &str) -> Option<&Component> {
        match self.document.names.get(name)? {
            crate::resolve::Binding::Component(id, _) => Some(self.resolved.component(*id)),
            _ => None,
        }
    }

    /// What `name` stands for, for its hint.
    fn component_hint(&self, name: &str) -> Option<ComponentHint> {
        let component = self.component(name)?;
        let origin = components::origin(self.resolved, component);
        Some(ComponentHint {
            defined_as: (component.name != name).then(|| component.name.clone()),
            kind: components::kind(component),
            signature: component.signature.clone(),
            file: (origin != self.document.origin).then(|| origin.to_string()),
            provided: self.resolved.document(component.document).provided,
            defined: component.position.map(Range::from),
        })
    }

    /// Every name the document can give a component, each with what implements it.
    fn component_names(&self) -> Vec<Completion> {
        self.document
            .names
            .components()
            .map(|(name, id)| {
                Completion::new(name, Some(components::kind(self.resolved.component(id))))
            })
            .collect()
    }

    /// The params the component `name` takes: those its signature declares, then the common params and the
    /// attributes. A component without a signature, or a name that stands for none, takes those every component
    /// takes.
    fn params(&self, name: &str) -> Vec<ParameterHint> {
        let component = self.component(name);
        let inputs = &self.document.settings.inputs;
        let declared = component
            .and_then(|component| Some((component, component.signature.as_ref()?)))
            .into_iter()
            .flat_map(|(component, signature)| {
                signature
                    .params
                    .iter()
                    .map(move |declaration| declared(declaration, component, inputs))
            });
        let mut params: Vec<ParameterHint> = declared.collect();
        let universal: Vec<ParameterHint> = Universal::ALL
            .iter()
            .filter(|universal| params.iter().all(|param| param.name != universal.name()))
            .map(|universal| ParameterHint {
                name: universal.name().to_string(),
                origin: if universal.is_common() {
                    ParamOrigin::Common
                } else {
                    ParamOrigin::Attribute
                },
                takes: KindHint::of(&universal.kind()),
                required: false,
                default: None,
                doc: None,
                breakpoints: universal.takes_breakpoint(),
            })
            .collect();
        params.extend(universal);
        params
    }

    /// The params a code block the code renderer `name` renders takes on its info string (spec §7.7): the
    /// renderer's, but `code`, which is the block's text, and `alt`.
    fn block_params(&self, name: &str) -> Vec<ParameterHint> {
        let mut params: Vec<ParameterHint> = self
            .params(name)
            .into_iter()
            .filter(|param| param.name != CODE)
            .collect();
        params.push(ParameterHint {
            name: "alt".to_string(),
            origin: ParamOrigin::Attribute,
            takes: KindHint::of(&Kind::declared(&DeclaredType::Name(TypeName::String))),
            required: false,
            default: None,
            doc: Some("The text alternative for screen readers.".to_string()),
            breakpoints: false,
        });
        params
    }
}

/// The hint of a declared param of `component`.
fn declared(declaration: &Declaration, component: &Component, inputs: &[Input]) -> ParameterHint {
    ParameterHint {
        name: declaration.name.clone(),
        origin: ParamOrigin::Declared,
        takes: KindHint::declared(&declaration.declared, inputs),
        required: !declaration.optional && declaration.default.is_none(),
        default: declaration.default.as_ref().map(written),
        doc: declaration.doc.clone(),
        breakpoints: component.takes_breakpoints(),
    }
}

/// A default as a param would write it.
fn written(default: &DefaultValue) -> String {
    match default {
        DefaultValue::Param(value) => match value {
            ParamValue::Bare(text)
            | ParamValue::Json(text)
            | ParamValue::Class(text)
            | ParamValue::Id(text) => text.clone(),
            ParamValue::Expr(source) => format!("({source})"),
            ParamValue::Flag => "true".to_string(),
            ParamValue::String(parts) => {
                let text: String = parts
                    .iter()
                    .map(|part| match part {
                        StringPart::Text(text) => text.value.clone(),
                        StringPart::Interpolation(interpolation) => {
                            format!("{{{{{}}}}}", interpolation.path)
                        }
                    })
                    .collect();
                format!("\"{text}\"")
            }
        },
        DefaultValue::Yaml(serde_json::Value::String(text)) => text.clone(),
        DefaultValue::Yaml(value) => value.to_string(),
    }
}

/// What a hint found on its line: the code points `start..end` it is about, what they are, and what may replace
/// them.
struct Found {
    start: usize,
    end: usize,
    subject: Subject,
    completions: Vec<Completion>,
}

/// Whether a place is in text whose component syntax counts, in the frontmatter, or in a block that holds other
/// text: code, math, raw HTML, a signature, a definition's header or host-language code.
#[derive(Debug, PartialEq)]
enum Surrounding<'a> {
    Text,
    Frontmatter,
    /// A code block, which a code renderer may render (spec §7.7).
    Code(&'a Code),
    Opaque,
}

/// What surrounds code point `offset` of the blocks `nodes`. The blocks are in order, so the one a place is in is the
/// last that starts before it.
fn surrounding(nodes: &[Node], offset: usize) -> Surrounding<'_> {
    let starts_before = nodes.partition_point(|node| {
        node.position()
            .is_none_or(|position| position.start.offset <= offset)
    });
    let Some(block) = starts_before.checked_sub(1).map(|index| &nodes[index]) else {
        return Surrounding::Text;
    };
    let inside = block
        .position()
        .is_some_and(|position| position.start.offset <= offset && offset <= position.end.offset);
    match block {
        Node::Yaml(_) if inside => Surrounding::Frontmatter,
        Node::Code(code) if inside => Surrounding::Code(code),
        Node::Math(_)
        | Node::Html(_)
        | Node::InlineCode(_)
        | Node::InlineMath(_)
        | Node::ComponentSignature(_)
            if inside =>
        {
            Surrounding::Opaque
        }
        Node::ComponentDefinition(definition) if inside => match &definition.body {
            crate::tree::DefinitionBody::Compmd { children }
                if children
                    .first()
                    .and_then(Node::position)
                    .is_some_and(|first| first.start.offset <= offset) =>
            {
                surrounding(children, offset)
            }
            _ => Surrounding::Opaque,
        },
        Node::ComponentContainer(container) => {
            let in_slot = container.slots.iter().rev().find(|slot| {
                slot.position
                    .is_some_and(|position| position.start.offset <= offset)
            });
            match in_slot {
                Some(slot) => surrounding(&slot.children, offset),
                None => surrounding(&container.children, offset),
            }
        }
        _ => surrounding(block.children(), offset),
    }
}

#[cfg(test)]
mod tests;
