//! Fenced code with `component=` in its info string: component definitions and signature fences (spec §7, §7.3).

use super::params::{self, is_name};
use super::source::right;
use super::yaml;
use super::{Converter, Labels};
use crate::diagnostic::{Code, Location};
use crate::tree::{
    Declaration, DeclaredType, DefaultValue, Definition, DefinitionBody, Headings, Node, Param,
    Point, Position, Signature, SignatureFence, SlotDeclaration, TypeName,
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
                signature: self.signature(&code.value, start.line + 1),
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
                    self.placed(&body, origin, &Labels::NONE)
                        .document(false)
                        .children,
                )
            }
            None => (code.value.clone(), Vec::new()),
        };
        Some(Node::ComponentDefinition(Definition {
            name: name.to_string(),
            language,
            signature: self.signature(&header, start.line + 1),
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

    /// A signature: YAML whose `params` are declarations in short or long form (spec §7.3), written from line
    /// `first` of the document; `None` when it isn't YAML.
    fn signature(&self, yaml_text: &str, first: usize) -> Option<Signature> {
        let document = yaml::parse(yaml_text).ok()?;
        let yaml_lines: Vec<&str> = yaml_text.split('\n').collect();
        // Where the text of the value at a pointer starts in the document: past its quote, and past what the fence's
        // container (a list item, a quote) puts before each of its lines.
        let text_at = |pointer: &str| -> Option<Location> {
            let mark = document.values.get(pointer)?;
            let yaml_line = yaml_lines.get(mark.line.checked_sub(1)?)?;
            let (line, start) = self.source.line(first + mark.line - 1)?;
            let before = line
                .chars()
                .count()
                .saturating_sub(yaml_line.chars().count());
            let quoted = yaml_line
                .chars()
                .nth(mark.column)
                .is_some_and(|c| c == '"' || c == '\'');
            Some(right(start, before + mark.column + usize::from(quoted)).into())
        };
        Some(signature(&document.value, &text_at))
    }
}

/// The JSON pointer where a declaration at `pointer` writes its type: the short form itself, or the long form's
/// `type`.
pub(crate) fn type_pointer(pointer: &str, written: &Value) -> String {
    if written.is_string() {
        pointer.to_string()
    } else {
        format!("{pointer}/type")
    }
}

/// A signature read from its YAML `value`; `text_at` gives where the text of the value at a JSON pointer starts.
fn signature(value: &Value, text_at: &dyn Fn(&str) -> Option<Location>) -> Signature {
    let mut params: Vec<Declaration> = value
        .get("params")
        .and_then(Value::as_object)
        .map(|params| {
            params
                .iter()
                .map(|(name, declared)| {
                    let at = text_at(&type_pointer(&format!("/params/{name}"), declared));
                    declaration(name, declared, at)
                })
                .collect()
        })
        .unwrap_or_default();
    let headings = if value.get("headings").and_then(Value::as_str) == Some("absolute") {
        Headings::Absolute
    } else {
        Headings::Relative
    };
    let renders = renders(value.get("renders"));
    if renders.is_some() && !params.iter().any(|param| param.name == CODE) {
        params.insert(0, code_declaration());
    }
    Signature {
        params,
        slots: slots(value.get("slots")),
        headings,
        renders,
    }
}

/// The param a code block's text fills when a code renderer renders it (spec §7.7).
pub(crate) const CODE: &str = "code";

/// `code` as a code renderer has it without declaring it: a required string.
fn code_declaration() -> Declaration {
    Declaration {
        name: CODE.to_string(),
        declared: DeclaredType::Name(TypeName::String),
        optional: false,
        default: None,
        doc: None,
        unknown_at: None,
        type_at: None,
    }
}

/// `renders`: a language, or a list of names of one (spec §7.3).
fn renders(value: Option<&Value>) -> Option<Vec<String>> {
    let languages: Vec<String> = match value? {
        Value::String(language) => vec![language.clone()],
        Value::Array(languages) => languages
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    };
    (!languages.is_empty()).then_some(languages)
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

/// A declaration, short form (`info | warn = info`) or long form (`{type, default, doc}`); `type_at` is where its
/// type's text starts in the document, when known.
pub(crate) fn declaration(name: &str, written: &Value, type_at: Option<Location>) -> Declaration {
    if let Some(short) = written.as_str() {
        let (type_text, default) = match split_default(short) {
            Some((type_text, default)) => (type_text, short_default(default.trim())),
            None => (short, None),
        };
        let (declared, optional) = type_of(type_text);
        let unknown_at = unknown_at(&declared, type_text, type_at);
        return Declaration {
            name: name.to_string(),
            declared,
            optional,
            default,
            doc: None,
            unknown_at,
            type_at,
        };
    }
    let type_text = written.get("type").and_then(Value::as_str).unwrap_or("");
    let (declared, optional) = type_of(type_text);
    Declaration {
        name: name.to_string(),
        unknown_at: unknown_at(&declared, type_text, type_at),
        type_at,
        declared,
        optional,
        default: written.get("default").cloned().map(DefaultValue::Yaml),
        doc: written
            .get("doc")
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

/// Where the name of `declared` that isn't a type name is, in `type_text`, which starts at `type_at`: the last place
/// it is written, since in `input<…>` it is the inner one.
fn unknown_at(
    declared: &DeclaredType,
    type_text: &str,
    type_at: Option<Location>,
) -> Option<Location> {
    let unknown = declared.unknown()?;
    let at = type_at?;
    let before = type_text
        .rfind(unknown)
        .map_or(0, |byte| type_text[..byte].chars().count());
    Some(Location::new(at.line, at.column + before))
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

/// A type and whether it ends in `?`. `enum` is a type only inside `input<…>`: alone, it isn't one.
fn type_of(text: &str) -> (DeclaredType, bool) {
    let text = text.trim();
    let (text, optional) = match text.strip_suffix('?') {
        Some(required) => (required.trim(), true),
        None => (text, false),
    };
    let declared = match declared_type(text) {
        DeclaredType::Name(TypeName::Enum) => DeclaredType::Other(text.to_string()),
        declared => declared,
    };
    (declared, optional)
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
    DeclaredType::named(text)
}
