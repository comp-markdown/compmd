//! The fonts a document declares under `md.fonts` (spec §9.4), as written, with where each part is: resolving checks
//! them (F009, F013, F026).

use super::{Located, Reader};
use crate::diagnostic::Location;
use serde_json::Value;

/// A font as `md.fonts` declares it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FontDeclaration {
    /// Its name, where its key is.
    pub(crate) name: Located<String>,
    /// Its `fallback` as written, if given.
    pub(crate) fallback: Option<Located<Value>>,
    /// Its `files`: `None` when it isn't a list, with where the declaration is.
    pub(crate) files: Option<Vec<FontFileEntry>>,
}

/// An entry of a font's `files`, as written.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FontFileEntry {
    /// Where the entry is.
    pub(crate) at: Location,
    /// Its `from`, `weight` and `style`, each if given.
    pub(crate) from: Option<Located<Value>>,
    pub(crate) weight: Option<Located<Value>>,
    pub(crate) style: Option<Located<Value>>,
}

impl Reader<'_> {
    /// The fonts `md.fonts` declares, in written order.
    pub(super) fn fonts(&self, md: &Value) -> Vec<FontDeclaration> {
        md.get("fonts")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .map(|(name, declared)| {
                let pointer = format!("/md/fonts/{name}");
                FontDeclaration {
                    name: Located {
                        value: name.clone(),
                        at: self.key_at(&pointer),
                    },
                    fallback: self.located(declared, &pointer, "fallback"),
                    files: declared
                        .get("files")
                        .and_then(Value::as_array)
                        .map(|files| {
                            files
                                .iter()
                                .enumerate()
                                .map(|(i, entry)| {
                                    let pointer = format!("{pointer}/files/{i}");
                                    FontFileEntry {
                                        at: self.value_at(&pointer),
                                        from: self.located(entry, &pointer, "from"),
                                        weight: self.located(entry, &pointer, "weight"),
                                        style: self.located(entry, &pointer, "style"),
                                    }
                                })
                                .collect()
                        }),
                }
            })
            .collect()
    }

    /// `entry`'s `key` as written, with where it is, when `pointer` points at `entry`.
    fn located(&self, entry: &Value, pointer: &str, key: &str) -> Option<Located<Value>> {
        Some(Located {
            value: entry.get(key)?.clone(),
            at: self.content_at(&format!("{pointer}/{key}")),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{ParseOptions, parse};
    use serde_json::json;

    #[test]
    fn reads_each_font_with_its_files_and_their_places() {
        let fonts = parse(
            "---\nmd:\n  fonts:\n    harbour-serif:\n      fallback: heading\n      files:\n        - { from: ./h.woff2 }\n        - { from: \"./h-bold.woff2\", weight: bold, style: italic }\n    plain: {}\n---\n",
            &ParseOptions::default(),
        )
        .settings
        .fonts;
        assert_eq!(fonts.len(), 2);
        let harbour = &fonts[0];
        assert_eq!(
            (harbour.name.value.as_str(), harbour.name.at),
            ("harbour-serif", Location::new(4, 5))
        );
        assert_eq!(
            harbour.fallback,
            Some(Located {
                value: json!("heading"),
                at: Location::new(5, 17)
            })
        );
        let files = harbour.files.as_ref().expect("files");
        assert_eq!(files[0].at, Location::new(7, 11));
        assert_eq!(
            files[1].from,
            Some(Located {
                value: json!("./h-bold.woff2"),
                at: Location::new(8, 20)
            })
        );
        assert_eq!(
            files[1].weight.as_ref().map(|w| w.value.clone()),
            Some(json!("bold"))
        );
        assert_eq!(files[0].style, None);
        assert_eq!(
            (fonts[1].fallback.as_ref(), fonts[1].files.as_ref()),
            (None, None)
        );
    }
}
