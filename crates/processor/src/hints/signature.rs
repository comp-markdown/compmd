//! Signature help: the call an expression's place is in, with the argument it is at, or the param list of a
//! component, with every param it takes and the one the place is on. Like a hint, it reads the place's line again,
//! so a call being typed (`md.number(1, |`) has its help.

use super::{ParameterHint, Place, Subject, invocation};
use crate::functions::{self, Function};
use crate::resolve::Resolved;
use crate::surrounding::{Surrounding, surrounding};
use serde::Serialize;

/// What is being written at a place: a function's arguments, or a component's params.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SignatureHelp {
    /// In a call, in an expression param.
    Call {
        /// The function, as the catalog has it.
        function: Function,
        /// The argument the place is in, from 0.
        argument: usize,
    },
    /// In a component's param list.
    Params {
        /// The component's name.
        component: String,
        /// Every param it takes, as [`super::Subject::Params`] lists them.
        params: Vec<ParameterHint>,
        /// The one the place is on, by its index in `params`.
        #[serde(skip_serializing_if = "Option::is_none")]
        active: Option<usize>,
    },
}

/// The help at code point `offset` of the processed document of `resolved`.
pub(crate) fn at(resolved: &Resolved, offset: usize) -> Option<SignatureHelp> {
    let place = Place::at(resolved, offset);
    if surrounding(&place.document.root.children, offset) != Surrounding::Text {
        return None;
    }
    if let Some(call) = call(&place.text, place.at) {
        return Some(call);
    }
    let found = invocation::hint(&place)?;
    let (component, written) = match found.subject {
        Subject::Param {
            component, param, ..
        } => (component, Some(param.name)),
        Subject::Params { component, .. } => (component, None),
        _ => return None,
    };
    let params = place.params(&component);
    let active = written.and_then(|name| params.iter().position(|param| param.name == name));
    Some(SignatureHelp::Params {
        component,
        params,
        active,
    })
}

/// The call that `at` of `line` is in, inside an expression param, `=(…)`: the innermost parenthesis that a name
/// opens, skipping those that only group.
fn call(line: &[char], at: usize) -> Option<SignatureHelp> {
    // Each parenthesis open before `at`, with the commas in it since.
    let mut open: Vec<(usize, usize)> = Vec::new();
    let mut quote = None;
    let mut i = 0;
    while i < at.min(line.len()) {
        let c = line[i];
        match quote {
            Some(_) if c == '\\' => i += 1,
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None => match c {
                '"' | '\'' => quote = Some(c),
                '(' => open.push((i, 0)),
                ')' => {
                    open.pop();
                }
                ',' => {
                    if let Some((_, commas)) = open.last_mut() {
                        *commas += 1;
                    }
                }
                _ => {}
            },
        }
        i += 1;
    }
    let in_expression = open.iter().any(|(paren, _)| {
        paren
            .checked_sub(1)
            .is_some_and(|before| line[before] == '=')
    });
    if !in_expression {
        return None;
    }
    open.iter().rev().find_map(|&(paren, argument)| {
        let name_len = line[..paren]
            .iter()
            .rev()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.'))
            .count();
        let written: String = line[paren - name_len..paren].iter().collect();
        // `md.number` is an `md.` function; `md.doc.tags.map`, a method on a value.
        let name = if written.starts_with("md.") && written.matches('.').count() == 1 {
            written.as_str()
        } else {
            written.rsplit('.').next().unwrap_or_default()
        };
        let function = (!name.is_empty())
            .then(|| functions::named(name))
            .flatten()?;
        Some(SignatureHelp::Call { function, argument })
    })
}
