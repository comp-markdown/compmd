//! A `compmd` component's invocation (spec §7.2, §9.2): its body, expanded in its own document's scope with the
//! invocation's params, in a box styled by its fence's params under the caller's.

use super::{Invocation, Kind};
use crate::breakpoint::BREAKPOINTS;
use crate::cel::{Value, Values};
use crate::context::key;
use crate::expand::env::{Env, Slots, with_md};
use crate::expand::params::{boxed, key_of};
use crate::expand::{Expander, origins};
use crate::expanded::{self, Container, Content, Leaf};
use crate::resolve::Component;
use crate::tree::{DeclaredType, Headings, Node, Param, ParamValue, Signature, SlotDeclaration};
use crate::validate::scope::Scope;
use crate::validate::universal::Universal;
use std::collections::HashMap;
use std::rc::Rc;

impl<'r> Expander<'r> {
    /// A `compmd` component (spec §7.2, §9.2): its body in its own document's scope with the invocation's params,
    /// in a box styled by the fence's params under the caller's; its body alone when it is `display=contents`; inline,
    /// its body's one paragraph.
    pub(super) fn compmd(
        &mut self,
        invocation: &Invocation<'r>,
        component: &'r Component,
        body: &'r [Node],
        given: &[&'r Param],
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        let signature = component.signature.as_ref();
        let closes_cycle = self
            .resolved
            .closes_cycle(env.document, invocation.node.position());
        let body_env = (!closes_cycle)
            .then(|| self.body_env(invocation, component, given, env))
            .flatten();
        let Some(body_env) = body_env else {
            return self.fallback(invocation, env, out);
        };
        let heading = (self.last_heading, self.heading_kept);
        let made = self.expanding(env, invocation.at(), |this| {
            this.compmd_body(invocation, component, body, given, env, &body_env)
        });
        // The body's headings are its own: the caller's next invocation is still under the caller's heading.
        (self.last_heading, self.heading_kept) = heading;
        match made.flatten() {
            Some(nodes) => {
                out.extend(nodes);
                self.after(invocation, signature, env, out);
            }
            None => self.fallback(invocation, env, out),
        }
    }

    /// What a `compmd` component makes of its body: its box holding it, it alone when the component is
    /// `display=contents`, inline the content of its one paragraph; `None` inline when it isn't one paragraph.
    fn compmd_body(
        &mut self,
        invocation: &Invocation<'r>,
        component: &'r Component,
        body: &'r [Node],
        given: &[&'r Param],
        env: &Env<'r>,
        body_env: &Env<'r>,
    ) -> Option<Vec<expanded::Node>> {
        let expanded = self.list(body, body_env);
        if let Kind::Inline(_) = invocation.kind {
            let Ok([paragraph]) = <[expanded::Node; 1]>::try_from(expanded) else {
                return None;
            };
            if !matches!(paragraph.content, Content::Paragraph(_)) {
                return None;
            }
            let children = paragraph.content.children().to_vec();
            return Some(self.inline_box(invocation, component, given, env, body_env, children));
        }
        if component.is_contents() {
            return Some(expanded);
        }
        let params = self.box_params(component, given, env, body_env);
        let content = Content::ComponentContainer(Container {
            name: "box".into(),
            params,
            children: expanded,
            instances: None,
            slots: Vec::new(),
        });
        Some(vec![self.made(content, origins::of(invocation.node, env))])
    }

    /// An inline invocation's box (spec §7.2): its fence's params under the caller's, but `box`'s layout params, as a
    /// `componentInline` `box` whose `fallback` is the body's paragraph, what a renderer without it shows. Without
    /// any params it is the paragraph's content alone, as before.
    fn inline_box(
        &mut self,
        invocation: &Invocation<'r>,
        component: &'r Component,
        given: &[&'r Param],
        env: &Env<'r>,
        body_env: &Env<'r>,
        children: Vec<expanded::Node>,
    ) -> Vec<expanded::Node> {
        let params: Vec<expanded::Param> = self
            .box_params(component, given, env, body_env)
            .into_iter()
            // The common params and the attributes; `box`'s own (`display`, `gap`, …) lay out blocks it doesn't hold.
            .filter(|param| Universal::from_name(&param.key).is_some())
            .collect();
        if params.is_empty() {
            return children;
        }
        let content = Content::ComponentInline(Leaf {
            name: "box".into(),
            params,
            fallback: children,
        });
        vec![self.made(content, origins::of(invocation.node, env))]
    }

    /// A `compmd` component's box params: its fence's under the caller's common params.
    fn box_params(
        &mut self,
        component: &'r Component,
        given: &[&'r Param],
        env: &Env<'r>,
        body_env: &Env<'r>,
    ) -> Vec<expanded::Param> {
        let signature = component.signature.as_ref();
        let fence = self.fence(component, body_env);
        let common: Vec<&'r Param> = given
            .iter()
            .copied()
            .filter(|param| {
                signature.is_none_or(|signature| {
                    signature
                        .params
                        .iter()
                        .all(|declared| declared.name != key_of(param))
                })
            })
            .collect();
        let caller = self.evaluated(&common, None, env);
        boxed(fence, caller)
    }

    /// The expression `written` at the caller's breakpoint `index`, when the caller's params have breakpoint values:
    /// one of them used in it makes this param take one at that breakpoint too (spec §9.3), `columns` in
    /// `grid-template-columns=(…columns…)` passed to a `grid`. `None` for a param that isn't an expression.
    fn at_callers_breakpoint(
        &mut self,
        written: Option<&'r Param>,
        declared: &DeclaredType,
        env: &Env<'r>,
        index: usize,
    ) -> Option<Value> {
        let param = written.filter(|param| matches!(param.value, ParamValue::Expr(_)))?;
        let values = env.breakpoints.as_ref()?.get(index)?.clone();
        let at = Env {
            values,
            ..env.clone()
        };
        self.param_value(param, Some(declared), &at)
    }

    /// What a `compmd` component's body sees: its document's `md`, the invocation's params or their defaults, by
    /// breakpoint where they have breakpoint values, and its slots. `None` when a required param has no value, so
    /// the component renders its fallback.
    fn body_env(
        &mut self,
        invocation: &Invocation<'r>,
        component: &'r Component,
        given: &[&'r Param],
        env: &Env<'r>,
    ) -> Option<Env<'r>> {
        let document = component.document;
        let base = Env::document(self, document);
        let signature = component.signature.as_ref();
        let mut values = base.values.clone();
        let mut at_breakpoints: [Values; 3] = [values.clone(), values.clone(), values.clone()];
        let mut responsive = false;
        let mut templates = Vec::new();
        let mut bases = HashMap::new();
        for declaration in signature
            .map(|signature| signature.params.as_slice())
            .unwrap_or_default()
        {
            let declared = &declaration.declared;
            let mine: Vec<&'r Param> = given
                .iter()
                .copied()
                .filter(|param| key_of(param) == declaration.name)
                .collect();
            let written = mine.iter().find(|param| param.breakpoint.is_none());
            let value = match (declared, written) {
                (
                    DeclaredType::Input { .. },
                    Some(Param {
                        value: ParamValue::Bare(input),
                        ..
                    }),
                ) => Some(env.input(input)),
                (_, Some(param)) => self.param_value(param, Some(declared), env),
                (_, None) => None,
            };
            let value = match (value, &declaration.default) {
                (Some(value), _) => value,
                (None, Some(default)) => self.default(default, declared, &base)?,
                (None, None) if declaration.optional => Value::Null,
                (None, None) => return None,
            };
            let variable = declaration.name.replace('-', "_");
            if let Some(param) = written {
                bases.insert(variable.clone(), env.base_of(&param.value));
            }
            let mut current = value.clone();
            for (index, (values, breakpoint)) in
                at_breakpoints.iter_mut().zip(BREAKPOINTS).enumerate()
            {
                let at = mine
                    .iter()
                    .find(|param| param.breakpoint.as_deref() == Some(breakpoint))
                    .and_then(|param| self.param_value(param, Some(declared), env))
                    .or_else(|| self.at_callers_breakpoint(written.copied(), declared, env, index));
                if let Some(at) = at {
                    responsive |= at.to_json() != value.to_json();
                    current = at;
                }
                *values = values.clone().with(variable.clone(), current.clone());
            }
            values = values.with(variable, value);
        }
        if let Some(signature) = signature {
            for slot in signature.slots.iter().flatten() {
                if let SlotDeclaration::Template { name, .. } = slot
                    && let Some(template) = self.template(signature, name, given, env)
                {
                    templates.push(template);
                }
            }
        }
        let given_slots = given_slots(invocation, signature);
        values = with_md(&values, "slots", given_slots.clone());
        for at in &mut at_breakpoints {
            *at = with_md(at, "slots", given_slots.clone());
        }
        let relative = signature.is_none_or(|signature| signature.headings == Headings::Relative);
        let shift = relative.then(|| self.heading_depth());
        Some(Env {
            document,
            scope: Scope::definition(&self.resolved.document(document).settings, signature),
            values,
            slots: Some(Rc::new(Slots {
                default: invocation.default_content(signature),
                named: invocation.named_slots(),
                fallback: invocation.fallback_text(),
                env: env.clone(),
                templates,
            })),
            breakpoints: responsive.then(|| Rc::new(at_breakpoints)),
            shift,
            origin: origins::of(invocation.node, env),
            bases: Rc::new(bases),
        })
    }
}

/// `md.slots` (spec §7.2): for the default slot and each slot `signature` declares, by its name with `-` written
/// `_`, whether the caller gave it content.
fn given_slots(invocation: &Invocation<'_>, signature: Option<&Signature>) -> Value {
    let default = invocation
        .default_content(signature)
        .iter()
        .any(|nodes| !nodes.is_empty());
    let declared = signature
        .and_then(|signature| signature.slots.as_ref())
        .into_iter()
        .flatten()
        .filter(|slot| slot.name() != "default")
        .map(|slot| {
            let name = slot.name();
            (
                key(&name.replace('-', "_")),
                Value::Bool(!invocation.slot(name).is_empty()),
            )
        });
    Value::Map(Rc::new(
        std::iter::once((key("default"), Value::Bool(default)))
            .chain(declared)
            .collect(),
    ))
}
