//! A `compmd` component's invocation (spec §7.2, §9.2): its body, expanded in its own document's scope with the
//! invocation's params, in a box styled by its fence's params under the caller's.

use super::{Invocation, Kind};
use crate::breakpoint::BREAKPOINTS;
use crate::cel::{Value, Values};
use crate::expand::env::{Env, Slots};
use crate::expand::params::{boxed, key_of};
use crate::expand::{Expander, origins};
use crate::expanded::{self, Container, Content};
use crate::resolve::Component;
use crate::tree::{DeclaredType, Headings, Node, Param, ParamValue, SlotDeclaration};
use crate::validate::scope::Scope;
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
            return match <[expanded::Node; 1]>::try_from(expanded) {
                Ok([paragraph]) if matches!(paragraph.content, Content::Paragraph(_)) => {
                    Some(paragraph.content.children().to_vec())
                }
                _ => None,
            };
        }
        if component.is_contents() {
            return Some(expanded);
        }
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
        let content = Content::ComponentContainer(Container {
            name: "box".into(),
            params: boxed(fence, caller),
            children: expanded,
            instances: None,
            slots: Vec::new(),
        });
        Some(vec![self.made(content, origins::of(invocation.node, env))])
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
            let mut current = value.clone();
            for (values, breakpoint) in at_breakpoints.iter_mut().zip(BREAKPOINTS) {
                let at = mine
                    .iter()
                    .find(|param| param.breakpoint.as_deref() == Some(breakpoint))
                    .and_then(|param| self.param_value(param, Some(declared), env));
                if let Some(at) = at {
                    current = at;
                    responsive = true;
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
        let relative = signature.is_none_or(|signature| signature.headings == Headings::Relative);
        let shift = relative.then(|| self.heading_depth());
        Some(Env {
            document,
            scope: Scope::definition(&self.resolved.document(document).settings, signature),
            values,
            slots: Some(Rc::new(Slots {
                default: invocation.default_content(signature),
                named: invocation.named_slots(),
                env: env.clone(),
                templates,
            })),
            breakpoints: responsive.then(|| Rc::new(at_breakpoints)),
            shift,
            origin: origins::of(invocation.node, env),
        })
    }
}
