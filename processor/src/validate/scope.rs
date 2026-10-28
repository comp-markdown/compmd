//! What a piece of a document sees (spec §8.1, §8.2): `md.doc` and `md.data` of the document it is written in,
//! `md.input` in document bodies, a definition's params and slots in its body, and `as` names and `md.loop` in a
//! template. Interpolation paths are checked against it (F014), and expressions against its types.

use crate::cel::{self, Type};
use crate::resolve::{DocumentId, Resolved};
use crate::settings::{Input, Settings};
use crate::tree::{DeclaredType, Signature, SlotDeclaration, TypeName};
use serde_json::Value;
use std::collections::BTreeMap;

/// The fields of `md.loop` (spec §8.3).
const LOOP_FIELDS: [(&str, Type); 5] = [
    ("index", Type::Int),
    ("number", Type::Int),
    ("count", Type::Int),
    ("first", Type::Bool),
    ("last", Type::Bool),
];

/// The names a piece of a document sees.
#[derive(Clone, Debug)]
pub(crate) struct Scope<'r> {
    /// The settings of the document it is written in.
    settings: &'r Settings,
    /// The document's inputs: in its body, not in definitions.
    inputs: Option<&'r [Input]>,
    /// The signature of the `compmd` definition whose body it is in.
    definition: Option<&'r Signature>,
    /// Names bound by `each` and template slots, with the types of their values.
    bindings: Vec<(String, Type)>,
    /// Whether `md.loop` is bound.
    in_loop: bool,
}

impl<'r> Scope<'r> {
    /// What the body of the document `id` sees: the processed document sees its inputs, any other document is used
    /// only through its definitions, so it sees what they do.
    pub(crate) fn body(resolved: &'r Resolved, id: DocumentId) -> Self {
        let settings = &resolved.document(id).settings;
        if id == resolved.main {
            Self::document(settings)
        } else {
            Self::definition(settings, None)
        }
    }

    /// The body of the processed document.
    fn document(settings: &'r Settings) -> Self {
        Self {
            settings,
            inputs: Some(&settings.inputs),
            definition: None,
            bindings: Vec::new(),
            in_loop: false,
        }
    }

    /// The body or the fence of a `compmd` definition with `signature`, in the document with `settings`.
    pub(crate) fn definition(settings: &'r Settings, signature: Option<&'r Signature>) -> Self {
        Self {
            settings,
            inputs: None,
            definition: signature,
            bindings: Vec::new(),
            in_loop: false,
        }
    }

    /// This scope inside a template: `name` bound to each element, of type `element`, and `md.loop`.
    pub(crate) fn template(&self, name: &str, element: Type) -> Self {
        let mut inner = self.clone();
        inner.bindings.retain(|(known, _)| known != name);
        inner.bindings.push((name.to_string(), element));
        inner.in_loop = true;
        inner
    }

    /// The declared input `name`, where inputs are seen.
    pub(crate) fn input(&self, name: &str) -> Option<&'r Input> {
        self.inputs?
            .iter()
            .find(|input| input.declaration.name == name)
    }

    /// Whether inputs are seen here.
    pub(crate) fn sees_inputs(&self) -> bool {
        self.inputs.is_some()
    }

    /// The definition's params that expressions read: all but `name` params, which bind a name. A `-` in a param's
    /// name is read as `_` (spec §5.1).
    fn params(&self) -> impl Iterator<Item = (String, &'r DeclaredType)> {
        self.definition
            .into_iter()
            .flat_map(|signature| &signature.params)
            .filter(|param| param.declared != DeclaredType::Name(TypeName::Name))
            .map(|param| (param.name.replace('-', "_"), &param.declared))
    }

    /// Whether the interpolation path `path` names something here (F014 otherwise).
    pub(crate) fn knows(&self, path: &str) -> bool {
        let segments: Vec<&str> = path.split('.').collect();
        match segments[..] {
            ["md"] | ["md", "version" | "lang" | "tz" | "data"] => true,
            ["md", "doc", ref keys @ ..] => has_keys(&self.settings.doc, keys),
            ["md", "data", name, ..] => self.settings.data.iter().any(|data| data.name == name),
            ["md", "input"] => self.sees_inputs(),
            ["md", "input", name, ..] => self.input(name).is_some(),
            ["md", "loop"] => self.in_loop,
            ["md", "loop", field] => {
                self.in_loop && LOOP_FIELDS.iter().any(|(known, _)| *known == field)
            }
            ["md", "slot"] => self.has_slot("default"),
            ["md", "slot", slot] => self.has_slot(slot),
            ["md", ..] | [] => false,
            [name, ..] => {
                self.bindings.iter().any(|(bound, _)| bound == name)
                    || self.params().any(|(param, _)| param == name)
            }
        }
    }

    fn has_slot(&self, name: &str) -> bool {
        self.definition
            .and_then(|signature| signature.slots.as_ref())
            .is_some_and(|slots| {
                slots.iter().any(|slot| match slot {
                    SlotDeclaration::Name(slot) | SlotDeclaration::Template { name: slot, .. } => {
                        slot == name
                    }
                })
            })
    }

    /// The types of the variables expressions see here.
    pub(crate) fn cel(&self) -> cel::Scope {
        let mut md = BTreeMap::from([
            (
                "doc".to_string(),
                Type::of_json(&Value::Object(self.settings.doc.clone())),
            ),
            ("data".to_string(), self.data()),
            ("version".to_string(), Type::String),
            ("lang".to_string(), Type::String),
            ("tz".to_string(), Type::String),
        ]);
        if let Some(inputs) = self.inputs {
            let fields = inputs
                .iter()
                .map(|input| {
                    (
                        input.declaration.name.clone(),
                        Type::declared(&input.declaration.declared),
                    )
                })
                .collect();
            md.insert("input".into(), Type::record(fields));
        }
        if self.in_loop {
            let fields = LOOP_FIELDS
                .iter()
                .map(|(name, ty)| ((*name).to_string(), ty.clone()))
                .collect();
            md.insert("loop".into(), Type::record(fields));
        }
        let mut scope = cel::Scope::default().with("md", Type::record(md));
        for (name, declared) in self.params() {
            scope = scope.with(name, Type::declared(declared));
        }
        for (name, ty) in &self.bindings {
            scope = scope.with(name.clone(), ty.clone());
        }
        scope
    }

    /// `md.data`: each data import, a list of records of its schema, or `dyn` without one.
    fn data(&self) -> Type {
        let fields = self
            .settings
            .data
            .iter()
            .map(|data| {
                let ty = data.schema.as_ref().map_or(Type::Dyn, |schema| {
                    let record = schema
                        .iter()
                        .map(|field| {
                            let declaration = &field.declaration;
                            (
                                declaration.name.clone(),
                                Type::declared(&declaration.declared),
                            )
                        })
                        .collect();
                    Type::list(Type::record(record))
                });
                (data.name.clone(), ty)
            })
            .collect();
        Type::record(fields)
    }
}

/// Whether `keys` lead through maps of `doc` to a value.
fn has_keys(doc: &serde_json::Map<String, Value>, keys: &[&str]) -> bool {
    let Some((first, rest)) = keys.split_first() else {
        return true;
    };
    let mut value = doc.get(*first);
    for key in rest {
        value = value
            .and_then(Value::as_object)
            .and_then(|fields| fields.get(*key));
    }
    value.is_some()
}
