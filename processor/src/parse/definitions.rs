//! Fenced code with `component=` in its info string: component definitions and signature fences (spec §7, §7.3).

use super::Converter;
use super::params::{self, is_name};
use super::source::right;
use super::yaml;
use crate::diagnostic::Code;
use crate::tree::{
    Declaration, DeclaredType, DefaultValue, Definition, DefinitionBody, Headings, Node, Param,
    Point, Position, Signature, SignatureFence, SlotDeclaration,
};
use markdown::mdast;
use serde_json::Value;

impl Converter<'_, '_> {
    /// The definition or signature fence `code` is, if its info string names a component.
    pub(super) fn definition(
        &mut self,
        code: &mdast::Code,
        position: Option<Position>,
    ) -> Option<Node> {
        let meta = code.meta.as_deref().unwrap_or("");
        let attributes: Vec<&str> = meta[..meta.find('{').unwrap_or(meta.len())]
            .split_whitespace()
            .collect();
        let name = attributes
            .iter()
            .find_map(|a| a.strip_prefix("component="))?;
        if !is_name(name) {
            return None;
        }
        let start = position.map_or(super::START, |position| position.start);
        let fence_params = self.fence_params(start);
        let language = code.lang.clone().unwrap_or_default();
        if language != "compmd" {
            let runtime = attributes.iter().find_map(|a| a.strip_prefix("runtime="));
            return Some(Node::ComponentDefinition(Definition {
                name: name.to_string(),
                language,
                signature: None,
                params: fence_params,
                body: DefinitionBody::Host {
                    source: code.value.clone(),
                    runtime: runtime.map(str::to_string),
                },
                position,
            }));
        }
        if attributes.contains(&"signature") {
            if let Some(first) = fence_params.first().and_then(|param| param.position) {
                self.diagnose(
                    Code::F036,
                    first.start,
                    "a signature fence takes no param list; ignored (style the component's box on its definition)",
                );
            }
            return Some(Node::ComponentSignature(SignatureFence {
                name: name.to_string(),
                signature: signature(&code.value),
                position,
            }));
        }
        let lines: Vec<&str> = code.value.split('\n').collect();
        let separator = lines.iter().position(|line| *line == "---");
        let (header, children) = match separator {
            Some(index) => {
                let body = lines[index + 1..].join("\n");
                let body_line = start.line + 2 + index;
                let origin = self
                    .source
                    .line(body_line)
                    .map_or(start, |(_, origin)| origin);
                (
                    lines[..index].join("\n"),
                    self.placed(&body, origin).document(false).children,
                )
            }
            None => (code.value.clone(), Vec::new()),
        };
        Some(Node::ComponentDefinition(Definition {
            name: name.to_string(),
            language,
            signature: signature(&header),
            params: fence_params,
            body: DefinitionBody::Compmd { children },
            position,
        }))
    }

    /// The param list that ends the info string of the fence at `start`, which styles the component's box
    /// (spec §7).
    fn fence_params(&mut self, start: Point) -> Vec<Param> {
        let (line, line_start) = self.source.line(start.line).unwrap_or(("", start));
        let chars: Vec<char> = line.chars().collect();
        let component = line
            .find("component=")
            .map_or(0, |byte| line[..byte].chars().count());
        let Some(brace) = chars[component..]
            .iter()
            .position(|c| *c == '{')
            .map(|i| component + i)
        else {
            return Vec::new();
        };
        let mut list = params::braced(&chars, brace);
        if list.broke_at.is_none() {
            let trailing = chars[list.end..].iter().position(|c| !c.is_whitespace());
            list.broke_at = trailing.map(|i| list.end + i);
        }
        let mut fence_params = Vec::new();
        self.add_params_at(&mut fence_params, list, |index| right(line_start, index));
        fence_params
    }
}

