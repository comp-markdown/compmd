//! The built-ins that produce content (spec §8.3, §8.4): `if` renders a branch, `each` its template once per item,
//! joined as its `join` says (F025), and `value` the text of its expression.

use super::components::{Invocation, Kind};
use super::env::Env;
use super::params::key_of;
use super::{Expander, origins};
use crate::cel::Value;
use crate::diagnostic::Code;
use crate::expanded::{self, Content};
use crate::tree::{self, DeclaredType, Param, ParamValue, Signature};
use crate::validate::params::value_at;
use std::rc::Rc;

/// How an `each`'s iterations combine (spec §8.3).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Join {
    None,
    List,
    Inline,
}

impl<'r> Expander<'r> {
    /// The value of the param `key`, read as `declared`.
    fn given(
        &mut self,
        given: &[&'r Param],
        key: &str,
        declared: &DeclaredType,
        env: &Env<'r>,
    ) -> Option<Value> {
        let param = given.iter().find(|param| key_of(param) == key)?;
        self.param_value(param, Some(declared), env)
    }

    /// `if`: its default slot when `when` is true, its `else` slot when it is false; both, as a container's
    /// fallback, when `when` has no value.
    pub(super) fn r#if(
        &mut self,
        invocation: &Invocation<'r>,
        given: &[&'r Param],
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        match self.given(given, "when", &named("boolean"), env) {
            Some(Value::Bool(true)) => out.extend(self.list(invocation.default_slot(), env)),
            Some(Value::Bool(false)) => out.extend(self.list(invocation.slot("else"), env)),
            _ => self.fallback(invocation, env, out),
        }
    }

    /// `value`: its `of` as text, in a paragraph of its own unless it is inline.
    pub(super) fn value(
        &mut self,
        invocation: &Invocation<'r>,
        given: &[&'r Param],
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        let text = self
            .given(given, "of", &named("string"), env)
            .and_then(|value| value.written().ok().flatten());
        let Some(text) = text else {
            return self.fallback(invocation, env, out);
        };
        let text = self.made(Content::Text(super::literal(&text)), None);
        if let Kind::Inline(_) = invocation.kind {
            out.push(text);
        } else {
            let paragraph = Content::Paragraph(tree::Parent {
                children: vec![text],
                position: None,
            });
            let made = self.made(paragraph, origins::of(invocation.node, env));
            out.push(made);
        }
    }

    /// `each`: its `before` slot and its template once per item, joined; its `empty` slot when there are none. An
    /// `in` that isn't a list is F024; data that failed to load is `null`, which has no items (spec §8.6).
    pub(super) fn each(
        &mut self,
        invocation: &Invocation<'r>,
        signature: Option<&Signature>,
        given: &[&'r Param],
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        let declared = |key: &str| {
            signature
                .and_then(|signature| signature.params.iter().find(|d| d.name == key))
                .map_or_else(
                    || named("string"),
                    |declaration| declaration.declared.clone(),
                )
        };
        let items = match self.given(given, "in", &named("array"), env) {
            Some(Value::List(items)) => items,
            Some(Value::Null) => Rc::default(),
            other => {
                if let (Some(value), Some(param)) =
                    (other, given.iter().find(|param| key_of(param) == "in"))
                {
                    self.report(
                        Code::F024,
                        env,
                        value_at(param),
                        format!("`in` gives a {}, not a list", value.kind()),
                    );
                }
                return self.fallback_template(invocation, env, out);
            }
        };
        let name = match given
            .iter()
            .find(|param| key_of(param) == "as")
            .map(|p| &p.value)
        {
            Some(ParamValue::Bare(name)) => name.clone(),
            _ => return self.fallback_template(invocation, env, out),
        };
        let text = |this: &mut Self, key: &str, otherwise: &str| {
            this.given(given, key, &declared(key), env)
                .and_then(|value| value.written().ok().flatten())
                .unwrap_or_else(|| otherwise.to_string())
        };
        let join = match text(self, "join", "none").as_str() {
            "list" => Join::List,
            "inline" => Join::Inline,
            _ => Join::None,
        };
        let sep = text(self, "sep", " ");
        let last = text(self, "last", &sep);
        if items.is_empty() {
            return out.extend(self.list(invocation.slot("empty"), env));
        }
        let template = invocation.default_slot();
        let iterations = self.expanding(env, invocation.at(), |this| {
            let mut iterations = Vec::new();
            for index in 0..items.len() {
                if this.over_limit() {
                    break;
                }
                iterations.push(this.list(template, &env.iteration(&name, &items, index)));
            }
            iterations
        });
        let Some(iterations) = iterations else {
            return self.fallback_template(invocation, env, out);
        };
        out.extend(self.list(invocation.slot("before"), env));
        let joined = match join {
            Join::List => joined_list(iterations),
            Join::Inline => joined_inline(iterations, &sep, &last),
            Join::None => Ok(iterations.into_iter().flatten().collect()),
        };
        match joined {
            Ok(nodes) => out.extend(nodes),
            Err((why, iterations)) => {
                let mode = if join == Join::List { "list" } else { "inline" };
                let at = invocation.at();
                self.report(
                    Code::F025,
                    env,
                    at,
                    format!("`join={mode}` needs {why}; the iterations are appended as they are"),
                );
                out.extend(iterations.into_iter().flatten());
            }
        }
    }

    /// An `each` that fails: its template once, as a plain renderer shows it (spec §4.5).
    pub(super) fn fallback_template(
        &mut self,
        invocation: &Invocation<'r>,
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        out.extend(self.list(invocation.default_slot(), env));
    }
}

/// A type by its name.
fn named(name: &str) -> DeclaredType {
    DeclaredType::Name(name.into())
}

type Iterations = Vec<Vec<expanded::Node>>;

/// `join=list`: every iteration's lists as one list, numbered on from the first and tight unless an item is spread;
/// the iterations as they are when one isn't lists of one type.
fn joined_list(iterations: Iterations) -> Result<Vec<expanded::Node>, (&'static str, Iterations)> {
    let lists: Option<Vec<&tree::List<expanded::Node>>> = iterations
        .iter()
        .flatten()
        .map(|node| match &node.content {
            Content::List(list) => Some(list),
            _ => None,
        })
        .collect();
    let ordered = lists
        .as_ref()
        .and_then(|lists| lists.first().map(|first| first.ordered));
    let one_type = lists
        .as_ref()
        .is_some_and(|lists| lists.iter().all(|list| Some(list.ordered) == ordered));
    if !one_type {
        return Err(("list items of one list type in each iteration", iterations));
    }
    let mut blocks = iterations.into_iter().flatten();
    let Some(mut first) = blocks.next() else {
        return Ok(Vec::new());
    };
    if let Content::List(list) = &mut first.content {
        for block in blocks {
            if let Content::List(more) = block.content {
                list.children.extend(more.children);
            }
        }
        list.spread = list.spread
            || list
                .children
                .iter()
                .any(|item| matches!(&item.content, Content::ListItem(item) if item.spread));
    }
    Ok(vec![first])
}

/// `join=inline`: every iteration's one paragraph as one paragraph, separated by `sep`, the last pair by `last`.
fn joined_inline(
    iterations: Iterations,
    sep: &str,
    last: &str,
) -> Result<Vec<expanded::Node>, (&'static str, Iterations)> {
    let one_paragraph = iterations.iter().all(|blocks| {
        matches!(blocks.as_slice(), [paragraph] if matches!(paragraph.content, Content::Paragraph(_)))
    });
    if !one_paragraph {
        return Err(("exactly one paragraph in each iteration", iterations));
    }
    let count = iterations.len();
    let mut paragraphs = iterations.into_iter().flatten();
    let Some(mut joined) = paragraphs.next() else {
        return Ok(Vec::new());
    };
    if let Content::Paragraph(paragraph) = &mut joined.content {
        for (index, next) in paragraphs.enumerate() {
            let separator = if index + 2 == count { last } else { sep };
            paragraph
                .children
                .push(expanded::Node::new(Content::Text(super::literal(
                    separator,
                ))));
            if let Content::Paragraph(next) = next.content {
                paragraph.children.extend(next.children);
            }
        }
        paragraph.children = super::text::joined(std::mem::take(&mut paragraph.children));
    }
    Ok(vec![joined])
}
