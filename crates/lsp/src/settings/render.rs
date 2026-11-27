//! The drawing's settings ([a document drawn in the editor], decision 6), under `compmd.render`: whether a document
//! is drawn, each element's switch and its options, the glyphs, when a mark gives way to the raw text, and whether
//! values are shown as inlay hints. The server plans with them; `render.modes`, the renderer's own, is read by the
//! editor's package from the same namespace and ignored here.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};

/// The drawing's settings, every default filled in.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct RenderSettings {
    /// Whether documents are drawn at all.
    pub(crate) enabled: bool,
    /// Whether `.md` files are drawn (decision 13; read by the Markdown files' task).
    pub(crate) markdown_files: bool,
    /// Headings: the `#`s as the level's glyph, the line in its style.
    pub(crate) heading: Switch<HeadingOptions>,
    /// Fenced code blocks and inline code.
    pub(crate) code: Switch<CodeOptions>,
    /// Tables drawn over their aligned text.
    pub(crate) table: Switch<TableOptions>,
    /// List bullets by depth.
    pub(crate) list: Toggle,
    /// Task list checkboxes.
    pub(crate) checkbox: Toggle,
    /// Block quotes' `>` by depth.
    pub(crate) quote: Toggle,
    /// GitHub alerts' `[!NOTE]` as their icon and title.
    pub(crate) callout: Toggle,
    /// Thematic breaks.
    pub(crate) rule: Toggle,
    /// Containers as boxes, their slots as dividers.
    pub(crate) container: Toggle,
    /// A leaf component's name in the component style.
    pub(crate) component: Toggle,
    /// Params, and the frontmatter's `md` key, set back.
    pub(crate) param: Toggle,
    /// The glyph set the marks' text comes from.
    pub(crate) glyphs: GlyphSet,
    /// The writer's own text for a glyph, by the glyph's name (`heading.2`, `bullet.1`).
    pub(crate) glyph: BTreeMap<String, String>,
    /// When a mark gives way to the raw text.
    pub(crate) reveal: Reveal,
    /// Whether an interpolation's and an expression's value is shown beside it, as an inlay hint (decision 9).
    pub(crate) values: bool,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            markdown_files: true,
            heading: Switch::default(),
            code: Switch::default(),
            table: Switch::default(),
            list: Toggle::default(),
            checkbox: Toggle::default(),
            quote: Toggle::default(),
            callout: Toggle::default(),
            rule: Toggle::default(),
            container: Toggle::default(),
            component: Toggle::default(),
            param: Toggle::default(),
            glyphs: GlyphSet::default(),
            glyph: BTreeMap::new(),
            reveal: Reveal::default(),
            values: true,
        }
    }
}

/// The switch of an element that has no options of its own: `true`, `false`, or `{}`.
pub(crate) type Toggle = Switch<NoOptions>;

/// An element's options when it has none.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
pub(crate) struct NoOptions {}

/// An element's switch: `true`, `false`, or its options, which turn it on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Switch<T> {
    /// Whether the element is drawn.
    pub(crate) on: bool,
    /// How it is drawn.
    pub(crate) options: T,
}

impl<T: Default> Default for Switch<T> {
    fn default() -> Self {
        Self {
            on: true,
            options: T::default(),
        }
    }
}

impl<'de, T: Deserialize<'de> + Default> Deserialize<'de> for Switch<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// A switch as the writer writes it.
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Written<T> {
            On(bool),
            Options(T),
        }
        Ok(match Written::deserialize(deserializer)? {
            Written::On(on) => Self {
                on,
                options: T::default(),
            },
            Written::Options(options) => Self { on: true, options },
        })
    }
}

/// A heading's options.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub(crate) struct HeadingOptions {
    /// Whether the heading's line is in its style to the window's edge.
    pub(crate) fill: bool,
}

impl Default for HeadingOptions {
    fn default() -> Self {
        Self { fill: true }
    }
}

/// A code block's options.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub(crate) struct CodeOptions {
    /// Where its language is shown.
    pub(crate) language: LanguagePlace,
}

/// Where a code block's language is shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LanguagePlace {
    /// On a line of its own above the block.
    #[default]
    Above,
    /// On the opening fence, in place of it.
    Fence,
    /// Not shown.
    Off,
}

/// A table's options.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub(crate) struct TableOptions {
    /// Whether a border is drawn above and below it.
    pub(crate) borders: bool,
}

impl Default for TableOptions {
    fn default() -> Self {
        Self { borders: true }
    }
}

/// The glyph set a mark's text comes from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum GlyphSet {
    /// Box drawing and plain symbols.
    #[default]
    Unicode,
    /// Nerd Font icons for headings, languages and callouts.
    Nerd,
    /// ASCII only, for terminals and fonts without either.
    Ascii,
}

/// When a mark gives way to the raw text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Reveal {
    /// On the cursor's line: each mark names the lines of the part it draws.
    #[default]
    Line,
    /// While the cursor is in the element: each mark names the element's lines.
    Element,
    /// Never: no mark names lines.
    Off,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn read(value: serde_json::Value) -> RenderSettings {
        serde_json::from_value(value).expect("render settings")
    }

    #[test]
    fn everything_is_drawn_by_default() {
        let settings = read(json!({}));
        assert_eq!(settings, RenderSettings::default());
        assert!(settings.enabled && settings.markdown_files && settings.heading.on);
        assert!(settings.values, "values shown");
        assert!(!read(json!({ "values": false })).values);
        assert!(settings.heading.options.fill && settings.table.options.borders);
        assert_eq!(settings.code.options.language, LanguagePlace::Above);
        assert_eq!(
            (settings.glyphs, settings.reveal),
            (GlyphSet::Unicode, Reveal::Line)
        );
    }

    #[test]
    fn a_switch_is_a_boolean_or_its_options() {
        let settings = read(json!({
            "heading": false,
            "code": { "language": "fence" },
            "table": { "borders": false },
            "list": false,
            "glyphs": "nerd",
            "glyph": { "heading.2": "H2" },
            "reveal": "element",
            "modes": ["n"],
        }));
        assert!(!settings.heading.on);
        assert_eq!(
            settings.code,
            Switch {
                on: true,
                options: CodeOptions {
                    language: LanguagePlace::Fence
                }
            }
        );
        assert!(settings.table.on && !settings.table.options.borders);
        assert!(!settings.list.on && settings.checkbox.on);
        assert!(
            read(json!({ "quote": {} })).quote.on,
            "no options turn it on"
        );
        assert_eq!(settings.glyphs, GlyphSet::Nerd);
        assert_eq!(settings.glyph["heading.2"], "H2");
        assert_eq!(settings.reveal, Reveal::Element);
        assert_eq!(read(json!({ "reveal": "off" })).reveal, Reveal::Off);
        assert_eq!(
            read(json!({ "code": { "language": "off" }, "glyphs": "ascii" }))
                .code
                .options
                .language,
            LanguagePlace::Off
        );
        assert!(serde_json::from_value::<RenderSettings>(json!({ "heading": 3 })).is_err());
    }
}
