//! What each component name of a document stands for, by the priority of spec §6.3: local definitions and listed
//! imports, then wildcard imports and `@compmd/core` at one level, then ambient packs.

use super::ComponentId;
use std::collections::BTreeMap;

/// Where a name comes from, highest priority first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    /// A definition fence of the document.
    Local,
    /// An import that lists the name.
    Listed,
    /// A wildcard (`"*"`) import.
    Wildcard,
    /// `@compmd/core`, at the wildcard imports' level.
    Core,
    /// An ambient pack the processor provides.
    Ambient,
}

/// What a name stands for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Binding {
    /// A component, from this level.
    Component(ComponentId, Level),
    /// A name several wildcard imports, or one and `@compmd/core`, provide: these sources. Using it is F023.
    Ambiguous(Vec<String>),
    /// A name from an import that failed: its uses render fallbacks, without F002.
    Dropped,
}

/// The component names of a document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Names {
    bindings: BTreeMap<String, Binding>,
    /// Whether a wildcard import failed, so that a name may be one of its unknown names.
    incomplete: bool,
}

impl Names {
    /// What `name` stands for; `None` when nothing provides it (F002).
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Binding> {
        const DROPPED: &Binding = &Binding::Dropped;
        self.bindings
            .get(name)
            .or_else(|| self.incomplete.then_some(DROPPED))
    }

    /// Every name, for suggestions.
    pub fn all(&self) -> impl Iterator<Item = &str> {
        self.bindings.keys().map(String::as_str)
    }

    /// Every name that stands for a component, with it.
    pub(crate) fn components(&self) -> impl Iterator<Item = (&str, ComponentId)> {
        self.bindings
            .iter()
            .filter_map(|(name, binding)| match binding {
                Binding::Component(id, _) => Some((name.as_str(), *id)),
                Binding::Ambiguous(_) | Binding::Dropped => None,
            })
    }
}

/// Collects a document's names level by level.
#[derive(Debug, Default)]
pub(crate) struct Builder {
    /// Local definitions and listed imports.
    declared: BTreeMap<String, Binding>,
    /// Wildcard imports and `@compmd/core`: each name's components, with the sources providing them.
    shared: BTreeMap<String, Vec<(ComponentId, String, Level)>>,
    /// Ambient packs; the first to provide a name wins.
    ambient: BTreeMap<String, ComponentId>,
    incomplete: bool,
}

impl Builder {
    /// Whether a local definition or a listed import already declared `name`.
    pub(crate) fn is_declared(&self, name: &str) -> bool {
        self.declared.contains_key(name)
    }

    /// What `name` stands for at the first two levels.
    pub(crate) fn declared(&self, name: &str) -> Option<&Binding> {
        self.declared.get(name)
    }

    /// Declares `name` at the first two levels; the caller has checked it isn't declared yet (F013).
    pub(crate) fn declare(&mut self, name: &str, binding: Binding) {
        self.declared.insert(name.to_string(), binding);
    }

    /// Adds `name` from a wildcard import or `@compmd/core`, named `source`.
    pub(crate) fn share(&mut self, name: &str, id: ComponentId, source: &str, level: Level) {
        let providers = self.shared.entry(name.to_string()).or_default();
        if !providers.iter().any(|(known, ..)| *known == id) {
            providers.push((id, source.to_string(), level));
        }
    }

    /// Adds `name` from an ambient pack.
    pub(crate) fn ambient(&mut self, name: &str, id: ComponentId) {
        self.ambient.entry(name.to_string()).or_insert(id);
    }

    /// Notes that a wildcard import failed.
    pub(crate) fn incomplete(&mut self) {
        self.incomplete = true;
    }

    /// The names, each with what the highest level gives it.
    pub(crate) fn build(self) -> Names {
        let mut bindings = self.declared;
        for (name, providers) in self.shared {
            bindings
                .entry(name)
                .or_insert_with(|| match &providers[..] {
                    [(id, _, level)] => Binding::Component(*id, *level),
                    _ => Binding::Ambiguous(
                        providers.into_iter().map(|(_, source, _)| source).collect(),
                    ),
                });
        }
        for (name, id) in self.ambient {
            bindings
                .entry(name)
                .or_insert(Binding::Component(id, Level::Ambient));
        }
        Names {
            bindings,
            incomplete: self.incomplete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn higher_levels_win_and_one_level_can_be_ambiguous() {
        let mut names = Builder::default();
        names.declare("card", Binding::Component(ComponentId(0), Level::Local));
        names.share("card", ComponentId(1), "./ui.comp.md", Level::Wildcard);
        names.share("chip", ComponentId(2), "./one.comp.md", Level::Wildcard);
        names.share("chip", ComponentId(3), "./two.comp.md", Level::Wildcard);
        names.share("row", ComponentId(4), "@compmd/core", Level::Core);
        names.share(
            "row",
            ComponentId(4),
            "./re-export.comp.md",
            Level::Wildcard,
        );
        names.ambient("row", ComponentId(5));
        names.ambient("list-view", ComponentId(6));
        names.ambient("list-view", ComponentId(7));
        assert!(names.is_declared("card"));
        let names = names.build();
        assert_eq!(
            names.get("card"),
            Some(&Binding::Component(ComponentId(0), Level::Local))
        );
        assert_eq!(
            names.get("chip"),
            Some(&Binding::Ambiguous(vec![
                "./one.comp.md".into(),
                "./two.comp.md".into()
            ]))
        );
        assert_eq!(
            names.get("row"),
            Some(&Binding::Component(ComponentId(4), Level::Core))
        );
        assert_eq!(
            names.get("list-view"),
            Some(&Binding::Component(ComponentId(6), Level::Ambient))
        );
        assert_eq!(names.get("nothing"), None);
        assert_eq!(names.all().count(), 4);
    }

    #[test]
    fn a_failed_wildcard_import_may_hold_any_name() {
        let mut names = Builder::default();
        names.incomplete();
        assert_eq!(names.build().get("anything"), Some(&Binding::Dropped));
    }
}
