//! Invocations (spec §4, §7, §9): a `compmd` component expands to its box holding its body, or to its body alone
//! when it is `display=contents`; `box`, the input components and host-language components become nodes with their
//! params evaluated and their slots expanded; `if`, `each` and `value` produce their content. A component that
//! can't render renders its fallback (spec §4.5).

use super::env::{Env, Slots, Template};
use super::params::{BREAKPOINTS, boxed, key_of};
use super::{Expander, origins};
use crate::cel::{Value, Values};
use crate::diagnostic::Location;
use crate::expanded::{self, Container, Content, Instance, Leaf, json};
use crate::reserved::Keyword;
use crate::resolve::{Binding, Body, Component, ComponentId};
use crate::tree::{
    self, DeclaredType, DefaultValue, Headings, Node, Param, ParamValue, Signature, SlotDeclaration,
};
use crate::validate::scope::Scope;
use std::rc::Rc;

/// How an invocation is written.
#[derive(Clone, Copy)]
pub(super) enum Kind<'r> {
    Container(&'r tree::Container),
    Leaf(&'r tree::Leaf),
    Inline(&'r tree::Leaf),
}

/// An invocation as written.
#[derive(Clone, Copy)]
pub(super) struct Invocation<'r> {
    pub(super) node: &'r Node,
    pub(super) name: &'r str,
    pub(super) params: &'r [Param],
    pub(super) kind: Kind<'r>,
}

impl<'r> Invocation<'r> {
    pub(super) fn container(node: &'r Node, container: &'r tree::Container) -> Self {
        Self {
            node,
            name: &container.name,
            params: &container.params,
            kind: Kind::Container(container),
        }
    }

    pub(super) fn leaf(node: &'r Node, leaf: &'r tree::Leaf) -> Self {
        Self {
            node,
            name: &leaf.name,
            params: &leaf.params,
            kind: Kind::Leaf(leaf),
        }
    }

    pub(super) fn inline(node: &'r Node, leaf: &'r tree::Leaf) -> Self {
        Self {
            node,
            name: &leaf.name,
            params: &leaf.params,
            kind: Kind::Inline(leaf),
        }
    }

    /// Where it starts.
    pub(super) fn at(&self) -> Location {
        self.node
            .position()
            .map_or(Location::new(1, 1), |position| position.start.into())
    }

    /// Its default slot.
    pub(super) fn default_slot(&self) -> &'r [Node] {
        match self.kind {
            Kind::Container(container) => &container.children,
            Kind::Leaf(_) | Kind::Inline(_) => &[],
        }
    }

    /// Its named slots.
    pub(super) fn named_slots(&self) -> &'r [tree::Slot] {
        match self.kind {
            Kind::Container(container) => &container.slots,
            Kind::Leaf(_) | Kind::Inline(_) => &[],
        }
    }

    /// The content of the named slot `name`.
    pub(super) fn slot(&self, name: &str) -> &'r [Node] {
        self.named_slots()
            .iter()
            .find(|slot| slot.name == name)
            .map_or(&[], |slot| slot.children.as_slice())
    }
}

impl<'r> Expander<'r> {
    /// An invocation expanded.
    pub(super) fn component(
        &mut self,
        invocation: Invocation<'r>,
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        let resolved = self.resolved;
        let Some(Binding::Component(id, _)) =
            resolved.document(env.document).names.get(invocation.name)
        else {
            return self.fallback(&invocation, env, out);
        };
        let component = resolved.component(*id);
        let given: Vec<&'r Param> = invocation
            .params
            .iter()
            .filter(|param| !self.validated.refuses_param(env.document, param))
            .collect();
        let signature = component.signature.as_ref();
        let renders = !self
            .validated
            .falls_back(env.document, invocation.node.position());
        match (&component.body, component.keyword()) {
            (_, Some(Keyword::Each)) if !renders => {
                self.fallback_template(&invocation, env, out);
            }
            _ if !renders => self.fallback(&invocation, env, out),
            (Body::Compmd(body), _) => {
                self.compmd(&invocation, *id, component, body, &given, env, out);
            }
            (_, Some(Keyword::If)) => self.r#if(&invocation, &given, env, out),
            (_, Some(Keyword::Each)) => self.each(&invocation, signature, &given, env, out),
            (_, Some(Keyword::Value)) => self.value(&invocation, &given, env, out),
            _ => self.rendered(&invocation, component, &given, env, out),
        }
    }

