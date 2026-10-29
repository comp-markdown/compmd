//! The processed document's inputs (spec §8.7): each one's value for the render, the one given when it fits the
//! input's type, otherwise its default. A default's expression sees `md.doc`, `md.version`, `md.lang`, `md.tz` and
//! the `md.` functions, not data or other inputs.

use super::schema::read;
use crate::cel::{Program, Value, Values};
use crate::context::RenderContext;
use crate::diagnostic::{Code, Diagnostic, Stage};
use crate::expanded::{Input, Inputs};
use crate::resolve::Resolved;
use crate::settings::Settings;
use crate::tree::{DeclaredType, DefaultValue, ParamValue, StringPart};
use serde_json::Value as Json;

/// The inputs of the processed document, and the errors of their defaults (F024).
pub(super) fn of(resolved: &Resolved, context: &RenderContext) -> (Inputs, Vec<Diagnostic>) {
    let settings = &resolved.document(resolved.main).settings;
    let mut diagnostics = Vec::new();
    let inputs = settings
        .inputs
        .iter()
        .map(|input| {
            let declaration = &input.declaration;
            let declared = &declaration.declared;
            let given = context
                .inputs
                .get(&declaration.name)
                .filter(|value| read((*value).clone(), declared, false, false).is_ok())
                .cloned();
            let value = given.unwrap_or_else(|| {
                default(declaration.default.as_ref(), declared, settings, context).unwrap_or_else(
                    |message| {
                        diagnostics.push(
                            Diagnostic::new(Code::F024, input.at, message).in_stage(Stage::Load),
                        );
                        Json::Null
                    },
                )
            });
            let (kind, members) = match declared {
                DeclaredType::Enum { members } => ("enum".to_string(), Some(members.clone())),
                DeclaredType::Name(name) => (name.name().to_string(), None),
                DeclaredType::Other(name) => (name.clone(), None),
                DeclaredType::Input { .. } => ("dyn".to_string(), None),
            };
            Input {
                name: declaration.name.clone(),
                kind,
                members,
                value,
            }
        })
        .collect();
    (Inputs(inputs), diagnostics)
}

/// An input's default, as the expanded tree writes it.
fn default(
    default: Option<&DefaultValue>,
    declared: &DeclaredType,
    settings: &Settings,
    context: &RenderContext,
) -> Result<Json, String> {
    let value = match default {
        None => Value::Null,
        Some(DefaultValue::Yaml(yaml)) => Value::from_json(yaml),
        Some(DefaultValue::Param(value)) => match value {
            ParamValue::Bare(text) => Value::bare(text, Some(declared)),
            ParamValue::String(parts) => Value::string(&text_of(parts, settings, context)?),
            ParamValue::Json(text) => {
                Value::from_json(&serde_json::from_str(text).map_err(|error| error.to_string())?)
            }
            ParamValue::Expr(source) => evaluate(source, settings, context)?,
            ParamValue::Flag => Value::Bool(true),
            ParamValue::Class(text) | ParamValue::Id(text) => Value::string(text),
        },
    };
    Ok(value.to_json())
}

/// A quoted default's text, its interpolations filled in.
fn text_of(
    parts: &[StringPart],
    settings: &Settings,
    context: &RenderContext,
) -> Result<String, String> {
    let mut text = String::new();
    for part in parts {
        match part {
            StringPart::Text(literal) => text.push_str(&literal.value),
            StringPart::Interpolation(interpolation) => {
                let value = evaluate(&interpolation.path, settings, context)?;
                text.push_str(&value.written()?.unwrap_or_default());
            }
        }
    }
    Ok(text)
}

/// An expression of a default, with `md` of the document and its functions.
fn evaluate(source: &str, settings: &Settings, context: &RenderContext) -> Result<Value, String> {
    let values = Values::default().with("md", context.md(settings).into());
    Program::parse(source)
        .and_then(|program| program.evaluate(&values, &context.functions(settings)))
        .map_err(|error| error.to_string())
}
