//! A document's fonts (spec §9.4): those `md.fonts` declares, checked (F009, F013, F026), those its imports take
//! (F013, F021), those it exports, and each font's one name in the expanded tree.

use super::Loader;
use crate::diagnostic::{Code, Location};
use crate::parse::params::is_name;
use crate::reserved;
use crate::resolve::fonts::FontNamesBuilder;
use crate::resolve::{
    DocumentId, Fallback, Font, FontBinding, FontFile, FontId, FontNames, Style, Weight,
};
use crate::settings::{FontDeclaration, FontFileEntry, Import, Located, Names};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

/// The file types a font's files may be.
const EXTENSIONS: [&str; 2] = [".woff2", ".woff"];

impl Loader<'_, '_> {
    /// Declares the fonts `md.fonts` of the document `id` declares, each checked: one with no file left is dropped,
    /// its uses absent without a diagnostic of their own.
    pub(super) fn declare_fonts(&mut self, id: DocumentId, fonts: &mut FontNamesBuilder) {
        for declaration in self.document(id).settings.fonts.clone() {
            let name = &declaration.name;
            if !is_name(&name.value) {
                self.report(
                    id,
                    Code::F009,
                    name.at,
                    format!("`{}` isn't a name, so it can't name a font", name.value),
                );
                continue;
            }
            if !self.admit_font(id, fonts, &name.value, name.at) {
                continue;
            }
            let binding = match self.font(id, &declaration) {
                Some(font) => {
                    self.fonts.push(font);
                    FontBinding::Font(FontId(self.fonts.len() - 1))
                }
                None => FontBinding::Dropped,
            };
            fonts.declare(&name.value, binding);
        }
    }

    /// The font `declaration` declares in the document `id`, checked; `None` when it has no file left.
    fn font(&mut self, id: DocumentId, declaration: &FontDeclaration) -> Option<Font> {
        let fallback = self.fallback(id, declaration.fallback.as_ref());
        let Some(entries) = declaration.files.as_ref().filter(|files| !files.is_empty()) else {
            self.report(
                id,
                Code::F009,
                declaration.name.at,
                format!(
                    "`{}` has no `files`: a font is a list of its `.woff2` or `.woff` files; it is dropped",
                    declaration.name.value
                ),
            );
            return None;
        };
        let mut files: Vec<FontFile> = Vec::new();
        for entry in entries {
            let Some(file) = self.font_file(id, entry) else {
                continue;
            };
            if files
                .iter()
                .any(|known| (known.weight, known.style) == (file.weight, file.style))
            {
                self.report(
                    id,
                    Code::F013,
                    entry.at,
                    format!(
                        "`{}` has a file of weight `{}` and style `{}` already; the first wins",
                        declaration.name.value,
                        file.weight.name(),
                        file.style.name()
                    ),
                );
                continue;
            }
            files.push(file);
        }
        (!files.is_empty()).then(|| Font {
            document: id,
            name: declaration.name.value.clone(),
            family: String::new(),
            fallback,
            files,
        })
    }

    /// A font's `fallback`: `body` when it isn't given, or isn't a token (F009).
    fn fallback(&mut self, id: DocumentId, written: Option<&Located<Value>>) -> Fallback {
        let Some(written) = written else {
            return Fallback::Body;
        };
        if let Some(fallback) = written.value.as_str().and_then(Fallback::from_name) {
            return fallback;
        }
        self.report(
            id,
            Code::F009,
            written.at,
            format!(
                "a font's `fallback` is `body`, `heading` or `mono`, not {}; `body` is used",
                written.value
            ),
        );
        Fallback::Body
    }