    /// After a component that rendered: a body given to a component that takes none follows it as normal content
    /// (F018).
    pub(super) fn after(
        &mut self,
        invocation: &Invocation<'r>,
        signature: Option<&Signature>,
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        if signature.is_some_and(|signature| signature.slots.is_none()) {
            self.stray_body(invocation, env, out);
        }
    }

    /// A container's slots as normal content: its default slot, then each named slot.
    fn stray_body(
        &mut self,
        invocation: &Invocation<'r>,
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        out.extend(self.list(invocation.default_slot(), env));
        for slot in invocation.named_slots() {
            out.extend(self.list(&slot.children, env));
        }
    }

    /// What a component that can't render shows (spec §4.5): a container its default slot then each named slot, a
    /// leaf its fallback as a paragraph, an inline component its fallback.
    pub(super) fn fallback(
        &mut self,
        invocation: &Invocation<'r>,
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        match invocation.kind {
            Kind::Container(_) => self.stray_body(invocation, env, out),
            Kind::Leaf(leaf) => {
                let children = self.list(&leaf.fallback, env);
                let paragraph = Content::Paragraph(tree::Parent {
                    children,
                    position: None,
                });
                let made = self.made(paragraph, origins::of(invocation.node, env));
                out.push(made);
            }
            Kind::Inline(leaf) => out.extend(self.inline(&leaf.fallback, env)),
        }
    }

