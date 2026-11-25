//! `source.organizeImports`: `md.import` sorted by source, the imports of one source merged, their names sorted by
//! the local name and those the document uses nowhere dropped (spec §6.1), written in the frontmatter's syntax.
//!
//! A name is used when the document invokes it or exports it (§6.2, a re-export). Fonts are merged and sorted, never
//! dropped. An import left with neither names nor fonts goes; imports of one source that differ in anything else (an
//! `integrity`) stay apart.

use std::collections::HashSet;

use compmd::tokens::TokenKind;
use lsp_types::{CodeAction, CodeActionKind};
use serde_json::{Map, Value};

use super::frontmatter::Frontmatter;
use super::{Asked, editing};

/// The action that organizes the imports, when they aren't organized yet.
pub(super) fn organize(asked: &Asked) -> Option<CodeAction> {
    let frontmatter = Frontmatter::of(asked)?;
    let imports = frontmatter.value.get("md")?.get("import")?.as_array()?;
    let used = used_names(asked, &frontmatter.value);
    let organized = organized(imports, &used);
    if &organized == imports {
        return None;
    }
    let mut value = frontmatter.value.clone();
    value
        .get_mut("md")
        .and_then(Value::as_object_mut)?
        .insert("import".into(), Value::Array(organized));
    let edit = frontmatter.rewritten(frontmatter.syntax, &value).ok()?;
    Some(CodeAction {
        title: "Organize imports".into(),
        kind: Some(CodeActionKind::SOURCE_ORGANIZE_IMPORTS),
        edit: Some(editing(asked, vec![edit])),
        ..CodeAction::default()
    })
}

/// The names the document invokes, and those its `md.export` lists.
fn used_names(asked: &Asked, frontmatter: &Map<String, Value>) -> HashSet<String> {
    let lines: Vec<&str> = asked.text.split('\n').collect();
    let mut used: HashSet<String> = asked
        .processor
        .into_iter()
        .flat_map(compmd::Processor::tokens)
        .filter(|token| token.kind == TokenKind::ComponentName)
        .filter_map(|token| {
            let line = lines.get(token.range.start.line.checked_sub(1)?)?;
            let start = token.range.start.column.checked_sub(1)?;
            let length = token.range.end?.column.checked_sub(start + 1)?;
            Some(line.chars().skip(start).take(length).collect())
        })
        .collect();
    let exported = frontmatter
        .get("md")
        .and_then(|md| md.get("export"))
        .and_then(|export| export.get("names"))
        .and_then(Value::as_array);
    used.extend(exported.into_iter().flatten().filter_map(|entry| {
        entry
            .as_str()
            .or_else(|| entry.get("name").and_then(Value::as_str))
            .map(str::to_owned)
    }));
    used
}

/// `imports` organized, keeping the names `used` holds.
fn organized(imports: &[Value], used: &HashSet<String>) -> Vec<Value> {
    let mut merged: Vec<Map<String, Value>> = Vec::new();
    for import in imports {
        let Value::Object(import) = import else {
            // What isn't an import is left for its diagnostic, where it is.
            return imports.to_vec();
        };
        match merged.iter_mut().find(|kept| same_source(kept, import)) {
            Some(kept) => {
                for key in ["names", "fonts"] {
                    merge_names(kept, import, key);
                }
            }
            None => merged.push(import.clone()),
        }
    }
    for import in &mut merged {
        if let Some(Value::Array(names)) = import.get_mut("names") {
            names.retain(|entry| local_name(entry).is_some_and(|name| used.contains(name)));
            if names.is_empty() {
                import.remove("names");
            }
        }
        for key in ["names", "fonts"] {
            if let Some(Value::Array(names)) = import.get_mut(key) {
                names.sort_by(|a, b| local_name(a).cmp(&local_name(b)));
                names.dedup();
            }
        }
    }
    merged.retain(|import| import.contains_key("names") || import.contains_key("fonts"));
    merged.sort_by(|a, b| from(a).cmp(from(b)));
    merged.into_iter().map(Value::Object).collect()
}

/// Whether two imports take from one source alike: the same `from`, and the same in all but their names and fonts.
fn same_source(a: &Map<String, Value>, b: &Map<String, Value>) -> bool {
    let rest = |import: &Map<String, Value>| {
        import
            .iter()
            .filter(|(key, _)| !matches!(key.as_str(), "names" | "fonts"))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Map<String, Value>>()
    };
    rest(a) == rest(b)
}

/// `kept`'s names (or fonts, by `key`) with `import`'s: `"*"` when either takes all.
fn merge_names(kept: &mut Map<String, Value>, import: &Map<String, Value>, key: &str) {
    let Some(more) = import.get(key) else {
        return;
    };
    let merged = match (kept.get(key), more) {
        (None, more) => more.clone(),
        (Some(Value::Array(names)), Value::Array(more)) => {
            Value::Array(names.iter().chain(more).cloned().collect())
        }
        _ => Value::String("*".into()),
    };
    kept.insert(key.into(), merged);
}

/// The local name an entry of `names` gives: the string, or its `as`, else its `name`.
fn local_name(entry: &Value) -> Option<&str> {
    entry.as_str().or_else(|| {
        entry
            .get("as")
            .or_else(|| entry.get("name"))
            .and_then(Value::as_str)
    })
}

/// An import's `from`.
fn from(import: &Map<String, Value>) -> &str {
    import
        .get("from")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests;