    /// A file of a font, checked: a relative path to a `.woff2` or `.woff` file (F009) in its root (F026), of a
    /// weight and style a font takes (F009); `None` when it isn't one.
    fn font_file(&mut self, id: DocumentId, entry: &FontFileEntry) -> Option<FontFile> {
        let from = entry.from.as_ref();
        let Some(path) = from
            .and_then(|from| from.value.as_str())
            .filter(|path| is_font_path(path))
        else {
            let written = from.map_or_else(|| "nothing".into(), |from| from.value.to_string());
            let at = from.map_or(entry.at, |from| from.at);
            self.report(
                id,
                Code::F009,
                at,
                format!("a font's file is a relative path to a `.woff2` or `.woff` file, not {written}; it is left out"),
            );
            return None;
        };
        if self.document(id).origin.join(path).is_none() {
            let at = from.map_or(entry.at, |from| from.at);
            self.report(
                id,
                Code::F026,
                at,
                format!("`{path}` leaves its root; it is left out"),
            );
            return None;
        }
        let weight = self.variant(id, entry.weight.as_ref(), "weight", Weight::NAMES)?;
        let style = self.variant(id, entry.style.as_ref(), "style", Style::NAMES)?;
        Some(FontFile {
            from: path.to_string(),
            weight: Weight::from_name(weight).unwrap_or(Weight::Normal),
            style: Style::from_name(style).unwrap_or(Style::Normal),
        })
    }

    /// A file's `weight` or `style` (`key`), one of `names`, the first by default; `None` when it is another (F009).
    fn variant(
        &mut self,
        id: DocumentId,
        written: Option<&Located<Value>>,
        key: &str,
        names: &'static [&'static str],
    ) -> Option<&'static str> {
        let Some(written) = written else {
            return names.first().copied();
        };
        let found = written
            .value
            .as_str()
            .and_then(|value| names.iter().copied().find(|name| *name == value));
        if found.is_none() {
            let takes: Vec<String> = names.iter().map(|name| format!("`{name}`")).collect();
            self.report(
                id,
                Code::F009,
                written.at,
                format!(
                    "a font file's `{key}` is {}, not {}; the file is left out",
                    takes.join(" or "),
                    written.value
                ),
            );
        }
        found
    }

    /// Whether the font name `name` may be given in the document `id`: not a type token's, and not given yet (F013).
    fn admit_font(
        &mut self,
        id: DocumentId,
        fonts: &FontNamesBuilder,
        name: &str,
        at: Location,
    ) -> bool {
        if let Some(why) = reserved::as_font(name) {
            self.report(id, Code::F013, at, why);
            return false;
        }
        if fonts.is_declared(name) {
            self.report(
                id,
                Code::F013,
                at,
                format!("a font `{name}` is already declared in this document; the first declaration wins"),
            );
            return false;
        }
        true
    }

    /// Binds the fonts `import` takes from the loaded document `target`, or, with none, drops them: an import that
    /// loads nothing, a pending one, or a component file, which exports no fonts (F021 for each listed).
    pub(super) fn import_fonts(
        &mut self,
        id: DocumentId,
        import: &Import,
        target: Option<DocumentId>,
        fonts: &mut FontNamesBuilder,
    ) {
        let Some(listed) = &import.fonts else {
            return;
        };
        let exports = target.map(|target| self.document(target).font_exports.clone());
        match listed {
            Names::All(_) => match exports {
                Some(exports) => {
                    for (name, binding) in exports {
                        if let FontBinding::Font(font) = binding {
                            fonts.share(&name, font, &import.from.value);
                        }
                    }
                }
                None => fonts.incomplete(),
            },
            Names::Listed(entries) => {
                for entry in entries {
                    let binding = match exports.as_ref().map(|exports| exports.get(&entry.name)) {
                        Some(Some(binding)) => binding.clone(),
                        Some(None) => {
                            self.report(
                                id,
                                Code::F021,
                                entry.at,
                                format!(
                                    "`{}` doesn't export a font `{}`",
                                    import.from.value, entry.name
                                ),
                            );
                            FontBinding::Dropped
                        }
                        None => FontBinding::Dropped,
                    };
                    if self.admit_font(id, fonts, &entry.renamed, entry.at) {
                        fonts.declare(&entry.renamed, binding);
                    }
                }
            }
        }
    }

    /// F021 for each font a component file's import lists: a component file exports one component and no font.
    pub(super) fn no_fonts_in_file(&mut self, id: DocumentId, import: &Import) {
        let Some(Names::Listed(entries)) = &import.fonts else {
            return;
        };
        for entry in entries {
            self.report(
                id,
                Code::F021,
                entry.at,
                format!(
                    "`{}` is a component file, which exports no font `{}`",
                    import.from.value, entry.name
                ),
            );
        }
    }

    /// The fonts the document `id` exports (spec §6.2): `"*"` those `md.fonts` declares, a list the names it gives
    /// (an imported font too), each under its public name. A keyword's or token's name is F013.
    pub(super) fn export_fonts(
        &mut self,
        id: DocumentId,
        fonts: &FontNames,
    ) -> BTreeMap<String, FontBinding> {
        let Some(listed) = self
            .document(id)
            .settings
            .export
            .as_ref()
            .and_then(|export| export.fonts.clone())
        else {
            return BTreeMap::new();
        };
        let mut exports = BTreeMap::new();
        match listed {
            Names::All(_) => {
                for (index, font) in self.fonts.iter().enumerate() {
                    if font.document == id {
                        exports.insert(font.name.clone(), FontBinding::Font(FontId(index)));
                    }
                }
            }
            Names::Listed(entries) => {
                for entry in entries {
                    if let Some(why) = reserved::as_font(&entry.renamed) {
                        self.report(id, Code::F013, entry.at, why);
                        continue;
                    }
                    if let Some(binding) = fonts.get(&entry.name) {
                        exports.insert(entry.renamed.clone(), binding.clone());
                    }
                }
            }
        }
        exports
    }
}

