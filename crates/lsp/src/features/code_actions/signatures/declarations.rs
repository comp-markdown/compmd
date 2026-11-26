//! A host-language component's props as a signature's params and slots (spec §7.3, §7.6): each param by its name in
//! kebab-case, as the binding passes it in camelCase; `children` the default slot, and a prop that takes a node or a
//! snippet a slot of its name. What the code states is written, never more: a prop whose type it doesn't state, or
//! states in a way no declaration can say, is left for the writer as a comment.

use compmd::analysis::{Prop, PropType, Props};
use serde_json::Value;

/// The binding's own prop (spec §7.6), which no signature declares.
const CONTEXT_PROP: &str = "compmd";

/// A component's params and slots, as its props give them.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Declared {
    /// Its params, in the order of its props.
    pub(super) params: Vec<Param>,
    /// Its slots' names, `default` for `children`.
    pub(super) slots: Vec<String>,
}

/// A param of a signature, from a prop.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Param {
    /// Its name, in kebab-case.
    pub(super) name: String,
    /// Its declaration's lines, under `params:`.
    pub(super) lines: Vec<String>,
}

impl Declared {
    /// What `props` declare.
    pub(super) fn of(props: &Props) -> Self {
        let mut declared = Self {
            params: Vec::new(),
            slots: Vec::new(),
        };
        for prop in props.props.iter().filter(|prop| prop.name != CONTEXT_PROP) {
            if prop.name == "children" {
                declared.slots.push("default".into());
            } else if prop.kind == Some(PropType::Slot) {
                declared.slots.push(kebab(&prop.name));
            } else {
                declared.params.push(param(prop));
            }
        }
        declared
    }
}

/// `prop` as a param.
fn param(prop: &Prop) -> Param {
    let name = kebab(&prop.name);
    let unknown = |why: &str| vec![format!("  # {name}: {why}")];
    let lines = match prop.kind.as_ref().and_then(type_text) {
        None => unknown("the component's code doesn't say its type"),
        Some(declared) => match &prop.default {
            Some(value) => with_default(&name, &declared, value)
                .unwrap_or_else(|| unknown("its default isn't a value its type takes")),
            None if prop.optional => vec![format!("  {name}: {declared}?")],
            None => vec![format!("  {name}: {declared}")],
        },
    };
    Param { name, lines }
}

/// The type a declaration writes for `kind`; none for an enum whose members aren't names, or that has one member.
fn type_text(kind: &PropType) -> Option<String> {
    match kind {
        PropType::String => Some("string".into()),
        PropType::Number => Some("number".into()),
        PropType::Boolean => Some("boolean".into()),
        PropType::Enum { members }
            if members.len() > 1 && members.iter().all(|member| is_name(member)) =>
        {
            Some(members.join(" | "))
        }
        _ => None,
    }
}

/// The declaration of `name`, of type `declared`, with `value` its default: the short form, or the long one when the
/// default would end the short form's YAML (`: `, ` #`); none when `value` isn't of the type.
fn with_default(name: &str, declared: &str, value: &Value) -> Option<Vec<String>> {
    let default = match value {
        Value::String(text) if declared == "string" => {
            if is_name(text) {
                text.clone()
            } else {
                serde_json::to_string(text).ok()?
            }
        }
        Value::String(text) if declared.split(" | ").any(|member| member == text) => text.clone(),
        Value::Number(number) if declared == "number" => number.to_string(),
        Value::Bool(boolean) if declared == "boolean" => boolean.to_string(),
        _ => return None,
    };
    if default.contains(": ") || default.contains(" #") {
        return Some(vec![
            format!("  {name}:"),
            format!("    type: {declared}"),
            format!("    default: {default}"),
        ]);
    }
    Some(vec![format!("  {name}: {declared} = {default}")])
}

/// Whether `text` is a name, as an enum's members and a bare default are written.
fn is_name(text: &str) -> bool {
    let mut chars = text.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// `name`, a prop's camelCase, in kebab-case: `alignItems` is `align-items`.
pub(super) fn kebab(name: &str) -> String {
    let mut kebab = String::with_capacity(name.len() + 2);
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                kebab.push('-');
            }
            kebab.push(c.to_ascii_lowercase());
        } else {
            kebab.push(c);
        }
    }
    kebab
}

#[cfg(test)]
mod tests;
