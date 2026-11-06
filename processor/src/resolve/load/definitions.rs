//! A document's definitions and signature fences (spec §7): the components it defines, F002 for a signature fence with
//! nothing to attach to, F013 and F009 for their params, and F037 for a declaration whose type isn't a type name.

use super::{Loader, location};
use crate::diagnostic::{Code, Fix, Location};
use crate::nearest::nearest;
use crate::parse::CODE;
use crate::reserved;
use crate::resolve::names::{Binding, Builder, Level};
use crate::resolve::{Body, Component, DocumentId};
use crate::tree::{
    Declaration, DeclaredType, Definition, DefinitionBody, Node, Signature, SignatureFence,
    SlotDeclaration, TypeName,
};
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
            body,
            fence: definition.params,
            position: definition.position,
        });
        names.declare(
            &definition.name,
            Binding::Component(component, Level::Local),
        );
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
            self.components[component.0]
                .signature
                .get_or_insert(fence.signature.unwrap_or_default());
        } else if self.document(id).provided {
            if self.admit(id, names, &fence.name, at) {
                let component = self.add(Component {
                    name: fence.name.clone(),
                    document: id,
                    signature: fence.signature,
                    body: Body::Processor,
                    fence: Vec::new(),
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
    /// expressions can't tell apart (spec §5.1), and, for a `compmd` definition, a param name expressions can't use (a
    /// `name` param binds a name and is never read).
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
            self.check_type(id, param);
        }
        self.check_code(id, signature, at);
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
}
