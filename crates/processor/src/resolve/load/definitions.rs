//! A document's definitions and signature fences (spec §7): the components it defines, F002 for a signature fence with
//! nothing to attach to, F036 for a box styled by both fences, F013 and F009 for their params, and F037 for a
//! declaration whose type isn't a type name.

use super::{Loader, location};
use crate::diagnostic::{Code, Fix, Location};
use crate::nearest::nearest;
use crate::parse::CODE;
use crate::reserved;
use crate::resolve::names::{Binding, Builder, Level};
use crate::resolve::{Body, Component, ComponentId, DocumentId};
use crate::tree::{
    Declaration, DeclaredType, Definition, DefinitionBody, Node, Param, Print, Signature,
    SignatureFence, SlotDeclaration, TypeName,
};
use crate::validate::universal::Universal;
use std::collections::{HashMap, HashSet};

impl Loader<'_, '_> {
    /// The document's definitions and signature fences.
    pub(super) fn define(&mut self, id: DocumentId, names: &mut Builder) {
        let root = &self.document(id).root;
        let definitions: Vec<Definition> = root
            .children
            .iter()
            .filter_map(|node| match node {
                Node::ComponentDefinition(definition) => Some(definition.clone()),
                _ => None,
            })
            .collect();
        let fences: Vec<SignatureFence> = root
            .children
            .iter()
            .filter_map(|node| match node {
                Node::ComponentSignature(fence) => Some(fence.clone()),
                _ => None,
            })
            .collect();
        for definition in definitions {
            self.definition(id, definition, names);
        }
        for fence in fences {
            self.signature_fence(id, fence, names);
        }
    }

    fn definition(&mut self, id: DocumentId, definition: Definition, names: &mut Builder) {
        let at = location(definition.position);
        if !self.admit(id, names, &definition.name, at) {
            return;
        }
        let compmd = matches!(definition.body, DefinitionBody::Compmd { .. });
        if let Some(signature) = &definition.signature {
            self.check_params(id, signature, at, compmd);
        }
        let body = match definition.body {
            DefinitionBody::Compmd { children } => Body::Compmd(children),
            DefinitionBody::Host { source, runtime } => Body::Host {
                language: definition.language,
                runtime,
                source,
            },
        };
        let component = self.add(Component {
            name: definition.name.clone(),
            document: id,
            signature: definition.signature,
            signature_at: definition.position,
            body,
            fence: definition.params,
            position: definition.position,
        });
        names.declare(
            &definition.name,
            Binding::Component(component, Level::Local),
        );
    }

    /// A signature fence's param list styles its component's box (spec §7), unless the component's definition fence
    /// has one: then that one is kept, with F036 at the signature fence's first param.
    fn style_box(&mut self, id: DocumentId, component: ComponentId, params: Vec<Param>) {
        let Some(first) = params.first() else {
            return;
        };
        let at = location(first.position);
        let fence = &mut self.components[component.0].fence;
        if fence.is_empty() {
            *fence = params;
        } else {
            self.report(
                id,
                Code::F036,
                at,
                "the component's definition fence styles its box already, so this param list is ignored: style the box in one place",
            );
        }
    }

    /// A signature fence gives its signature to the document's host-language component of that name; in a pack the
    /// processor provides, a fence alone is a component the processor implements (spec §9).
    fn signature_fence(&mut self, id: DocumentId, fence: SignatureFence, names: &mut Builder) {
        let at = location(fence.position);
        if let Some(signature) = &fence.signature {
            self.check_params(id, signature, at, false);
        }
        let host = match names.declared(&fence.name) {
            Some(Binding::Component(component, _)) => Some(*component).filter(|component| {
                let component = &self.components[component.0];
                component.document == id && component.body.is_host()
            }),
            _ => None,
        };
        if let Some(component) = host {
            let host = &mut self.components[component.0];
            if host.signature.is_none() {
                host.signature = Some(fence.signature.unwrap_or_default());
                host.signature_at = fence.position;
            }
            self.style_box(id, component, fence.params);
        } else if self.document(id).provided {
            if self.admit(id, names, &fence.name, at) {
                let component = self.add(Component {
                    name: fence.name.clone(),
                    document: id,
                    signature: fence.signature,
                    signature_at: fence.position,
                    body: Body::Processor,
                    fence: fence.params,
                    position: fence.position,
                });
                names.declare(&fence.name, Binding::Component(component, Level::Local));
            }
        } else {
            self.report(
                id,
                Code::F002,
                at,
                format!(
                    "the signature fence for `{}` has nothing to attach to: no definition of `{0}` and no component file imported as `{0}`",
                    fence.name
                ),
            );
        }
    }

    /// F013 for a signature's params and slots: two whose names differ only in `-` and `_`, which paths and
    /// expressions can't tell apart (spec §5.1); a name the processor keeps for itself; and, for a `compmd` definition,
    /// a param name expressions can't use (a `name` param binds a name and is never read).
    fn check_params(&mut self, id: DocumentId, signature: &Signature, at: Location, compmd: bool) {
        let mut slots: HashMap<String, &str> = HashMap::new();
        for slot in signature.slots.iter().flatten() {
            let (SlotDeclaration::Name(name) | SlotDeclaration::Template { name, .. }) = slot;
            match slots.insert(name.replace('-', "_"), name) {
                Some(other) if other != name => self.report(
                    id,
                    Code::F013,
                    at,
                    format!("the slots `{other}` and `{name}` differ only in `-` and `_`, which `md.slot` paths read alike"),
                ),
                _ => {}
            }
        }
        let mut seen = HashSet::new();
        for param in &signature.params {
            if !seen.insert(param.name.replace('-', "_")) {
                self.report(
                    id,
                    Code::F013,
                    at,
                    format!("two params differ only in `-` and `_` from `{}`, which expressions read alike", param.name),
                );
            }
            let binds_a_name = param.declared == DeclaredType::Name(TypeName::Name);
            if let Some(why) =
                reserved::as_variable(&param.name).filter(|_| compmd && !binds_a_name)
            {
                self.report(id, Code::F013, at, why);
            }
            if let Some(why) = processors_name(&param.name, signature.renders.is_some()) {
                self.report(id, Code::F013, param.type_at.unwrap_or(at), why);
            }
            self.check_type(id, param);
        }
        self.check_code(id, signature, at);
        self.check_print(id, signature, compmd);
    }

