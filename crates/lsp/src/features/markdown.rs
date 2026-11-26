//! What a hint is about, written as Markdown for hovers and completions' documentation: a component, a param, a
//! function from the processor's catalog (`compmd::functions`). Writing to a `String` never fails, so `write!`'s result
//! is let go.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::OnceLock;

use compmd::functions::{self, Function};
use compmd::hints::{ComponentHint, ParameterHint};
use compmd::tree::{Signature, SlotDeclaration};

use crate::pack_components::PackComponent;

/// The function written `name` (`md.currency`, `trim`, `map`), from the catalog.
pub(crate) fn function_named(name: &str) -> Option<&'static Function> {
    static CATALOG: OnceLock<HashMap<String, Function>> = OnceLock::new();
    CATALOG
        .get_or_init(|| {
            functions::all()
                .into_iter()
                .map(|function| (function.name.clone(), function))
                .collect()
        })
        .get(name)
}

/// A function's signatures, its description and its first example with its result.
pub(crate) fn function(function: &Function) -> String {
    let signatures: Vec<&str> = function
        .signatures
        .iter()
        .map(|signature| signature.written.as_str())
        .collect();
    let mut text = format!(
        "```cel\n{}\n```\n\n{}",
        signatures.join("\n"),
        function.description
    );
    if let Some(example) = function.examples.first() {
        let _ = write!(
            text,
            "\n\n`{}` gives `{}`",
            example.expression, example.result
        );
    }
    let _ = write!(text, "\n\n(spec §{})", function.section);
    text
}

/// A param: what it takes, whether it must be given, its default and its doc.
pub(crate) fn param(param: &ParameterHint) -> String {
    let mut text = format!("`{}`: {}", param.name, param.takes.description);
    if param.required {
        text.push_str(", required");
    }
    if let Some(default) = &param.default {
        let _ = write!(text, ", `{default}` by default");
    }
    if param.breakpoints {
        text.push_str(", with breakpoints");
    }
    if let Some(doc) = &param.doc {
        let _ = write!(text, ". {doc}");
    }
    text
}

/// A component a name stands for: what implements it, where it comes from, and its signature.
pub(crate) fn component(name: &str, component: &ComponentHint) -> String {
    let what = match component.kind.as_str() {
        "compmd" => "a CompMD component".to_owned(),
        "processor" => "built into the processor".to_owned(),
        language => format!("a {language} component"),
    };
    let from = match &component.file {
        Some(file) => format!(" from `{file}`"),
        None => " defined in this document".to_owned(),
    };
    let named = component
        .defined_as
        .as_ref()
        .map(|defined| format!(" (`{defined}` there)"))
        .unwrap_or_default();
    let mut text = format!("**`{name}`**{named}, {what}{from}");
    match &component.signature {
        Some(signature) => text.push_str(&signature_text(signature)),
        None => text.push_str("\n\nNo signature: its params go unchecked."),
    }
    text
}

/// A component a pack exports that the document doesn't import yet: its pack, the pack's description and its
/// signature.
pub(crate) fn pack_component(component: &PackComponent) -> String {
    let mut text = format!("**`{}`**, from `{}`", component.name, component.from);
    if let Some(description) = &component.description {
        let _ = write!(text, "\n\n{description}");
    }
    if let Some(signature) = &component.signature {
        text.push_str(&signature_text(signature));
    }
    text
}

/// A signature's params and slots.
fn signature_text(signature: &Signature) -> String {
    let mut text = String::new();
    if !signature.params.is_empty() {
        text.push_str("\n\nParams:");
        for declared in &signature.params {
            let optional = if declared.optional || declared.default.is_some() {
                ""
            } else {
                ", required"
            };
            let _ = write!(
                text,
                "\n- `{}`: {}{optional}",
                declared.name,
                declared.declared.written()
            );
            if let Some(doc) = &declared.doc {
                let _ = write!(text, ". {doc}");
            }
        }
    }
    if let Some(slots) = &signature.slots {
        let names: Vec<String> = slots
            .iter()
            .map(|slot| format!("`{}`", slot_name(slot)))
            .collect();
        let _ = write!(text, "\n\nSlots: {}", names.join(", "));
    }
    text
}

/// A slot's name.
pub(crate) fn slot_name(slot: &SlotDeclaration) -> &str {
    match slot {
        SlotDeclaration::Name(name) | SlotDeclaration::Template { name, .. } => name,
    }
}
