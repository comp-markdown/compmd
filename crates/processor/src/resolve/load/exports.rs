//! A document's exports (spec §6.2): F013 for a keyword, F029 and W117 for host-language components, and W105 for a
//! local one without a signature fence.

use super::{Loader, location};
use crate::diagnostic::{Code, Location};
use crate::resolve::names::{Binding, Level, Names};
use crate::resolve::{Body, ComponentId, DocumentId, Language};
use crate::{reserved, settings};
use std::collections::{BTreeMap, HashSet};

impl Loader<'_, '_> {
    /// The document's exports (spec §6.2): F013 for a keyword, F029 and W117 for host-language components.
    pub(super) fn exports(&mut self, id: DocumentId, names: &Names) -> BTreeMap<String, Binding> {
        let Some(export) = self.document(id).settings.export.clone() else {
            return BTreeMap::new();
        };
        let mut exports = BTreeMap::new();
        match &export.names {
            settings::Names::All(at) => {
                let defined: Vec<ComponentId> = (0..self.components.len())
                    .map(ComponentId)
                    .filter(|component| {
                        let component = &self.components[component.0];
                        component.document == id && !matches!(component.body, Body::File { .. })
                    })
                    .collect();
                for component in defined {
                    let name = self.components[component.0].name.clone();
                    exports.insert(name, Binding::Component(component, Level::Local));
                    self.check_export(id, component, *at);
                }
            }
            settings::Names::Listed(entries) => {
                for entry in entries {
                    if let Some(why) = reserved::as_component(&entry.renamed) {
                        self.report(id, Code::F013, entry.at, why);
                        continue;
                    }
                    let Some(binding) = names.get(&entry.name).cloned() else {
                        continue;
                    };
                    if let Binding::Component(component, _) = binding {
                        self.check_export(id, component, entry.at);
                    }
                    exports.insert(entry.renamed.clone(), binding);
                }
            }
        }
        exports
    }

    /// An exported host-language component needs a signature fence (F029) and, unless plain JavaScript, a runtime
    /// (W117, at its definition, by the document that defines it).
    fn check_export(&mut self, id: DocumentId, component: ComponentId, at: Location) {
        let exported = &self.components[component.0];
        if !exported.body.is_host() {
            return;
        }
        if exported.signature.is_none() {
            let name = exported.name.clone();
            self.report(
                id,
                Code::F029,
                at,
                format!("`{name}` is exported, so it needs a signature fence: packs are used where it may not run"),
            );
        }
        let exported = &self.components[component.0];
        let unpinned = match &exported.body {
            Body::Host {
                language,
                runtime: None,
                ..
            } => Language::from_name(language).is_some_and(Language::has_runtimes),
            _ => false,
        };
        if unpinned && exported.document == id {
            let (name, at) = (exported.name.clone(), location(exported.position));
            self.report(
                id,
                Code::W117,
                at,
                format!("`{name}` is exported without `runtime=`, so it changes when the binding's default moves"),
            );
        }
    }

    /// W105: a local host-language component without a signature fence, whose params can't be typed: one defined in a
    /// fence, at the fence, or a component file the document imports, at its `names` entry (spec §7.1).
    pub(super) fn untyped(&mut self, id: DocumentId, exports: &BTreeMap<String, Binding>) {
        let exported: HashSet<ComponentId> = exports
            .values()
            .filter_map(|binding| match binding {
                Binding::Component(component, _) => Some(*component),
                _ => None,
            })
            .collect();
        let untyped: Vec<(String, Location)> = self
            .components
            .iter()
            .enumerate()
            .filter(|(index, component)| {
                component.document == id
                    && component.body.is_host()
                    && component.signature.is_none()
                    && !exported.contains(&ComponentId(*index))
            })
            .map(|(index, component)| {
                let at = self
                    .file_imports
                    .get(&ComponentId(index))
                    .copied()
                    .unwrap_or_else(|| location(component.position));
                (component.name.clone(), at)
            })
            .collect();
        for (name, at) in untyped {
            self.report(
                id,
                Code::W105,
                at,
                format!("`{name}` has no signature fence, so its params go through unchecked"),
            );
        }
    }
}
