//! Hints in interpolations and expressions (spec §8.1, §8.2): what each step of a path names, with its type and the
//! names at that step, and the profile's functions. A place sees what the validator gives it: `md`, a definition's
//! params in its body, and the names templates and macros bind.

use super::{Completion, Found, Place, Subject};
use crate::cel::{self, Program, Type};
use crate::md::{self, Signatures};
use crate::tree::{
    Container, Definition, DefinitionBody, Node, ParamValue, Signature, SlotDeclaration,
};
use crate::validate::scope::Scope;
use serde::Serialize;

/// What a step of a path names.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NameHint {
    /// The step, as written.
    pub name: String,
    /// Its type, as CEL writes it; `None` for a name nothing here has.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub ty: Option<String>,
    /// What it is: "data from `./sales.csv`", "a param of `card`".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    /// The fields of a record, or of a list's records, each `name: type`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<String>,
}

/// A function of the profile, or an `md.` function.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FunctionHint {
    /// Each way to call it: `string.indexOf(string) → int`.
    pub signatures: Vec<String>,
    /// What it gives.
    pub doc: String,
}

/// What a place sees.
pub(super) struct Sees<'r> {
    scope: Scope<'r>,
    /// The `compmd` definition whose body the place is in.
    definition: Option<(&'r str, &'r Signature)>,
    /// The names templates bind here, each with what it iterates, as written.
    bound: Vec<(String, String)>,
}

/// What code point `place.offset` sees.
pub(super) fn sees<'r>(place: &Place<'r>) -> Sees<'r> {
    let mut sees = Sees {
        scope: Scope::body(place.resolved, place.resolved.main),
        definition: None,
        bound: Vec::new(),
    };
    walk(place, &place.document.root.children, &mut sees);
    sees
}

/// Narrows what `sees` holds to the block of `nodes` the place is in, and the blocks in it.
fn walk<'r>(place: &Place<'r>, nodes: &'r [Node], sees: &mut Sees<'r>) {
    let offset = place.offset;
    let starts_before = nodes.partition_point(|node| {
        node.position()
            .is_none_or(|position| position.start.offset <= offset)
    });
    let Some(block) = starts_before.checked_sub(1).map(|index| &nodes[index]) else {
        return;
    };
    match block {
        Node::ComponentDefinition(
            definition @ Definition {
                body: DefinitionBody::Compmd { children },
                ..
            },
        ) => {
            let signature = definition.signature.as_ref();
            sees.scope = Scope::definition(&place.document.settings, signature);
            sees.definition = signature.map(|signature| (definition.name.as_str(), signature));
            walk(place, children, sees);
        }
        Node::ComponentContainer(container) => {
            let past_opener = container
                .position
                .is_some_and(|position| position.end.offset < offset);
            let before_closer = container
                .close_position
                .is_none_or(|position| offset <= position.start.offset);
            if !(past_opener && before_closer) {
                return;
            }
            let slot = container.slots.iter().rev().find(|slot| {
                slot.position
                    .is_some_and(|position| position.start.offset <= offset)
            });
            bind(
                place,
                container,
                slot.map_or("default", |slot| &slot.name),
                sees,
            );
            walk(
                place,
                slot.map_or(&container.children, |slot| &slot.children),
                sees,
            );
        }
        _ => walk(place, block.children(), sees),
    }
}

/// Binds the name the template slot `slot` of `container` binds, when it is one (spec §7.5).
fn bind(place: &Place, container: &Container, slot: &str, sees: &mut Sees) {
    let slots = place
        .component(&container.name)
        .and_then(|component| component.signature.as_ref()?.slots.as_ref());
    let template = slots
        .into_iter()
        .flatten()
        .find_map(|declared| match declared {
            SlotDeclaration::Template {
                name,
                each,
                binding,
            } if name == slot => Some((each, binding)),
            _ => None,
        });
    let Some((each, binding)) = template else {
        return;
    };
    let value = |key: &str| {
        container
            .params
            .iter()
            .find(|param| param.key.as_deref() == Some(key))
            .map(|param| &param.value)
    };
    let Some(ParamValue::Bare(bound)) = value(binding) else {
        return;
    };
    let (iterated, written) = match value(each) {
        Some(ParamValue::Expr(source)) => {
            (type_of(source, &sees.scope.cel()), format!("({source})"))
        }
        Some(ParamValue::Json(json)) => (
            serde_json::from_str(json)
                .ok()
                .map(|value| Type::of_json(&value)),
            json.clone(),
        ),
        Some(ParamValue::Bare(text)) => (None, text.clone()),
        _ => (None, each.clone()),
    };
    sees.scope = sees.scope.template(bound, element(iterated));
    sees.bound.push((bound.clone(), written));
}

