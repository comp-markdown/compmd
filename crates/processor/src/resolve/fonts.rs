//! Fonts of a document's own (spec §9.4): each font a loaded document declares, and what each font name of a
//! document stands for, by the levels of spec §6.3, apart from its component names.

use super::DocumentId;
use crate::named::named;
use std::collections::BTreeMap;

/// A font, by its place in [`Resolved::fonts`](super::Resolved).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FontId(pub(super) usize);

named! {
    /// A type token a font falls back to (spec §9.4), and that `font-family` takes.
    pub(crate) enum Fallback {
        Body = "body",
        Heading = "heading",
        Mono = "mono",
    }
}

named! {
    /// The weight of a font's file.
    pub(crate) enum Weight {
        Normal = "normal",
        Bold = "bold",
    }
}

named! {
    /// The style of a font's file.
    pub(crate) enum Style {
        Normal = "normal",
        Italic = "italic",
    }
}

/// A font a document declares, checked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Font {
    /// The document that declares it.
    pub(crate) document: DocumentId,
    /// Its name there.
    pub(crate) name: String,
    /// The one name it has in the expanded tree: its name, or that with `-2`, `-3`, … when another font has it.
    pub(crate) family: String,
    pub(crate) fallback: Fallback,
    /// Its files, in written order.
    pub(crate) files: Vec<FontFile>,
}

/// A file of a font.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FontFile {
    /// Its `from`, a relative path in the declaring document.
    pub(crate) from: String,
    pub(crate) weight: Weight,
    pub(crate) style: Style,
}

/// What a font name stands for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FontBinding {
    /// A font.
    Font(FontId),
    /// A name two wildcard imports give, these: using it is F023.
    Ambiguous(Vec<String>),
    /// A name from an import that failed or is pending: a param naming it is treated as absent, without F009.
    Dropped,
}

/// The font names of a document.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FontNames {
    bindings: BTreeMap<String, FontBinding>,
    /// Whether a wildcard import of fonts failed or is pending, so that a name may be one of its unknown fonts.
    incomplete: bool,
}

impl FontNames {
    /// What `name` stands for; `None` when no font has it (F009).
    pub(crate) fn get(&self, name: &str) -> Option<&FontBinding> {
        const DROPPED: &FontBinding = &FontBinding::Dropped;
        self.bindings
            .get(name)
            .or_else(|| self.incomplete.then_some(DROPPED))
    }

    /// Every name, for what `font-family` takes.
    pub(crate) fn all(&self) -> impl Iterator<Item = &str> {
        self.bindings.keys().map(String::as_str)
    }
}

/// Collects a document's font names: declared and listed ones, then those of wildcard imports.
#[derive(Debug, Default)]
pub(crate) struct FontNamesBuilder {
    declared: BTreeMap<String, FontBinding>,
    /// Each name of a wildcard import, with the fonts and the sources giving it.
    shared: BTreeMap<String, Vec<(FontId, String)>>,
    incomplete: bool,
}

impl FontNamesBuilder {
    /// Whether `md.fonts` or a listed import already gives `name`.
    pub(crate) fn is_declared(&self, name: &str) -> bool {
        self.declared.contains_key(name)
    }

    /// Gives `name` at the first two levels; the caller has checked it isn't given yet (F013).
    pub(crate) fn declare(&mut self, name: &str, binding: FontBinding) {
        self.declared.insert(name.to_string(), binding);
    }

    /// Adds `name` from a wildcard import, named `source`.
    pub(crate) fn share(&mut self, name: &str, id: FontId, source: &str) {
        let providers = self.shared.entry(name.to_string()).or_default();
        if !providers.iter().any(|(known, _)| *known == id) {
            providers.push((id, source.to_string()));
        }
    }

    /// Notes that a wildcard import of fonts failed or is pending.
    pub(crate) fn incomplete(&mut self) {
        self.incomplete = true;
    }

    /// The names, each with what the highest level gives it.
    pub(crate) fn build(self) -> FontNames {
        let mut bindings = self.declared;
        for (name, providers) in self.shared {
            bindings
                .entry(name)
                .or_insert_with(|| match &providers[..] {
                    [(id, _)] => FontBinding::Font(*id),
                    _ => FontBinding::Ambiguous(
                        providers.into_iter().map(|(_, source)| source).collect(),
                    ),
                });
        }
        FontNames {
            bindings,
            incomplete: self.incomplete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_names_shadow_wildcard_ones_and_two_wildcards_are_ambiguous() {
        let mut names = FontNamesBuilder::default();
        names.declare("serif", FontBinding::Font(FontId(0)));
        names.share("serif", FontId(1), "@a/look");
        names.share("sans", FontId(2), "@a/look");
        names.share("sans", FontId(3), "@b/look");
        names.share("mono-x", FontId(4), "@a/look");
        names.share("mono-x", FontId(4), "@c/re-export");
        assert!(names.is_declared("serif") && !names.is_declared("sans"));
        let names = names.build();
        assert_eq!(names.get("serif"), Some(&FontBinding::Font(FontId(0))));
        assert_eq!(
            names.get("sans"),
            Some(&FontBinding::Ambiguous(vec![
                "@a/look".into(),
                "@b/look".into()
            ]))
        );
        assert_eq!(names.get("mono-x"), Some(&FontBinding::Font(FontId(4))));
        assert_eq!(names.get("other"), None);
        assert_eq!(names.all().collect::<Vec<_>>(), ["mono-x", "sans", "serif"]);
    }

    #[test]
    fn an_incomplete_wildcard_import_may_hold_any_name() {
        let mut names = FontNamesBuilder::default();
        names.incomplete();
        assert_eq!(names.build().get("any"), Some(&FontBinding::Dropped));
    }

    #[test]
    fn the_tokens_weights_and_styles_have_their_names() {
        assert_eq!(Fallback::from_name("heading"), Some(Fallback::Heading));
        assert_eq!(Weight::Bold.name(), "bold");
        assert_eq!(Style::from_name("oblique"), None);
    }
}
