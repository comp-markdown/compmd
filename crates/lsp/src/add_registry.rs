//! Adding the registry of CompMD's own packs to a site's configuration ([the language server], Q4): the edit, which
//! the writer sees and can undo, made to the file the configuration is in, or to a new `compmd.config.json`; and who
//! then fetches the packs, which the question says. Nothing is installed.
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use compmd_project::Fetch;
use lsp_types::{
    CreateFile, DocumentChangeOperation, DocumentChanges, OneOf,
    OptionalVersionedTextDocumentIdentifier, Position, Range, ResourceOp, TextDocumentEdit,
    TextEdit, Uri, WorkspaceEdit,
};
use serde_json::Value;

use crate::encoding::Encoding;
use crate::json_object::JsonObject;
use crate::own_packs::SCOPE;

/// The configuration's own file.
const CONFIG_FILE: &str = "compmd.config.json";

/// The edit that adds a registry for CompMD's own packs to a site's configuration.
#[derive(Clone, Debug)]
pub(crate) struct AddRegistry {
    /// The file the configuration is in, or will be: `compmd.config.json`, or `package.json`.
    pub(crate) file: &'static str,
    /// Who fetches the packs once it is added.
    pub(crate) fetch: Fetch,
    /// The edit.
    pub(crate) edit: WorkspaceEdit,
}

/// The `file:` URI of `path`.
fn uri_of(path: &Path) -> Option<Uri> {
    url::Url::from_file_path(path).ok()?.as_str().parse().ok()
}

/// The position of the byte `offset` of `text`.
fn position_of(text: &str, offset: usize, encoding: Encoding) -> Position {
    let before = &text[..offset];
    let line_start = before.rfind('\n').map_or(0, |newline| newline + 1);
    let line = before.matches('\n').count() + 1;
    encoding.position(text, line, before[line_start..].chars().count() + 1)
}

/// Whether the project at `root` has a server to proxy registries: it builds with `@compmd/vite`.
fn has_proxy(root: &Path) -> bool {
    let Some(manifest) = fs::read_to_string(root.join("package.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
    else {
        return false;
    };
    ["dependencies", "devDependencies"]
        .iter()
        .any(|key| manifest[key].get("@compmd/vite").is_some())
}

/// Who fetches from the registries of the configuration `value`, once one for CompMD's own packs is added: what it
/// says; else the host when it already names registries, which it then proxies, or when the project has a proxy; else
/// the readers' browsers, which the configuration then has to say.
fn fetch_of(value: &Value, root: &Path) -> (Fetch, bool) {
    match value.get("fetch").and_then(Value::as_str) {
        Some("direct") => (Fetch::Direct, false),
        Some(_) => (Fetch::Host, false),
        None => {
            let names_registries = value
                .get("registries")
                .and_then(Value::as_object)
                .is_some_and(|registries| !registries.is_empty());
            if names_registries || has_proxy(root) {
                (Fetch::Host, false)
            } else {
                (Fetch::Direct, true)
            }
        }
    }
}

impl AddRegistry {
    /// The edit that adds `registry` for CompMD's own packs to the configuration of the project at `root`, its
    /// positions counted in `encoding`; `None` when the configuration's file isn't a JSON object.
    pub(crate) fn of(root: &Path, registry: &str, encoding: Encoding) -> Option<Self> {
        let scoped = format!("{}: {}", Value::from(SCOPE), Value::from(registry));
        let config = root.join(CONFIG_FILE);
        if let Ok(text) = fs::read_to_string(&config) {
            let object = JsonObject::top(&text)?;
            return Self::into_object(&config, CONFIG_FILE, &text, &object, &scoped, encoding);
        }
        let manifest = root.join("package.json");
        if let Ok(text) = fs::read_to_string(&manifest)
            && let Some(object) = JsonObject::top(&text)?.member(&text, "compmd")
        {
            return Self::into_object(&manifest, "package.json", &text, &object, &scoped, encoding);
        }
        Some(Self::new_file(&config, root, &scoped))
    }

    /// The edit that adds the member `scoped` to the registries of the configuration `object` in `text`, the file at
    /// `path`, named `file`.
    fn into_object(
        path: &Path,
        file: &'static str,
        text: &str,
        object: &JsonObject,
        scoped: &str,
        encoding: Encoding,
    ) -> Option<Self> {
        let value: Value = serde_json::from_str(text).ok()?;
        let value = if file == CONFIG_FILE {
            &value
        } else {
            &value["compmd"]
        };
        let (fetch, write_fetch) = fetch_of(value, path.parent()?);
        let mut added = Vec::new();
        let mut members = Vec::new();
        match object.member(text, "registries") {
            Some(registries) => added.push(registries.adding(text, &[scoped.to_owned()])),
            None => members.push(format!("\"registries\": {{ {scoped} }}")),
        }
        if write_fetch {
            members.push("\"fetch\": \"direct\"".to_owned());
        }
        if !members.is_empty() {
            added.push(object.adding(text, &members));
        }
        let edits = added
            .into_iter()
            .map(|(at, new_text)| {
                let at = position_of(text, at, encoding);
                TextEdit {
                    range: Range { start: at, end: at },
                    new_text,
                }
            })
            .collect();
        Some(Self {
            file,
            fetch,
            edit: WorkspaceEdit {
                changes: Some(HashMap::from([(uri_of(path)?, edits)])),
                ..WorkspaceEdit::default()
            },
        })
    }

    /// The edit that writes a new configuration at `path`, with `scoped` as its registries, of the project at `root`.
    fn new_file(path: &Path, root: &Path, scoped: &str) -> Self {
        let (fetch, write_fetch) = fetch_of(&Value::Null, root);
        let fetch_member = if write_fetch {
            ",\n  \"fetch\": \"direct\""
        } else {
            ""
        };
        let text = format!("{{\n  \"registries\": {{ {scoped} }}{fetch_member}\n}}\n");
        let operations = uri_of(path).map(|uri| {
            vec![
                DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
                    uri: uri.clone(),
                    options: None,
                    annotation_id: None,
                })),
                DocumentChangeOperation::Edit(TextDocumentEdit {
                    text_document: OptionalVersionedTextDocumentIdentifier { uri, version: None },
                    edits: vec![OneOf::Left(TextEdit {
                        range: Range::default(),
                        new_text: text,
                    })],
                }),
            ]
        });
        Self {
            file: CONFIG_FILE,
            fetch,
            edit: WorkspaceEdit {
                document_changes: operations.map(DocumentChanges::Operations),
                ..WorkspaceEdit::default()
            },
        }
    }

    /// The question asked when a document first imports one of CompMD's own packs its site can't load: what adding
    /// the registry does, and who then fetches the packs.
    pub(crate) fn question(&self) -> String {
        let fetched = match self.fetch {
            Fetch::Host => "your site's host then fetches them for its readers (`fetch: \"host\"`)",
            Fetch::Direct => {
                "readers' browsers then fetch them from npm, which sees their addresses (`fetch: \"direct\"`)"
            }
        };
        format!(
            "Your site loads packs from npm at runtime only if {} names it. Add npm for `{SCOPE}`? {fetched}.",
            self.file
        )
    }

    /// The quick fix's title.
    pub(crate) fn title(&self) -> String {
        format!("Add npm for `{SCOPE}` to {}", self.file)
    }
}

#[cfg(test)]
mod tests;
