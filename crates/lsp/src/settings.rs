//! The editor's settings ([the language server], decision 12), in one namespace, `compmd`: the suggested scopes and
//! the network switch (decision 7), the preview's browser (decision 13), the trace level, and the drawing's and the
//! tables' ([a document drawn in the editor], decisions 6 and 7). How a document renders comes from its `compmd.config.json`, never
//! from settings.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

mod render;
mod tables;

pub(crate) use render::{GlyphSet, LanguagePlace, RenderSettings, Reveal};
pub(crate) use tables::TableSettings;

use serde::Deserialize;
use serde_json::Value;

/// How much the server says in the editor's log (`window/logMessage`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Trace {
    /// Its errors only.
    #[default]
    Off,
    /// What it does with each document, too.
    Messages,
    /// Everything, timings included.
    Verbose,
}

/// What the known packs come from (decision 7).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub(crate) struct PackSettings {
    /// The writer's own suggested scopes, besides CompMD's and the project's.
    pub(crate) suggest: Vec<String>,
    /// Whether the server may reach registries at all.
    pub(crate) network: bool,
}

impl Default for PackSettings {
    fn default() -> Self {
        Self {
            suggest: Vec::new(),
            network: true,
        }
    }
}

/// The preview's settings (decision 13).
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub(crate) struct PreviewSettings {
    /// The browser to open it in; the system's when absent.
    pub(crate) browser: Option<String>,
}

/// Markdown files' settings ([a document drawn in the editor], decision 13; read by the Markdown files' task).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub(crate) struct MarkdownSettings {
    /// Whether `.md` files are served at all.
    pub(crate) enabled: bool,
}

impl Default for MarkdownSettings {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// The `compmd` settings, every default filled in; what isn't a setting is ignored.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub(crate) struct Settings {
    pub(crate) packs: PackSettings,
    pub(crate) preview: PreviewSettings,
    pub(crate) trace: Trace,
    pub(crate) render: RenderSettings,
    pub(crate) markdown: MarkdownSettings,
    pub(crate) tables: TableSettings,
}

impl Settings {
    /// The settings in `value`: the `compmd` object a client sends (`initializationOptions`, or
    /// `workspace/didChangeConfiguration`'s `settings`, which may hold it under `compmd`); the defaults for what is
    /// missing or wrong.
    pub(crate) fn from_value(value: &Value) -> Self {
        let settings = value.get("compmd").unwrap_or(value);
        serde_json::from_value(settings.clone()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn settings_have_defaults_and_come_under_compmd_or_alone() {
        assert_eq!(Settings::from_value(&json!(null)), Settings::default());
        assert!(Settings::default().packs.network);
        let given = json!({ "compmd": { "packs": { "suggest": ["@acme"], "network": false }, "preview": { "browser": "firefox" }, "trace": "verbose", "other": 1 } });
        let settings = Settings::from_value(&given);
        assert_eq!(
            settings.packs,
            PackSettings {
                suggest: vec!["@acme".into()],
                network: false
            }
        );
        assert_eq!(settings.preview.browser.as_deref(), Some("firefox"));
        assert_eq!(settings.trace, Trace::Verbose);
        assert_eq!(
            Settings::from_value(&given["compmd"]),
            settings,
            "the object alone"
        );
        assert_eq!(
            Settings::from_value(&json!({ "trace": "messages" })).trace,
            Trace::Messages
        );
        assert_eq!(
            Settings::from_value(&json!({ "trace": "loud" })),
            Settings::default(),
            "a wrong value"
        );
    }

    #[test]
    fn markdown_files_are_served_and_drawn_by_default() {
        let settings = Settings::default();
        assert!(settings.markdown.enabled && settings.render.markdown_files);
        let off = Settings::from_value(
            &json!({ "markdown": { "enabled": false }, "render": { "markdownFiles": false } }),
        );
        assert!(!off.markdown.enabled && !off.render.markdown_files);
    }

    #[test]
    fn tables_are_aligned_on_type_and_on_format_by_default() {
        let tables = Settings::default().tables;
        assert!(tables.align_on_type && tables.align_on_format);
        let off = Settings::from_value(
            &json!({ "compmd": { "tables": { "alignOnType": false, "alignOnFormat": false } } }),
        );
        assert_eq!(
            off.tables,
            TableSettings {
                align_on_type: false,
                align_on_format: false
            }
        );
        assert!(
            Settings::from_value(&json!({ "tables": { "alignOnType": false } }))
                .tables
                .align_on_format,
            "each on its own"
        );
    }
}