/// A signature: YAML whose `params` are declarations in short or long form (spec §7.3); `None` when it isn't YAML.
fn signature(yaml_text: &str) -> Option<Signature> {
    let value = yaml::parse(yaml_text).ok()?.value;
    let params = value
        .get("params")
        .and_then(Value::as_object)
        .map(|params| {
            params
                .iter()
                .map(|(name, declared)| declaration(name, declared))
                .collect()
        })
        .unwrap_or_default();
    let headings = if value.get("headings").and_then(Value::as_str) == Some("absolute") {
        Headings::Absolute
    } else {
        Headings::Relative
    };
    Some(Signature {
        params,
        slots: slots(value.get("slots")),
        headings,
    })
}

/// `slots`: a list of names, or a mapping from names to options, where `{each, as}` makes a template slot.
fn slots(value: Option<&Value>) -> Option<Vec<SlotDeclaration>> {
    let name = |value: &Value| {
        value
            .as_str()
            .map_or_else(|| value.to_string(), str::to_string)
    };
    match value? {
        Value::Array(names) => Some(
            names
                .iter()
                .map(|n| SlotDeclaration::Name(name(n)))
                .collect(),
        ),
        Value::Object(entries) => Some(
            entries
                .iter()
                .map(|(slot, options)| {
                    match (
                        options.get("each").and_then(Value::as_str),
                        options.get("as").and_then(Value::as_str),
                    ) {
                        (Some(each), Some(binding)) => SlotDeclaration::Template {
                            name: slot.clone(),
                            each: each.to_string(),
                            binding: binding.to_string(),
                        },
                        _ => SlotDeclaration::Name(slot.clone()),
                    }
                })
                .collect(),
        ),
        _ => None,
    }
}

/// A declaration, short form (`info | warn = info`) or long form (`{type, default, doc}`).
pub(crate) fn declaration(name: &str, written: &Value) -> Declaration {
    if let Some(short) = written.as_str() {
        let (type_text, default) = match split_default(short) {
            Some((type_text, default)) => (type_text, short_default(default.trim())),
            None => (short, None),
        };
        let (declared, optional) = type_of(type_text);
        return Declaration {
            name: name.to_string(),
            declared,
            optional,
            default,
            doc: None,
        };
    }
    let (declared, optional) = type_of(written.get("type").and_then(Value::as_str).unwrap_or(""));
    Declaration {
        name: name.to_string(),
        declared,
        optional,
        default: written.get("default").cloned().map(DefaultValue::Yaml),
        doc: written
            .get("doc")
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

/// A short form split at its `=` with white space on both sides: the type, and the default.
fn split_default(short: &str) -> Option<(&str, &str)> {
    let bytes = short.as_bytes();
    let space = |at: usize| matches!(bytes.get(at), Some(b' ' | b'\t'));
    let equals =
        (1..bytes.len()).find(|at| bytes[*at] == b'=' && space(at - 1) && space(at + 1))?;
    Some((&short[..equals], &short[equals + 1..]))
}

/// A short form's default, read as a param value.
fn short_default(text: &str) -> Option<DefaultValue> {
    let chars: Vec<char> = format!("d={text}").chars().collect();
    let list = params::unbraced(&chars, 0);
    match (list.broke_at, list.params.into_iter().next()) {
        (None, Some(param)) => Some(DefaultValue::Param(param.value)),
        _ => None,
    }
}

/// A type and whether it ends in `?`.
fn type_of(text: &str) -> (DeclaredType, bool) {
    let text = text.trim();
    match text.strip_suffix('?') {
        Some(required) => (declared_type(required.trim()), true),
        None => (declared_type(text), false),
    }
}

/// A type name, an enum (`a | b`), or `input<type>`.
fn declared_type(text: &str) -> DeclaredType {
    if let Some(inner) = text
        .strip_prefix("input<")
        .and_then(|t| t.strip_suffix('>'))
    {
        return DeclaredType::Input {
            input: Box::new(declared_type(inner.trim())),
        };
    }
    if text.contains('|') {
        return DeclaredType::Enum {
            members: text
                .split('|')
                .map(|member| member.trim().to_string())
                .collect(),
        };
    }
    DeclaredType::Name(text.to_string())
}
