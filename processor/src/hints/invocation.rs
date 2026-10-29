//! Hints on component lines, inline components and slot lines (spec §3, §4): a component's name, its params and their
//! values, the params a list takes, and slots. The line is read again as the parser reads it, so a line being typed
//! has a hint too.

use super::{Completion, Found, ParameterHint, Place, Subject, expression};
use crate::parse::line::{Form, classify, matching_bracket};
use crate::parse::params::{self, List, Written, name_len};
use crate::tree::{Container, Node, ParamValue, SlotDeclaration};

/// The hint at the caret of `place`, on a component line or in an inline component.
pub(super) fn hint(place: &Place) -> Option<Found> {
    let indent = indent(&place.text);
    let line = &place.text[indent..];
    let on_line = place.at >= indent && line.starts_with(&[':', ':']);
    let found = on_line
        .then(|| component_line(place, indent))
        .flatten()
        .map(|found| Found {
            start: found.start + indent,
            end: found.end + indent,
            ..found
        });
    found.or_else(|| inline(place))
}

/// How many code points of `line` come before what a component line starts with: indentation, and the markers of
/// block quotes and list items.
fn indent(line: &[char]) -> usize {
    let space = |c: Option<&char>| matches!(c, Some(' ' | '\t'));
    let mut at = 0;
    loop {
        while space(line.get(at)) {
            at += 1;
        }
        let digits = line[at..].iter().take_while(|c| c.is_ascii_digit()).count();
        match line.get(at) {
            Some('>') => at += 1,
            Some('-' | '*' | '+') if space(line.get(at + 1)) => at += 1,
            Some(_)
                if digits > 0
                    && matches!(line.get(at + digits), Some('.' | ')'))
                    && space(line.get(at + digits + 1)) =>
            {
                at += digits + 1;
            }
            _ => return at,
        }
    }
}

/// The hint on a line that starts with `::`, whose caret is at `place.at - indent` of `line`; its indexes are the
/// line's past `indent`.
fn component_line(place: &Place, indent: usize) -> Option<Found> {
    let line = &place.text[indent..];
    let at = place.at - indent;
    let colons = line.iter().take_while(|c| **c == ':').count();
    let name_start = colons
        + line[colons..]
            .iter()
            .take_while(|c| matches!(c, ' ' | '\t'))
            .count();
    match classify(line) {
        Form::Opener { name, params } => {
            let end = name_start + name.chars().count();
            named(place, line, at, &name, name_start, Some((end, &params)))
        }
        Form::Leaf {
            name,
            fallback,
            params,
        } => {
            if fallback.is_some_and(|(start, end)| start <= at && at <= end) {
                return None;
            }
            let end = name_start + name.chars().count();
            let after = fallback.map_or(end, |(_, close)| close + 1);
            named(place, line, at, &name, name_start, Some((after, &params)))
        }
        Form::Closer { name } => named(place, line, at, &name, name_start + 1, None),
        Form::Continuation { params } => {
            params_of(place, line, at, &continued(place)?, &params, name_start + 1)
        }
        Form::Slot { name } => slot(place, at, &name, name_start + 1),
        Form::Malformed { .. } => {
            let len = name_len(line, name_start)?;
            let name: String = line[name_start..name_start + len].iter().collect();
            named(place, line, at, &name, name_start, None)
        }
    }
}

/// The hint on a component's name at `start` of `line`, or in its param list, which follows `after`.
fn named(
    place: &Place,
    line: &[char],
    at: usize,
    name: &str,
    start: usize,
    list: Option<(usize, &List)>,
) -> Option<Found> {
    let end = start + name.chars().count();
    if (start..=end).contains(&at) {
        return Some(component(place, name, start, end));
    }
    let (after, list) = list?;
    let brace = after
        + line[after..]
            .iter()
            .take_while(|c| matches!(c, ' ' | '\t'))
            .count();
    if line.get(brace) != Some(&'{') || at <= brace {
        return None;
    }
    params_of(place, line, at, name, list, brace + 1)
}

/// The hint on the name `name` at `start..end`.
fn component(place: &Place, name: &str, start: usize, end: usize) -> Found {
    Found {
        start,
        end,
        subject: Subject::Component {
            name: name.to_string(),
            component: place.component_hint(name),
        },
        completions: place.component_names(),
    }
}

