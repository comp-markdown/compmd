//! What a document's frontmatter gives a render (spec §2.1, §6, §8.6, §8.7), for a tool that shows it: the author's
//! own fields (`md.doc`), the settings under `md`, each data import with what was loaded for it, and the inputs with
//! their values.

use crate::expanded::Inputs;
use crate::load;
use crate::resolve::{DocumentId, Resolved};
use crate::settings::{Names, Settings};
use crate::tree::DeclaredType;
use serde::Serialize;
use serde_json::{Map, Value};

/// What a processed document's frontmatter gives its render.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Frontmatter {
    /// The author's own fields, what `md.doc` holds.
    pub doc: Map<String, Value>,
    /// The settings under `md` other than data and inputs, which have their own fields.
    pub settings: DocumentSettings,
    /// Each data import, in written order, with what was loaded.
    pub data: Vec<DataImport>,
    /// The inputs, by name, with the values the render used.
    pub inputs: Inputs,
}

/// The settings under `md`: the version, language and time zone, and the component imports.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DocumentSettings {
    /// `md.version` as written.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// `md.lang`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    /// `md.tz`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tz: Option<String>,
    /// Each component import: where from, and the names it takes, or `*`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub import: Vec<ComponentImport>,
}

/// A component import (spec §6.1).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ComponentImport {
    /// Its `from`.
    pub from: String,
    /// The names it takes, each with the local name it gets when that differs; `*` for all of them.
    pub names: Vec<String>,
}

/// A data import (spec §8.6), and what was loaded for it.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DataImport {
    /// The name it gets under `md.data`.
    pub name: String,
    /// Its `from`, as written.
    pub from: String,
    /// Its schema's fields and their types.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub schema: Vec<(String, DeclaredType)>,
    /// The local file loaded when a remote or `host:` source fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback: Option<String>,
    /// What was loaded: `null` when it failed without a fallback, which a diagnostic says.
    pub value: Value,
}

/// The frontmatter of the document `id` as `loaded` gives it.
pub(crate) fn of(resolved: &Resolved, id: DocumentId, loaded: &load::Loaded) -> Frontmatter {
    let Settings {
        doc,
        imports,
        data,
        version,
        lang,
        tz,
        ..
    } = &resolved.document(id).settings;
    let values = loaded.data(id).to_json();
    Frontmatter {
        doc: doc.clone(),
        settings: DocumentSettings {
            version: version.clone(),
            lang: lang.clone(),
            tz: tz.clone(),
            import: imports
                .iter()
                .map(|import| ComponentImport {
                    from: import.from.value.clone(),
                    names: match &import.names {
                        Names::All(_) => vec!["*".into()],
                        Names::Listed(entries) => entries
                            .iter()
                            .map(|entry| {
                                if entry.name == entry.renamed {
                                    entry.name.clone()
                                } else {
                                    format!("{} as {}", entry.name, entry.renamed)
                                }
                            })
                            .collect(),
                    },
                })
                .collect(),
        },
        data: data
            .iter()
            .map(|import| DataImport {
                name: import.name.clone(),
                from: import.from.value.clone(),
                schema: import
                    .schema
                    .iter()
                    .flatten()
                    .map(|field| {
                        (
                            field.declaration.name.clone(),
                            field.declaration.declared.clone(),
                        )
                    })
                    .collect(),
                fallback: import.fallback.clone(),
                value: values.get(&import.name).cloned().unwrap_or(Value::Null),
            })
            .collect(),
        inputs: loaded.inputs.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::context::RenderContext;
    use crate::expand::Options;
    use crate::{Ambient, ParseOptions, Processor};
    use serde_json::json;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn gives_settings_imports_data_and_inputs() {
        let text = "---\nmd:\n  version: 1\n  lang: de\n  tz: UTC\n  import:\n    - from: ./ui.comp.md\n      names: \"*\"\n    - from: ./more.comp.md\n      names: [card, {name: badge, as: tag}]\n  data:\n    - from: ./gone.csv\n      as: gone\n      fallback: ./kept.csv\n      schema: {n: number}\n  inputs:\n    region: eu | us = eu\n---\n";
        let files = HashMap::from([
            ("ui.comp.md".to_string(), String::new()),
            ("more.comp.md".to_string(), String::new()),
        ]);
        let context = RenderContext {
            now: 0,
            tz: "UTC".into(),
            time: Rc::new(()),
            inputs: serde_json::Map::default(),
        };
        let frontmatter = Processor::new("a.comp.md", text)
            .parse(&ParseOptions::default())
            .resolve(files, (), &Ambient::default())
            .validate(&[])
            .load(context, (), ())
            .expand(&Options::default())
            .frontmatter();
        let json = serde_json::to_value(&frontmatter).expect("JSON");
        assert_eq!(
            json["settings"],
            json!({
                "version": "1",
                "lang": "de",
                "tz": "UTC",
                "import": [
                    { "from": "./ui.comp.md", "names": ["*"] },
                    { "from": "./more.comp.md", "names": ["card", "badge as tag"] }
                ]
            })
        );
        assert_eq!(
            json["data"],
            json!([{ "name": "gone", "from": "./gone.csv", "schema": [["n", "number"]], "fallback": "./kept.csv", "value": null }])
        );
        assert_eq!(json["inputs"]["region"]["value"], "eu");
        assert_eq!(json["doc"], json!({}));
    }
}
