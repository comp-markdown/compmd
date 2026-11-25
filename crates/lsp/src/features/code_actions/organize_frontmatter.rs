//! `source.organizeFrontmatter.compmd`: the frontmatter's keys in a canonical order, the author's first as they are
//! written, then `md` with its keys in the order spec §2.1 lists them (a key it doesn't list last, as written).

use lsp_types::{CodeAction, CodeActionKind};
use serde_json::{Map, Value};

use super::frontmatter::Frontmatter;
use super::{Asked, editing};

/// The kind of the action.
pub(crate) const KIND: &str = "source.organizeFrontmatter.compmd";

/// The action that puts the frontmatter's keys in order, when they aren't in order yet.
pub(super) fn organize(asked: &Asked) -> Option<CodeAction> {
    let frontmatter = Frontmatter::of(asked)?;
    let ordered = ordered(&frontmatter.value);
    let keys = |value: &Map<String, Value>| -> Vec<String> {
        let md = value.get("md").and_then(Value::as_object);
        value
            .keys()
            .chain(md.into_iter().flat_map(Map::keys))
            .cloned()
            .collect()
    };
    if keys(&ordered) == keys(&frontmatter.value) {
        return None;
    }
    let edit = frontmatter.rewritten(frontmatter.syntax, &ordered).ok()?;
    Some(CodeAction {
        title: "Organize the frontmatter".into(),
        kind: Some(CodeActionKind::new(KIND)),
        edit: Some(editing(asked, vec![edit])),
        ..CodeAction::default()
    })
}

/// `value`'s keys in order.
fn ordered(value: &Map<String, Value>) -> Map<String, Value> {
    let mut ordered: Map<String, Value> = value
        .iter()
        .filter(|(key, _)| *key != "md")
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    match value.get("md") {
        Some(Value::Object(md)) => {
            let rank = |key: &str| {
                compmd::frontmatter::md_keys()
                    .position(|known| known == key)
                    .unwrap_or(usize::MAX)
            };
            let mut entries: Vec<(&String, &Value)> = md.iter().collect();
            // A stable sort: keys it doesn't list stay in written order.
            entries.sort_by_key(|(key, _)| rank(key));
            let md = entries
                .into_iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
            ordered.insert("md".into(), Value::Object(md));
        }
        Some(other) => {
            ordered.insert("md".into(), other.clone());
        }
        None => {}
    }
    ordered
}

#[cfg(test)]
mod tests;
