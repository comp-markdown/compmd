//! The host-language component a caret is on, for its signature: a definition fence in the document, an import of a
//! component file (`./Chart.jsx`, spec §6.1) in the frontmatter, or the signature fence of either. A component file is
//! read from the disk, next to the document.

use std::path::PathBuf;

use compmd::Language;
use compmd::tree::{DefinitionBody, Node};
use lsp_types::Position;
use serde_json::Value;

use super::super::Asked;
use super::super::frontmatter::Frontmatter;

/// A host-language component whose signature an action writes.
pub(super) struct Component {
    /// The name the document gives it, which its signature fence names.
    pub(super) name: String,
    /// What its code is written in.
    pub(super) language: Language,
    /// Its code.
    pub(super) source: String,
    /// Where a new signature fence for it goes: after its definition fence, or after the frontmatter that imports it.
    pub(super) after: Position,
}

/// The component the caret at `at` is on, with its code; `None` on anything else, or for code that can't be had.
pub(super) fn component_at(at: Position, asked: &Asked) -> Option<Component> {
    let children = &asked.processor?.tree().children;
    let line = at.line as usize + 1;
    let on = |position: Option<compmd::tree::Position>| {
        position.is_some_and(|fence| fence.start.line <= line && line <= fence.end.line)
    };
    for node in children {
        match node {
            Node::ComponentDefinition(definition) if on(definition.position) => {
                return defined(asked, &definition.name);
            }
            Node::ComponentSignature(fence) if on(fence.position) => {
                return defined(asked, &fence.name)
                    .or_else(|| imported(asked, |import| import.name == fence.name));
            }
            _ => {}
        }
    }
    let frontmatter = Frontmatter::of(asked)?;
    let entry = frontmatter
        .key_lines
        .iter()
        .find(|(pointer, key_line)| **key_line == at.line && pointer.starts_with("/md/import/"))
        .and_then(|(pointer, _)| {
            pointer["/md/import/".len()..]
                .split('/')
                .next()?
                .parse::<usize>()
                .ok()
        })?;
    imported(asked, |import| import.index == entry)
}

/// The host-language component the document defines as `name`, after its definition fence.
fn defined(asked: &Asked, name: &str) -> Option<Component> {
    asked
        .processor?
        .tree()
        .children
        .iter()
        .find_map(|node| match node {
            Node::ComponentDefinition(definition) if definition.name == name => {
                match &definition.body {
                    DefinitionBody::Host { source, .. } => Some(Component {
                        name: name.to_owned(),
                        language: Language::from_name(&definition.language)?,
                        source: source.clone(),
                        after: asked.encoding.span(asked.text, &definition.position?).end,
                    }),
                    DefinitionBody::Compmd { .. } => None,
                }
            }
            _ => None,
        })
}

/// An import of a component file, as the frontmatter writes it.
struct FileImport {
    /// Its place among the imports.
    index: usize,
    /// The local name its component gets.
    name: String,
    /// Its file's language.
    language: Language,
    /// Its `from`.
    from: String,
}

/// The component of the first component file import `wanted` takes, read from its file, after the frontmatter.
fn imported(asked: &Asked, wanted: impl Fn(&FileImport) -> bool) -> Option<Component> {
    let frontmatter = Frontmatter::of(asked)?;
    let imports = frontmatter.value.get("md")?.get("import")?.as_array()?;
    let import = imports
        .iter()
        .enumerate()
        .filter_map(|(index, import)| file_import(index, import))
        .find(|import| wanted(import))?;
    let source = std::fs::read_to_string(beside_document(asked, &import.from)?).ok()?;
    Some(Component {
        name: import.name,
        language: import.language,
        source,
        after: frontmatter.end(),
    })
}

/// `import`, the `index`th, when it takes the one component of a JSX, TSX or Svelte file (spec §6.1): its default
/// export, named after the file's stem, under the local name `names` gives it.
fn file_import(index: usize, import: &Value) -> Option<FileImport> {
    let from = import.get("from")?.as_str()?;
    let (stem, extension) = from.rsplit('/').next()?.rsplit_once('.')?;
    let language = match extension {
        "jsx" | "tsx" | "svelte" => Language::from_name(extension)?,
        _ => return None,
    };
    let name = match import.get("names")? {
        Value::String(all) if all == "*" => stem.to_owned(),
        Value::Array(entries) => entries.iter().find_map(|entry| match entry {
            Value::String(name) if name == stem => Some(name.clone()),
            Value::Object(renamed) if renamed.get("name").and_then(Value::as_str) == Some(stem) => {
                Some(
                    renamed
                        .get("as")
                        .and_then(Value::as_str)
                        .unwrap_or(stem)
                        .to_owned(),
                )
            }
            _ => None,
        })?,
        _ => return None,
    };
    Some(FileImport {
        index,
        name,
        language,
        from: from.to_owned(),
    })
}

/// The path of `from`, relative to the document `asked` is about.
fn beside_document(asked: &Asked, from: &str) -> Option<PathBuf> {
    let document = url::Url::parse(asked.uri.as_str())
        .ok()?
        .to_file_path()
        .ok()?;
    Some(document.parent()?.join(from.trim_start_matches("./")))
}

#[cfg(test)]
mod tests;
