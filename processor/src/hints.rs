//! Positional hints: what is at a place in a processed document's text, and what it takes. An editor shows a hint
//! beside the text and offers its completions; a language server makes its hovers and completions of the same hint.
//!
//! A hint is a lookup in what the processor keeps, never a stage run again: it finds the block a place is in, reads
//! the place's line again, and looks its names up. Reading the line again gives half-typed text a hint too, such as
//! `border=` or a list whose `}` isn't written yet.

mod invocation;

use crate::components;
use crate::diagnostic::{Diagnostic, Location, Range};
use crate::resolve::{Component, Document, Resolved};
use crate::settings::Input;
use crate::tree::{
    Declaration, DeclaredType, DefaultValue, Node, ParamValue, Signature, SlotDeclaration,
    StringPart,
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
        line: caret.line,
        at: (caret.column - 1).min(text.len()),
        text,
    };
    let found = match surrounding(&document.root.children, offset) {
        Surrounding::Text => invocation::hint(&place),
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
            defined: component.position.map(|position| Range {
                start: position.start.into(),
                end: Some(position.end.into()),
            }),
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

/// Whether a place is in text whose component syntax counts, or in a block that holds other text: code, math, raw
/// HTML, a signature, a definition's header or host-language code.
#[derive(Debug, PartialEq, Eq)]
enum Surrounding {
    Text,
    Opaque,
}

/// What surrounds code point `offset` of the blocks `nodes`. The blocks are in order, so the one a place is in is the
/// last that starts before it.
fn surrounding(nodes: &[Node], offset: usize) -> Surrounding {
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
        Node::Code(_)
        | Node::Math(_)
        | Node::Html(_)
        | Node::InlineCode(_)
        | Node::InlineMath(_)
        | Node::Yaml(_)
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
mod tests {
    use super::*;
    use crate::context::RenderContext;
    use crate::expand::Options;
    use crate::tree::Literal;
    use crate::{Ambient, ParseOptions, Processor};
    use serde_json::{Value, json};
    use std::collections::HashMap;
    use std::rc::Rc;

    /// The caret in the texts of these tests.
    const CARET: char = '‸';

    /// The hint at the caret of `text`, with `files`, as JSON.
    fn hint_with(text: &str, files: &[(&str, &str)]) -> Value {
        let offset = text.chars().position(|c| c == CARET).expect("a caret");
        let text = text.replacen(CARET, "", 1);
        let files: HashMap<String, String> = files
            .iter()
            .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
            .collect();
        let context = RenderContext {
            now: 0,
            tz: "UTC".into(),
            time: Rc::new(()),
            inputs: serde_json::Map::default(),
        };
        let processor = Processor::new("a.comp.md", text)
            .parse(&ParseOptions::default())
            .resolve(files, (), &Ambient::default())
            .validate(&[])
            .load(context, (), ())
            .expand(&Options::default());
        serde_json::to_value(processor.hint_at(offset)).expect("JSON")
    }

    fn hint(text: &str) -> Value {
        hint_with(text, &[])
    }

    /// The range on `line` from `start` to `end`, columns from 1.
    fn range(line: usize, start: usize, end: usize) -> Value {
        json!({ "start": { "line": line, "column": start }, "end": { "line": line, "column": end } })
    }

    /// The texts of a hint's completions.
    fn texts(hint: &Value) -> Vec<&str> {
        hint["completions"]
            .as_array()
            .expect("completions")
            .iter()
            .map(|completion| completion["text"].as_str().expect("a text"))
            .collect()
    }

    /// The names of a `params` subject's params.
    fn names(hint: &Value) -> Vec<&str> {
        hint["subject"]["params"]
            .as_array()
            .expect("params")
            .iter()
            .map(|param| param["name"].as_str().expect("a name"))
            .collect()
    }

    const CARD: &str = "```compmd component=card\nparams:\n  kind: note | warning = note\n  title:\n    type: string\n    doc: The card's heading.\nslots: [default, footer]\n---\n{{title}}\n```\n\n";

    #[test]
    fn a_name_is_the_component_it_stands_for() {
        let json = hint(&format!(
            "{CARD}::: ca{CARET}rd {{title=\"Setup\"}}\nText.\n::: /card\n"
        ));
        assert_eq!(json["range"], range(12, 5, 9));
        let subject = &json["subject"];
        assert_eq!(
            (&subject["kind"], &subject["name"]),
            (&json!("component"), &json!("card"))
        );
        let card = &subject["component"];
        assert_eq!(
            (&card["kind"], &card["provided"]),
            (&json!("compmd"), &json!(false))
        );
        assert_eq!(card.get("file"), None, "defined in the document");
        assert_eq!(card["defined"]["start"], json!({ "line": 1, "column": 1 }));
        assert_eq!(card["signature"]["params"][1]["doc"], "The card's heading.");
        let completions = texts(&json);
        assert!(completions.contains(&"card") && completions.contains(&"box"));
        let closer = hint(&format!("{CARD}::: card {{title=x}}\n::: /c{CARET}ard\n"));
        assert_eq!(closer["range"], range(13, 6, 10));
        assert_eq!(closer["subject"]["name"], "card");
    }

    #[test]
    fn a_built_in_is_cores_and_an_unknown_name_has_its_diagnostic() {
        let json = hint(&format!("::: bo{CARET}x\n::: /box\n"));
        let core = &json["subject"]["component"];
        assert_eq!(
            (&core["kind"], &core["file"], &core["provided"]),
            (
                &json!("processor"),
                &json!("@compmd/core@1.0.0/index.comp.md"),
                &json!(true)
            )
        );
        let unknown = hint(&format!("::: bxo{CARET}\n::: /bxo\n"));
        assert_eq!(unknown["subject"].get("component"), None);
        assert_eq!(unknown["diagnostics"][0]["code"], "F002");
        assert_eq!(unknown["diagnostics"][0]["fix"]["text"], "box");
        let typed = hint(&format!("::: car{CARET}"));
        assert_eq!(typed["subject"]["name"], "car", "a name being typed");
    }

    #[test]
    fn a_key_says_what_its_param_takes() {
        let json = hint(&format!("::: box {{padd{CARET}ing=m}}\n::: /box\n"));
        assert_eq!(json["range"], range(1, 10, 17));
        let subject = &json["subject"];
        assert_eq!(
            (&subject["kind"], &subject["component"]),
            (&json!("param"), &json!("box"))
        );
        assert_eq!(subject.get("value"), None);
        let padding = &subject["param"];
        assert_eq!(
            (&padding["origin"], &padding["breakpoints"]),
            (&json!("common"), &json!(true))
        );
        assert_eq!(
            padding["takes"]["description"],
            "one or two of `none`, `s`, `m`, `l` or a length in `px`, `rem`, `em` or `%`, such as `s l`"
        );
        let keys = texts(&json);
        assert!(keys.contains(&"padding") && keys.contains(&"gap") && keys.contains(&"lang"));
    }

    #[test]
    fn a_value_offers_the_tokens_its_param_takes() {
        let json = hint(&format!("::: box {{border=sub{CARET}tle}}\n::: /box\n"));
        assert_eq!(json["range"], range(1, 17, 23));
        assert_eq!(json["subject"]["value"], "subtle");
        assert_eq!(texts(&json), ["none", "subtle", "strong"]);
        let empty = hint(&format!("::: box {{border={CARET}}}\n::: /box\n"));
        assert_eq!(
            empty["subject"]["value"], "",
            "after an `=` with nothing yet"
        );
        assert_eq!(empty["range"], range(1, 17, 17));
        assert_eq!(empty["diagnostics"][0]["code"], "F011");
        let unclosed = hint(&format!("::: box {{padding=m border={CARET}"));
        assert_eq!(
            (
                &unclosed["subject"]["param"]["name"],
                &unclosed["subject"]["value"]
            ),
            (&json!("border"), &json!(""))
        );
    }

    #[test]
    fn between_params_the_list_offers_those_not_written() {
        let json = hint(&format!("::: box {{padding=m {CARET}}}\n::: /box\n"));
        assert_eq!(json["subject"]["kind"], "params");
        assert_eq!(json["range"], range(1, 20, 20));
        let params = names(&json);
        assert!(params.contains(&"display") && params.contains(&"border"));
        assert!(!params.contains(&"padding"));
        assert_eq!(params[0], "display", "the declared params first");
        let typed = hint(&format!("::: box {{padding=m bord{CARET}"));
        assert_eq!(typed["subject"]["kind"], "params", "a key being typed");
        assert_eq!(typed["range"], range(1, 20, 24));
        assert!(texts(&typed).contains(&"border"));
        assert_eq!(
            hint(&format!("::: box {{padding=m}}{CARET}\n::: /box\n")),
            Value::Null
        );
    }

    #[test]
    fn a_declared_param_has_its_declaration() {
        let json = hint(&format!(
            "{CARD}::: card {{kind=w{CARET} title=x}}\n::: /card\n"
        ));
        let kind = &json["subject"]["param"];
        assert_eq!(
            (&kind["origin"], &kind["default"], &kind["required"]),
            (&json!("declared"), &json!("note"), &json!(false))
        );
        assert_eq!(texts(&json), ["note", "warning"]);
        let title = hint(&format!("{CARD}::: card {{ti{CARET}tle=x}}\n::: /card\n"));
        let title = &title["subject"]["param"];
        assert_eq!(
            (&title["doc"], &title["required"]),
            (&json!("The card's heading."), &json!(true))
        );
        let gap = hint(&format!("::: box {{md:ga{CARET}p=l}}\n::: /box\n"));
        assert_eq!(gap["range"], range(1, 13, 16));
        assert_eq!(gap["subject"]["param"]["origin"], "declared");
        assert_eq!(gap["subject"]["param"]["breakpoints"], true);
    }

    #[test]
    fn a_bound_input_offers_the_inputs_it_binds() {
        let text = format!(
            "---\nmd:\n  inputs:\n    since: number = 1970\n    name: string = \"\"\n---\n\n```compmd component=slider\nparams:\n  of: input<number>\n---\nS.\n```\n\n:: slider[] {{of={CARET}}}\n"
        );
        let json = hint(&text);
        let takes = &json["subject"]["param"]["takes"];
        assert_eq!(
            takes["description"],
            "the name of an input of a number under `md.inputs`"
        );
        assert_eq!(texts(&json), ["since"]);
    }

    #[test]
    fn strings_classes_and_ids_are_values() {
        let quoted = hint(&format!("::: box {{title=\"a b{CARET}"));
        assert_eq!(quoted["subject"]["kind"], "params", "`box` has no `title`");
        assert_eq!(quoted["range"], range(1, 10, 15));
        let open = hint(&format!("{CARD}::: card {{title=\"a b{CARET}"));
        assert_eq!(open["subject"]["value"], "\"a b", "to the end of the line");
        let class = hint(&format!("::: box {{.wi{CARET}de}}\n::: /box\n"));
        assert_eq!(
            (
                &class["subject"]["param"]["name"],
                &class["subject"]["value"]
            ),
            (&json!("class"), &json!("wide"))
        );
        let id = hint(&format!("::: box {{#in{CARET}tro .x}}\n::: /box\n"));
        assert_eq!(id["subject"]["param"]["name"], "id");
        let typed = hint(&format!("::: box {{.x #b{CARET}"));
        assert_eq!(
            (
                &typed["subject"]["param"]["name"],
                &typed["subject"]["value"]
            ),
            (&json!("id"), &json!("b"))
        );
    }

    #[test]
    fn leaves_inline_components_and_continuation_lines() {
        let leaf = hint(&format!("{CARD}:: card[Sum{CARET}mary] {{title=x}}\n"));
        assert_eq!(leaf, Value::Null, "the fallback is text");
        let params = hint(&format!("{CARD}:: card[S] {{title=x ki{CARET}nd=note}}\n"));
        assert_eq!(params["subject"]["param"]["name"], "kind");
        let inline = hint(
            &format!("{CARD}Read :card[it]{{kind={CARET}}} and :bo{CARET}x[b].\n")
                .replacen(CARET, "", 1),
        );
        assert_eq!(inline["subject"]["name"], "box");
        let inline = hint(&format!(
            "{CARD}Read :card[it :box[b]{{bor{CARET}der=s}}] now.\n"
        ));
        assert_eq!(
            inline["subject"]["param"]["name"], "border",
            "inside another's fallback"
        );
        let quoted = hint(&format!("> - :: card[S] {{ti{CARET}tle=x}}\n"));
        assert_eq!(
            quoted["subject"]["kind"], "params",
            "no `card` here, so the universal params"
        );
        let continued = hint(&format!(
            "::: box {{gap=s}}\n::: + bor{CARET}der=subtle\nText.\n::: /box\n"
        ));
        assert_eq!(continued["subject"]["param"]["name"], "border");
        assert_eq!(continued["range"], range(2, 7, 13));
        let orphan = hint(&format!("Text.\n::: + bor{CARET}der=subtle\n"));
        assert_eq!(orphan["subject"], Value::Null, "F012 only");
        assert_eq!(orphan["diagnostics"][0]["code"], "F012");
    }

    #[test]
    fn a_slot_line_names_the_containers_slots() {
        let json = hint(&format!(
            "{CARD}::: card {{title=x}}\nBody.\n:: #foo{CARET}ter\nEnd.\n::: /card\n"
        ));
        assert_eq!(json["range"], range(14, 5, 11));
        let subject = &json["subject"];
        assert_eq!(
            (&subject["component"], &subject["slot"]),
            (&json!("card"), &json!("footer"))
        );
        assert_eq!(subject["slots"], json!(["default", "footer"]));
        assert_eq!(texts(&json), ["footer"]);
        let each = hint(&format!(
            "::: box\n::: each {{in=([1]) as=n}}\n{{{{n}}}}\n:: #emp{CARET}ty\nNone.\n::: /each\n::: /box\n"
        ));
        assert_eq!(
            each["subject"]["component"], "each",
            "the innermost container"
        );
        assert_eq!(texts(&each), ["before", "empty"]);
        let outside = hint(&format!(":: #foo{CARET}ter\n"));
        assert_eq!(outside["diagnostics"][0]["code"], "F015");
    }

    #[test]
    fn code_math_and_prose_have_no_hint() {
        assert_eq!(
            hint(&format!("```md\n::: box {{bor{CARET}der=x}}\n```\n")),
            Value::Null
        );
        assert_eq!(
            hint(&format!("Some `:box[b]{{bor{CARET}der=x}}` code.\n")),
            Value::Null
        );
        assert_eq!(hint(&format!("Plain pro{CARET}se.\n")), Value::Null);
        assert_eq!(hint(&format!("$$\n:: box[{CARET}]\n$$\n")), Value::Null);
        let definition = format!(
            "```compmd component=note\npar{CARET}ams:\n  tone: string?\n---\n::: box {{border=subtle}}\nN.\n::: /box\n```\n"
        );
        assert_eq!(hint(&definition), Value::Null, "a definition's header");
        let body = format!(
            "```compmd component=note\nparams:\n  tone: string?\n---\n::: box {{border=sub{CARET}tle}}\nN.\n::: /box\n```\n"
        );
        assert_eq!(
            hint(&body)["subject"]["param"]["name"],
            "border",
            "a definition's body"
        );
        let host = format!("```jsx component=hello\nconst x = <a>{{bor{CARET}der=1}}</a>;\n```\n");
        assert_eq!(hint(&host), Value::Null);
    }

    #[test]
    fn a_host_language_component_without_a_signature_takes_the_universal_params() {
        let text = format!(
            "---\nmd:\n  import:\n    - from: ./Chart.jsx\n      names: [{{name: Chart, as: chart}}]\n---\n\n:: chart[C] {{{CARET}}}\n"
        );
        let json = hint_with(&text, &[("Chart.jsx", "export default () => null;")]);
        let params = names(&json);
        assert_eq!(params[0], "width");
        let name = hint_with(
            &text.replace("{‸}", "{}").replace(":: chart", ":: ch‸art"),
            &[("Chart.jsx", "export default () => null;")],
        );
        assert_eq!(name["subject"]["component"]["kind"], "jsx");
        assert_eq!(name["subject"]["component"]["file"], "Chart.jsx");
    }

    #[test]
    fn a_diagnostic_alone_has_a_hint_without_a_subject() {
        let json = hint(&format!("Say {{{{ 1 {CARET}}}}}.\n"));
        assert_eq!(json["subject"], Value::Null);
        assert_eq!(json["diagnostics"][0]["code"], "W106");
        assert_eq!(json["range"], range(1, 10, 10));
    }

    #[test]
    fn defaults_are_written_as_params_write_them() {
        let text = |value: &str| {
            StringPart::Text(Literal {
                value: value.into(),
                position: None,
            })
        };
        let interpolation = StringPart::Interpolation(crate::tree::Interpolation {
            path: "n".into(),
            ..Default::default()
        });
        for (default, expected) in [
            (ParamValue::Bare("m".into()), "m"),
            (ParamValue::Expr("1 + 1".into()), "(1 + 1)"),
            (ParamValue::Flag, "true"),
            (
                ParamValue::String(vec![text("Step "), interpolation]),
                "\"Step {{n}}\"",
            ),
        ] {
            assert_eq!(written(&DefaultValue::Param(default)), expected);
        }
        assert_eq!(written(&DefaultValue::Yaml(json!("a b"))), "a b");
        assert_eq!(written(&DefaultValue::Yaml(json!([1]))), "[1]");
    }
}