/// The name of the component whose line the continuation line `place` continues.
fn continued(place: &Place) -> Option<String> {
    let lines = place.document.lines();
    (1..place.line)
        .rev()
        .find_map(|number| {
            let text: Vec<char> = lines.line(number)?.chars().collect();
            let line = &text[indent(&text)..];
            if !line.starts_with(&[':', ':']) {
                return Some(None);
            }
            match classify(line) {
                Form::Continuation { .. } => None,
                Form::Opener { name, .. } | Form::Leaf { name, .. } => Some(Some(name)),
                _ => Some(None),
            }
        })
        .flatten()
}

/// Where a param's key and value are on its line.
struct Spot {
    key: String,
    /// The key's code points.
    key_at: (usize, usize),
    /// The value's code points, past the `=`; `None` for a flag, or a key with no `=` yet.
    value_at: Option<(usize, usize)>,
}

/// The hint in the param list of the component `component`, which starts at `from` of `line`.
fn params_of(
    place: &Place,
    line: &[char],
    at: usize,
    component: &str,
    list: &List,
    from: usize,
) -> Option<Found> {
    let closed = list.broke_at.is_none()
        && list
            .end
            .checked_sub(1)
            .is_some_and(|last| line.get(last) == Some(&'}'));
    if at < from || (closed && at >= list.end) {
        return None;
    }
    let spot = list
        .params
        .iter()
        .find(|param| param.start <= at && at <= param.end)
        .map(spot_of)
        .or_else(|| broken(line, list, at))
        .unwrap_or_else(|| token(line, at));
    if let Some(found) = in_expression(place, line, at, &spot) {
        return Some(found);
    }
    let written: Vec<&str> = list.params.iter().filter_map(key_of).collect();
    let all = place.params(component);
    let others: Vec<ParameterHint> = all
        .iter()
        .filter(|param| param.name == spot.key || !written.contains(&param.name.as_str()))
        .cloned()
        .collect();
    let keys = others
        .iter()
        .map(|param| Completion::new(&param.name, Some(param.takes.description.clone())))
        .collect();
    let component = component.to_string();
    let (key_start, key_end) = spot.key_at;
    let found = match (
        all.into_iter().find(|param| param.name == spot.key),
        spot.value_at,
    ) {
        (Some(param), Some((start, end))) if at > key_end => Found {
            start,
            end,
            completions: param
                .takes
                .tokens
                .iter()
                .map(|token| Completion::new(token, None))
                .collect(),
            subject: Subject::Param {
                component,
                param,
                value: Some(line[start..end].iter().collect()),
            },
        },
        (Some(param), _) => Found {
            start: key_start,
            end: key_end,
            subject: Subject::Param {
                component,
                param,
                value: None,
            },
            completions: keys,
        },
        (None, _) => Found {
            start: key_start,
            end: key_end,
            subject: Subject::Params {
                component,
                params: others
                    .into_iter()
                    .filter(|param| param.name != spot.key)
                    .collect(),
            },
            completions: keys,
        },
    };
    Some(found)
}

/// The hint inside the expression that is the value at `spot`, `(…)`, when the caret is in it.
fn in_expression(place: &Place, line: &[char], at: usize, spot: &Spot) -> Option<Found> {
    let (start, end) = spot.value_at?;
    if line.get(start) != Some(&'(') {
        return None;
    }
    let inner = start + 1;
    let inner_end = if end > inner && line[end - 1] == ')' {
        end - 1
    } else {
        end
    };
    if at < inner || at > inner_end {
        return None;
    }
    let found = expression::expression(place, &line[inner..inner_end], at - inner)?;
    Some(Found {
        start: found.start + inner,
        end: found.end + inner,
        ..found
    })
}

/// The key a written param gives, `class` and `id` for `.name` and `#name`.
fn key_of(param: &Written) -> Option<&str> {
    match param.value {
        ParamValue::Class(_) => Some("class"),
        ParamValue::Id(_) => Some("id"),
        _ => param.key.as_deref(),
    }
}

/// Where a param the list read is.
fn spot_of(param: &Written) -> Spot {
    let key = key_of(param).unwrap_or_default().to_string();
    if matches!(param.value, ParamValue::Class(_) | ParamValue::Id(_)) {
        return Spot {
            key,
            key_at: (param.start, param.start),
            value_at: Some((param.start + 1, param.end)),
        };
    }
    let key_start = param.start
        + param
            .breakpoint
            .as_ref()
            .map_or(0, |breakpoint| breakpoint.chars().count() + 1);
    let key_end = key_start + key.chars().count();
    Spot {
        key,
        key_at: (key_start, key_end),
        value_at: (param.value != ParamValue::Flag).then_some((key_end + 1, param.end)),
    }
}