    /// A node the processor renders: `box`, an input component, a host-language component (spec §12.3). Its params
    /// are its fence's under the caller's; its slots are expanded, a template slot once per element.
    fn rendered(
        &mut self,
        invocation: &Invocation<'r>,
        component: &'r Component,
        given: &[&'r Param],
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        let fence_env = Env::document(self, component.document);
        let fence = self.fence(component, &fence_env);
        let signature = component.signature.as_ref();
        let caller = self.evaluated(given, signature, env);
        let params = boxed(fence, caller);
        let name = invocation.name.to_string();
        let content = match invocation.kind {
            Kind::Container(_) => {
                let (children, instances) = self.rendered_slot(
                    invocation,
                    signature,
                    "default",
                    invocation.default_slot(),
                    given,
                    env,
                );
                let mut slots = Vec::new();
                for slot in invocation.named_slots() {
                    let (children, instances) = self.rendered_slot(
                        invocation,
                        signature,
                        &slot.name,
                        &slot.children,
                        given,
                        env,
                    );
                    slots.push(expanded::Slot {
                        name: slot.name.clone(),
                        children,
                        instances,
                        origin: env
                            .origin
                            .or_else(|| origins::range(&Node::ComponentSlot(slot.clone()))),
                    });
                }
                Content::ComponentContainer(Container {
                    name,
                    params,
                    children,
                    instances,
                    slots,
                })
            }
            Kind::Leaf(leaf) => Content::ComponentLeaf(Leaf {
                name,
                params,
                fallback: self.inline(&leaf.fallback, env),
            }),
            Kind::Inline(leaf) => Content::ComponentInline(Leaf {
                name,
                params,
                fallback: self.inline(&leaf.fallback, env),
            }),
        };
        let made = self.made(content, origins::of(invocation.node, env));
        out.push(made);
        self.after(invocation, signature, env, out);
    }

    /// A slot of a rendered node: its content expanded, or for a template slot, once per element.
    fn rendered_slot(
        &mut self,
        invocation: &Invocation<'r>,
        signature: Option<&Signature>,
        name: &str,
        content: &'r [Node],
        given: &[&'r Param],
        env: &Env<'r>,
    ) -> (Vec<expanded::Node>, Option<Vec<Instance>>) {
        let Some(template) =
            signature.and_then(|signature| self.template(signature, name, given, env))
        else {
            return (self.list(content, env), None);
        };
        let instances = self.expanding(env, invocation.at(), |this| {
            let mut instances = Vec::new();
            for index in 0..template.items.len() {
                if this.over_limit() {
                    break;
                }
                let inner = env.iteration(&template.name, &template.items, index);
                instances.push(Instance {
                    value: json(&template.items[index]),
                    children: this.list(content, &inner),
                });
            }
            instances
        });
        match instances {
            Some(instances) => (Vec::new(), Some(instances)),
            None => (self.list(content, env), None),
        }
    }

    /// The template slot `name` of an invocation, with its elements and the name it binds them to (spec §7.5).
    fn template(
        &mut self,
        signature: &Signature,
        name: &str,
        given: &[&'r Param],
        env: &Env<'r>,
    ) -> Option<Template> {
        let (each, binding) = signature
            .slots
            .iter()
            .flatten()
            .find_map(|slot| match slot {
                SlotDeclaration::Template {
                    name: slot,
                    each,
                    binding,
                } if slot == name => Some((each, binding)),
                _ => None,
            })?;
        let param = |key: &str| given.iter().find(|param| key_of(param) == key).copied();
        let items = match param(each).and_then(|param| self.param_value(param, None, env)) {
            Some(Value::List(items)) => items,
            _ => Rc::default(),
        };
        let bound = match param(binding).map(|param| &param.value) {
            Some(ParamValue::Bare(bound)) => bound.clone(),
            _ => String::new(),
        };
        Some(Template {
            slot: name.to_string(),
            items,
            name: bound,
        })
    }

    /// A definition fence's params, evaluated in `env`: what validating refused, and `display=contents`, left out.
    fn fence(&mut self, component: &'r Component, env: &Env<'r>) -> Vec<super::params::Out> {
        let fence: Vec<&'r Param> = component
            .fence
            .iter()
            .filter(|param| {
                !is_contents(param) && !self.validated.refuses_param(component.document, param)
            })
            .collect();
        self.evaluated(&fence, None, env)
    }

    /// A `compmd` component (spec §7.2, §9.2): its body in its own document's scope with the invocation's params,
    /// in a box styled by the fence's params under the caller's; its body alone when it is `display=contents`; inline,
    /// its body's one paragraph.
    #[allow(
        clippy::too_many_arguments,
        reason = "the invocation, its component and what it is given, as the other invocations take them"
    )]
    fn compmd(
        &mut self,
        invocation: &Invocation<'r>,
        id: ComponentId,
        component: &'r Component,
        body: &'r [Node],
        given: &[&'r Param],
        env: &Env<'r>,
        out: &mut Vec<expanded::Node>,
    ) {
        let signature = component.signature.as_ref();
        let body_env = (!self.invoked.contains(&id))
            .then(|| self.body_env(invocation, component, given, env))
            .flatten();
        let Some(body_env) = body_env else {
            return self.fallback(invocation, env, out);
        };
        self.invoked.push(id);
        let heading = self.last_heading;
        let made = self.expanding(env, invocation.at(), |this| {
            this.compmd_body(invocation, component, body, given, env, &body_env)
        });
        // The body's headings are its own: the caller's next invocation is still under the caller's heading.
        self.last_heading = heading;
        self.invoked.pop();
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
        if component.fence.iter().any(is_contents) {
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
        Some(Env {
            document,
            scope: Scope::definition(&self.resolved.document(document).settings, signature),
            values,
            slots: Some(Rc::new(Slots {
                default: invocation.default_slot(),
                named: invocation.named_slots(),
                env: env.clone(),
                templates,
            })),
            breakpoints: responsive.then(|| Rc::new(at_breakpoints)),
            shift: relative.then_some(self.last_heading),
            origin: origins::of(invocation.node, env),
        })
    }

    /// A declared param's default, in the definition's own document.
    fn default(
        &mut self,
        default: &DefaultValue,
        declared: &DeclaredType,
        env: &Env<'r>,
    ) -> Option<Value> {
        match default {
            DefaultValue::Yaml(yaml) => Some(Value::from_json(yaml)),
            DefaultValue::Param(value) => {
                let param = Param {
                    key: None,
                    breakpoint: None,
                    value: value.clone(),
                    position: None,
                };
                self.param_value(&param, Some(declared), env)
            }
        }
    }
}

/// A fence's bare `display=contents`: the component has no box (spec §9.2).
fn is_contents(param: &Param) -> bool {
    param.key.as_deref() == Some("display")
        && param.breakpoint.is_none()
        && param.value == ParamValue::Bare("contents".into())
}