/// Whether `path` is a relative path to a `.woff2` or `.woff` file: no scheme, not from the root of its host, no
/// query or fragment.
fn is_font_path(path: &str) -> bool {
    let has_scheme = path.split_once(':').is_some();
    let lower = path.to_ascii_lowercase();
    !has_scheme
        && !path.starts_with('/')
        && !path.contains(['?', '#'])
        && EXTENSIONS
            .iter()
            .any(|extension| lower.ends_with(extension) && lower.len() > extension.len())
}

/// Gives each font its one name in the expanded tree (spec §12.3): its name, or that with the first of `-2`, `-3`,
/// … no other font has, the processed document's fonts first, then the others in load order.
pub(super) fn name_families(fonts: &mut [Font], main: DocumentId) {
    let mut order: Vec<usize> = (0..fonts.len()).collect();
    order.sort_by_key(|index| fonts[*index].document != main);
    let mut taken: HashSet<String> = HashSet::new();
    // Fewer names are taken than there are fonts, so one of that many more is free.
    let tries = fonts.len() + 1;
    for index in order {
        let name = fonts[index].name.clone();
        let family = std::iter::once(name.clone())
            .chain((2..=tries).map(|n| format!("{name}-{n}")))
            .find(|family| !taken.contains(family))
            .unwrap_or(name);
        taken.insert(family.clone());
        fonts[index].family = family;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font(document: usize, name: &str) -> Font {
        Font {
            document: DocumentId(document),
            name: name.into(),
            family: String::new(),
            fallback: Fallback::Body,
            files: Vec::new(),
        }
    }

    #[test]
    fn the_processed_documents_fonts_keep_their_names_and_others_take_the_next_free_one() {
        let mut fonts = vec![
            font(0, "serif"),
            font(0, "serif-2"),
            font(1, "serif"),
            font(1, "sans"),
        ];
        name_families(&mut fonts, DocumentId(1));
        let families: Vec<&str> = fonts.iter().map(|font| font.family.as_str()).collect();
        assert_eq!(families, ["serif-2", "serif-2-2", "serif", "sans"]);
    }

    #[test]
    fn a_font_file_is_a_relative_woff2_or_woff_path() {
        for path in ["./a.woff2", "fonts/a.WOFF", "../x/a.woff"] {
            assert!(is_font_path(path), "{path}");
        }
        for path in [
            "https://x.com/a.woff2",
            "/a.woff2",
            "a.ttf",
            "a.woff2?v=1",
            ".woff2",
            "data:a.woff",
        ] {
            assert!(!is_font_path(path), "{path}");
        }
    }
}
