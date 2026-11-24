//! Fonts of a document's own in the expanded tree (spec §9.4, §12.3): a `font-family` value naming a font of the
//! document it is written in becomes the font's one name in the tree, and the root lists the fonts its params name.

use super::Expander;
use super::destinations;
use super::env::Env;
use crate::cel::Value;
use crate::diagnostic::Code;
use crate::expanded::{self, Content};
use crate::resolve::{Fallback, FontBinding, Resolved};
use crate::tree::Param;
use crate::validate::params::value_at;
use indexmap::IndexSet;

/// The param that names a font.
pub(super) const FONT_FAMILY: &str = "font-family";

impl Expander<'_> {
    /// `value`, a `font-family`'s, written in `env`'s document: a type token as is, a font's name as the font's one
    /// name in the tree. `None` for a font from an import that failed or is pending, and for a name no font has
    /// there, F009, which only a value given at render can be: validating refused the others.
    pub(super) fn font_family(&mut self, value: Value, param: &Param, env: &Env) -> Option<Value> {
        let Ok(Some(written)) = value.written() else {
            return Some(value);
        };
        let written = written.trim();
        if Fallback::from_name(written).is_some() {
            return Some(value);
        }
        let resolved = self.resolved;
        match resolved.document(env.document).fonts.get(written) {
            Some(FontBinding::Font(font)) => Some(Value::string(&resolved.font(*font).family)),
            Some(FontBinding::Dropped) => None,
            Some(FontBinding::Ambiguous(_)) | None => {
                self.report(
                    Code::F009,
                    env,
                    value_at(param),
                    format!(
                        "`font-family` is `{written}`, which names no type token or font of this document, so the param is treated as absent"
                    ),
                );
                None
            }
        }
    }
}

/// The fonts the params of `nodes` name, in the order they first do, each with its files resolved for the processed
/// document of `resolved` (spec §8.6).
pub(super) fn used(nodes: &[expanded::Node], resolved: &Resolved) -> Vec<expanded::Font> {
    let mut families: IndexSet<&str> = IndexSet::new();
    named(nodes, &mut families);
    let main = &resolved.document(resolved.main).origin;
    families
        .into_iter()
        .filter_map(|family| resolved.font_of_family(family))
        .map(|font| {
            let origin = &resolved.document(font.document).origin;
            expanded::Font {
                family: font.family.clone(),
                fallback: font.fallback.name().to_string(),
                files: font
                    .files
                    .iter()
                    .filter_map(|file| {
                        let destination = destinations::resolved(&file.from, origin, main)?;
                        Some(expanded::FontFile {
                            url: destination.url,
                            pack: destination.pack,
                            weight: file.weight.name().to_string(),
                            style: file.style.name().to_string(),
                        })
                    })
                    .collect(),
            }
        })
        .collect()
}

/// Adds the values of the `font-family` params of `nodes` and what they hold to `families`, each once. An example's
/// root lists its own.
fn named<'n>(nodes: &'n [expanded::Node], families: &mut IndexSet<&'n str>) {
    for node in nodes {
        let params: Vec<&[expanded::Param]> = match &node.content {
            Content::ComponentContainer(container) => vec![&container.params],
            Content::ComponentLeaf(leaf) | Content::ComponentInline(leaf) => vec![&leaf.params],
            Content::Code(code) => {
                let rendered = code
                    .component
                    .as_ref()
                    .map(|component| component.params.as_slice());
                std::iter::once(code.params.as_slice())
                    .chain(rendered)
                    .collect()
            }
            _ => Vec::new(),
        };
        let values = params
            .into_iter()
            .flatten()
            .filter(|param| param.key == FONT_FAMILY)
            .filter_map(|param| param.value.as_str());
        families.extend(values);
        for list in node.content.lists() {
            named(list, families);
        }
    }
}