    /// W122 for a `print` that is neither `drawn` nor `fallback`, or that a `compmd` definition writes (spec §7.3):
    /// the component prints as drawn.
    fn check_print(&mut self, id: DocumentId, signature: &Signature, compmd: bool) {
        let Some((at, written)) = &signature.print_written else {
            return;
        };
        let why = if compmd {
            "a `compmd` definition prints its Markdown as it renders, so `print` is for host-language components"
                .to_string()
        } else if Print::from_name(written).is_none() {
            format!(
                "`print` is `drawn` or `fallback`, not `{written}`, so the component prints as drawn"
            )
        } else {
            return;
        };
        self.report(id, Code::W122, *at, why);
    }

    /// F009 for a code renderer that declares `code` as anything but a required string: the block's text fills it
    /// (spec §7.3).
    fn check_code(&mut self, id: DocumentId, signature: &Signature, at: Location) {
        if signature.renders.is_none() {
            return;
        }
        let declared = signature.params.iter().find(|param| param.name == CODE);
        if let Some(declared) = declared.filter(|declared| {
            declared.declared != DeclaredType::Name(TypeName::String)
                || declared.optional
                || declared.default.is_some()
        }) {
            self.report(
                id,
                Code::F009,
                declared.type_at.unwrap_or(at),
                "a code renderer's `code` holds the block's text, so it is a required `string`, with no default",
            );
        }
    }

    /// F037 for the declarations of the document's frontmatter: its inputs, and its data imports' schemas.
    pub(super) fn check_frontmatter_types(&mut self, id: DocumentId) {
        let settings = &self.document(id).settings;
        let fields = settings
            .data
            .iter()
            .flat_map(|data| data.schema.iter().flatten())
            .map(|field| &field.declaration);
        let declarations: Vec<Declaration> = settings
            .inputs
            .iter()
            .map(|input| &input.declaration)
            .chain(fields)
            .cloned()
            .collect();
        for declaration in &declarations {
            self.check_type(id, declaration);
        }
    }

    /// F037 for a declaration whose type isn't a type name (spec §7.3), with the nearest one and the fix that writes
    /// it. The declaration is kept: its type takes any value.
    fn check_type(&mut self, id: DocumentId, declaration: &Declaration) {
        self.check_unquoted(id, declaration);
        let (Some(at), Some(written)) = (declaration.unknown_at, declaration.declared.unknown())
        else {
            return;
        };
        // `enum` is a type only inside `input<…>`.
        let near = nearest(
            written,
            TypeName::NAMES
                .iter()
                .copied()
                .filter(|name| *name != "enum"),
        );
        let hint = near.map_or_else(String::new, |near| format!("; did you mean `{near}`?"));
        let fix = near.map(|near| Fix::replacing(at, written, near));
        self.report_fixed(
            id,
            Code::F037,
            at,
            format!(
                "`{written}` isn't a type, so `{}` takes any value{hint}",
                declaration.name
            ),
            fix,
        );
    }

    /// F011 for a short form's default written unquoted with spaces: a bare value ends at its first white space
    /// (spec §3.1, §7.3), so the rest would be left out without a word. The fix quotes the whole default; the
    /// default is kept as far as its first white space.
    fn check_unquoted(&mut self, id: DocumentId, declaration: &Declaration) {
        let Some(unquoted) = &declaration.unquoted else {
            return;
        };
        let quoted = format!(
            "\"{}\"",
            unquoted.text.replace('\\', "\\\\").replace('"', "\\\"")
        );
        let message = format!(
            "`{}`'s default ends at its first space, so the rest isn't part of it: quote it, {quoted}",
            declaration.name
        );
        let fix = Fix::replacing(unquoted.at, &unquoted.text, quoted);
        self.report_fixed(id, Code::F011, unquoted.rest_at, message, Some(fix));
    }
}

/// Why a signature can't declare `name`, when the processor keeps it (spec §5.1): the common params, `class`, `id`,
/// `lang` and `dir` are every component's, applied to its box or its block, and a code renderer's block takes `alt`
/// and `render` for itself, so the component never gets them.
fn processors_name(name: &str, renders: bool) -> Option<String> {
    if let Some(universal) = Universal::from_name(name) {
        return Some(match universal {
            Universal::ObjectFit | Universal::ObjectPosition => format!(
                "`{name}` is a common param, which every component gets as `fit` (spec §7.4), so a signature can't declare it"
            ),
            _ if universal.is_common() => format!(
                "`{name}` is a common param, which styles the component's box, so the component never gets it and a signature can't declare it"
            ),
            _ => format!(
                "`{name}` is every component's own, for its box, so a signature can't declare it"
            ),
        });
    }
    match name {
        "alt" | "render" if renders => Some(format!(
            "`{name}` is a rendered block's own, read by the processor, so a code renderer can't declare it"
        )),
        _ => None,
    }
}