/// Where the param is whose quoted string, expression or JSON value opens at the list's break and doesn't close
/// before the caret: its value runs to the end of the line.
fn broken(line: &[char], list: &List, at: usize) -> Option<Spot> {
    let broke = list
        .broke_at
        .filter(|broke| at > *broke && matches!(line.get(*broke), Some('"' | '(' | '[' | '{')))?;
    let mut spot = token(line, broke);
    spot.value_at = spot.value_at.map(|(start, _)| (start, line.len()));
    Some(spot)
}

/// Where the param is that the caret is in when the list didn't read it: the text around the caret up to a space or
/// a brace, such as `border=` or a key being typed. Between params it is empty.
fn token(line: &[char], at: usize) -> Spot {
    let stop = |c: &&char| matches!(c, ' ' | '\t' | '{' | '}');
    let start = at - line[..at].iter().rev().take_while(|c| !stop(c)).count();
    let end = at + line[at..].iter().take_while(|c| !stop(c)).count();
    match line.get(start) {
        Some(marker @ ('.' | '#')) if start < end => {
            return Spot {
                key: if *marker == '.' { "class" } else { "id" }.to_string(),
                key_at: (start, start),
                value_at: Some((start + 1, end)),
            };
        }
        _ => {}
    }
    let equals = line[start..end].iter().position(|c| *c == '=');
    let head_end = equals.map_or(end, |equals| start + equals);
    let key_start = line[start..head_end]
        .iter()
        .position(|c| *c == ':')
        .map_or(start, |colon| start + colon + 1);
    Spot {
        key: line[key_start..head_end].iter().collect(),
        key_at: (key_start, head_end),
        value_at: equals.map(|equals| (start + equals + 1, end)),
    }
}

/// The hint on a slot line, whose name is at `start`.
fn slot(place: &Place, at: usize, name: &str, start: usize) -> Option<Found> {
    let end = start + name.chars().count();
    if at + 1 < start {
        return None;
    }
    let container = container_of(&place.document.root.children, place.line)?;
    let slots: Vec<SlotDeclaration> = place
        .component(&container.name)
        .and_then(|component| component.signature.as_ref()?.slots.clone())
        .unwrap_or_default();
    let completions = slots
        .iter()
        .map(|slot| match slot {
            SlotDeclaration::Name(slot) | SlotDeclaration::Template { name: slot, .. } => slot,
        })
        .filter(|slot| *slot != "default")
        .map(|slot| Completion::new(slot, None))
        .collect();
    Some(Found {
        start,
        end,
        subject: Subject::Slot {
            component: container.name.clone(),
            slot: name.to_string(),
            slots,
        },
        completions,
    })
}

/// The container whose slot line is on `line`.
fn container_of(nodes: &[Node], line: usize) -> Option<&Container> {
    nodes.iter().find_map(|node| match node {
        Node::ComponentContainer(container) => {
            let here = container.slots.iter().any(|slot| {
                slot.position
                    .is_some_and(|position| position.start.line == line)
            });
            if here {
                return Some(container);
            }
            container_of(&container.children, line).or_else(|| {
                container
                    .slots
                    .iter()
                    .find_map(|slot| container_of(&slot.children, line))
            })
        }
        _ => container_of(node.children(), line),
    })
}

/// The hint in an inline component, `:name[fallback]{params}`, on the caret's line; the innermost one, when one is
/// in another's fallback.
fn inline(place: &Place) -> Option<Found> {
    let (text, at) = (&place.text, place.at);
    let mut found = None;
    for colon in 0..text.len() {
        let after_word = colon
            .checked_sub(1)
            .is_some_and(|before| text[before] == ':' || text[before].is_alphanumeric());
        if text[colon] != ':' || after_word || text.get(colon + 1) == Some(&':') {
            continue;
        }
        let Some(len) = name_len(text, colon + 1) else {
            continue;
        };
        let (start, end) = (colon + 1, colon + 1 + len);
        if text.get(end) != Some(&'[') {
            continue;
        }
        let name: String = text[start..end].iter().collect();
        if (start..=end).contains(&at) {
            found = Some(component(place, &name, start, end));
            continue;
        }
        let Some(close) = matching_bracket(text, end) else {
            continue;
        };
        let brace = close + 1;
        if text.get(brace) == Some(&'{') && at > brace {
            let list = params::braced(text, brace);
            found = params_of(place, text, at, &name, &list, brace + 1).or(found);
        }
    }
    found
}