/// The type of the expression `source` where `scope` is seen.
fn type_of(source: &str, scope: &cel::Scope) -> Option<Type> {
    Program::parse(source).ok()?.check(scope, &Signatures).ok()
}

/// The type of the elements of a list of type `list`.
fn element(list: Option<Type>) -> Type {
    match list {
        Some(Type::List(element)) => (*element).clone(),
        _ => Type::Dyn,
    }
}

/// Whether `c` can be in a path: an identifier's letters, digits and `_`.
fn in_name(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// The hint in an interpolation, `{{path}}`, on the caret's line.
pub(super) fn interpolation(place: &Place) -> Option<Found> {
    let (text, at) = (&place.text, place.at);
    let open = (0..at.saturating_sub(1))
        .rev()
        .find(|&i| text[i] == '{' && text[i + 1] == '{')?;
    let escaped = open
        .checked_sub(1)
        .is_some_and(|before| text[before] == '\\');
    let closed = text[open + 2..at].windows(2).any(|pair| pair == ['}', '}']);
    if escaped || closed {
        return None;
    }
    let start = open
        + 2
        + text[open + 2..]
            .iter()
            .take_while(|c| matches!(c, ' ' | '\t'))
            .count();
    let end = start
        + text[start..]
            .iter()
            .take_while(|c| in_name(**c) || **c == '.')
            .count();
    if at < start || at > end {
        return None;
    }
    let sees = sees(place);
    Some(path(place, &sees, &text[..end], start, at, &[]))
}

/// The hint in the expression `source`, a param's value without its parentheses, whose caret is at `at`.
pub(super) fn expression(place: &Place, source: &[char], at: usize) -> Option<Found> {
    let quotes = source[..at]
        .iter()
        .filter(|c| matches!(c, '"' | '\''))
        .count();
    if quotes % 2 == 1 {
        return None;
    }
    let start = at
        - source[..at]
            .iter()
            .rev()
            .take_while(|c| in_name(**c) || **c == '.')
            .count();
    let end = at + source[at..].iter().take_while(|c| in_name(**c)).count();
    if start == end || source[start].is_ascii_digit() {
        return None;
    }
    let sees = sees(place);
    let locals = locals(&sees, source, at);
    let rest = &source[end..];
    let called = rest
        .iter()
        .find(|c| !matches!(c, ' ' | '\t'))
        .is_some_and(|c| *c == '(');
    if called {
        return Some(function(source, start, end));
    }
    Some(path(place, &sees, &source[..end], start, at, &locals))
}

/// The names the macros around `at` bind in `source`, `b` in `md.data.books.exists(b, …)`, with the types of the
/// elements they iterate and what they iterate, as written.
fn locals(sees: &Sees, source: &[char], at: usize) -> Vec<(String, Type, String)> {
    let mut found: Vec<(String, Type, String)> = Vec::new();
    let names = cel::function_names(true);
    for dot in (0..at).filter(|&i| source[i] == '.') {
        let name_end = dot
            + 1
            + source[dot + 1..]
                .iter()
                .take_while(|c| in_name(**c))
                .count();
        let name: String = source[dot + 1..name_end].iter().collect();
        if source.get(name_end) != Some(&'(') || !names.contains(&name.as_str()) || name_end > at {
            continue;
        }
        let close = matching(source, name_end);
        if close.is_some_and(|close| close < at) {
            continue;
        }
        let var_start = name_end
            + 1
            + source[name_end + 1..]
                .iter()
                .take_while(|c| matches!(c, ' ' | '\t'))
                .count();
        let var_end = var_start
            + source[var_start..]
                .iter()
                .take_while(|c| in_name(**c))
                .count();
        if var_start == var_end || !source[var_end..].contains(&',') {
            continue;
        }
        let range_start = dot
            - source[..dot]
                .iter()
                .rev()
                .take_while(|c| in_name(**c) || **c == '.')
                .count();
        let range: String = source[range_start..dot].iter().collect();
        let mut scope = sees.scope.cel();
        for (name, ty, _) in &found {
            scope = scope.with(name.clone(), Type::clone(ty));
        }
        let iterated = resolve(&scope, &range.split('.').collect::<Vec<_>>());
        found.push((
            source[var_start..var_end].iter().collect(),
            element(iterated),
            range,
        ));
    }
    found
}

/// The index of the `)` that closes the `(` at `open`, if it is closed.
fn matching(source: &[char], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in source.iter().enumerate().skip(open) {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// The type `segments` name where `scope` is seen, past `md.` functions' names too; `None` past a name nothing has.
fn resolve(scope: &cel::Scope, segments: &[&str]) -> Option<Type> {
    let (first, rest) = segments.split_first()?;
    let mut ty = scope.get(first)?.clone();
    for segment in rest {
        ty = match ty {
            Type::Record(fields) => fields.get(*segment)?.clone(),
            Type::Map(_, value) => (*value).clone(),
            Type::Dyn => Type::Dyn,
            _ => return None,
        };
    }
    Some(ty)
}

/// The fields a value of type `ty` has, or the fields of each of its elements, as `name: type`.
fn fields(ty: &Type) -> Vec<(String, Type)> {
    match ty {
        Type::Record(fields) => fields
            .iter()
            .map(|(name, ty)| (name.clone(), ty.clone()))
            .collect(),
        Type::List(element) => match &**element {
            Type::Record(_) => fields(element),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

/// The hint on the path that ends at `end` of `text` and starts at `start`, whose caret is at `at`, with the names
/// `locals` bind there.
fn path(
    place: &Place,
    sees: &Sees,
    text: &[char],
    start: usize,
    at: usize,
    locals: &[(String, Type, String)],
) -> Found {
    let written: String = text[start..].iter().collect();
    let segments: Vec<&str> = written.split('.').collect();
    let step = text[start..at].iter().filter(|c| **c == '.').count();
    let step_start = start
        + segments[..step]
            .iter()
            .map(|segment| segment.chars().count() + 1)
            .sum::<usize>();
    let step_end = step_start + segments[step].chars().count();
    let mut scope = sees.scope.cel();
    for (name, ty, _) in locals {
        scope = scope.with(name.clone(), ty.clone());
    }
    let named = &segments[..=step];
    let ty = resolve(&scope, named);
    let about = match ty {
        None => sees.scope.missing_space(&named.join(".")),
        Some(_) => None,
    }
    .or_else(|| about(place, sees, named, locals));
    let name = NameHint {
        name: segments[step].to_string(),
        ty: ty.as_ref().map(ToString::to_string),
        about,
        fields: ty
            .as_ref()
            .map(fields)
            .unwrap_or_default()
            .into_iter()
            .map(|(name, ty)| format!("{name}: {ty}"))
            .collect(),
    };
    let siblings: Vec<(String, Type)> = if step == 0 {
        scope
            .names()
            .map(|name| {
                (
                    name.to_string(),
                    scope.get(name).cloned().unwrap_or(Type::Dyn),
                )
            })
            .collect()
    } else {
        resolve(&scope, &segments[..step])
            .map(|parent| match parent {
                Type::List(_) => Vec::new(),
                parent => fields(&parent),
            })
            .unwrap_or_default()
    };
    let mut completions: Vec<Completion> = siblings
        .iter()
        .map(|(name, ty)| Completion::new(name, Some(ty.to_string())))
        .collect();
    if segments[..step] == ["md"] {
        completions.extend(
            md_names()
                .iter()
                .map(|name| Completion::new(name, Some("function".to_string()))),
        );
    }
    Found {
        start: step_start,
        end: step_end,
        subject: Subject::Path {
            path: segments[..=step].join("."),
            name,
        },
        completions,
    }
}

/// The `md.` functions' names.
fn md_names() -> &'static [&'static str] {
    md::MdFunction::NAMES
}

/// What the path `segments` names, in words.
fn about(
    place: &Place,
    sees: &Sees,
    segments: &[&str],
    locals: &[(String, Type, String)],
) -> Option<String> {
    let settings = &place.document.settings;
    let text = match segments {
        ["md"] => "the document's own names: `doc`, `data`, `input`, `loop`, `slot`, `version`, `lang`, `tz`".to_string(),
        ["md", "doc"] => "the frontmatter's own keys".to_string(),
        ["md", "doc", keys @ ..] => format!("the frontmatter's `{}`", keys.join(".")),
        ["md", "data"] => "the data the frontmatter imports".to_string(),
        ["md", "data", name] => {
            let data = settings.data.iter().find(|data| data.name == *name)?;
            format!("data from `{}`", data.from.value)
        }
        ["md", "data", name, field, ..] => format!("the field `{field}` of `md.data.{name}`'s records"),
        ["md", "input"] => "the inputs the reader sets".to_string(),
        ["md", "input", name] => {
            let input = settings.inputs.iter().find(|input| input.declaration.name == *name)?;
            format!("the input `{name}`, {}", input.declaration.declared.written())
        }
        ["md", "loop"] => "the iteration of a template: `index`, `number`, `count`, `first`, `last`".to_string(),
        ["md", "loop", field] => loop_field(field)?.to_string(),
        ["md", "slot"] => "the default slot's content".to_string(),
        ["md", "slot", slot] => format!("the content of the slot `{slot}`"),
        ["md", "version"] => "the CompMD version the document is written in".to_string(),
        ["md", "lang"] => "the document's language, a BCP 47 tag".to_string(),
        ["md", "tz"] => "the render's time zone".to_string(),
        [name] => {
            if let Some((_, _, range)) = locals.iter().rev().find(|(local, ..)| local == name) {
                format!("each element of `{range}`")
            } else if let Some((_, written)) = sees.bound.iter().rev().find(|(bound, _)| bound == name) {
                format!("each element of `{written}`")
            } else {
                let (definition, signature) = sees.definition?;
                let declared = signature
                    .params
                    .iter()
                    .find(|param| param.name.replace('-', "_") == *name)?;
                match &declared.doc {
                    Some(doc) => format!("a param of `{definition}`: {doc}"),
                    None => format!("a param of `{definition}`"),
                }
            }
        }
        _ => return None,
    };
    Some(text)
}

/// What a field of `md.loop` holds (spec §8.3).
fn loop_field(field: &str) -> Option<&'static str> {
    Some(match field {
        "index" => "the element's position, from 0",
        "number" => "the element's position, from 1, for display",
        "count" => "how many elements there are",
        "first" => "whether it is the first element",
        "last" => "whether it is the last element",
        _ => return None,
    })
}

/// The hint on the name of a function called at `start..end` of `source`; the caret is in its name.
fn function(source: &[char], start: usize, end: usize) -> Found {
    let written: String = source[start..end].iter().collect();
    let segments: Vec<&str> = written.split('.').collect();
    let (name, receiver) = segments
        .split_last()
        .expect("splitting gives at least one part");
    let name_start = end - name.chars().count();
    let md = receiver == ["md"];
    let (described, names): (Option<cel::Described>, Vec<&str>) = if md {
        (md::described(name), md_names().to_vec())
    } else {
        (
            cel::described(name),
            cel::function_names(!receiver.is_empty()),
        )
    };
    let shown = if md {
        format!("md.{name}")
    } else {
        (*name).to_string()
    };
    Found {
        start: name_start,
        end,
        subject: Subject::Function {
            name: shown,
            hint: described.map(|described| FunctionHint {
                signatures: described.signatures,
                doc: described.doc.to_string(),
            }),
        },
        completions: names
            .into_iter()
            .map(|name| Completion::new(name, None))
            .collect(),
    }
}

#[cfg(test)]
mod tests;
