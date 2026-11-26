//! npm's search, for a scope's packs ([packs from a registry], R6; [the language server], decision 7): the packs of a
//! scope that carry the keyword `compmd-pack`, which only packs use. An editor lists them to suggest them.
//!
//! [packs from a registry]: https://github.com/comp-markdown/compmd/blob/main/docs/packs-from-a-registry.md
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md

use compmd_pack::PackName;
use serde_json::Value;

/// The keyword a pack's `package.json` lists, by which a registry's search finds packs.
pub const PACK_KEYWORD: &str = "compmd-pack";

/// The most packs one search lists: npm's own limit.
const MOST: usize = 250;

/// The path of the search, under a registry's URL, for the packs of `scope` (`@kit`).
#[must_use]
pub fn search_path(scope: &str) -> String {
    let scope = scope.trim_start_matches('@');
    format!("-/v1/search?text=scope:{scope}%20keywords:{PACK_KEYWORD}&size={MOST}")
}

/// The packs of `scope` that `body`, a registry's answer to [`search_path`], lists, in its order: a name of another
/// scope, or that isn't a pack's, is left out.
///
/// # Errors
///
/// When `body` isn't a search's answer.
pub fn searched(body: &str, scope: &str) -> Result<Vec<PackName>, String> {
    let answer: Value = serde_json::from_str(body)
        .map_err(|error| format!("the search's answer isn't JSON: {error}"))?;
    let objects = answer["objects"]
        .as_array()
        .ok_or_else(|| "the search's answer has no `objects`".to_owned())?;
    let prefix = format!("{scope}/");
    Ok(objects
        .iter()
        .filter_map(|object| object["package"]["name"].as_str())
        .filter(|name| name.starts_with(&prefix))
        .filter_map(|name| name.parse().ok())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scopes_search_asks_for_its_packs_by_their_keyword() {
        assert_eq!(
            search_path("@kit"),
            "-/v1/search?text=scope:kit%20keywords:compmd-pack&size=250"
        );
    }

    #[test]
    fn the_answer_gives_the_scopes_packs_in_its_order() {
        let body = r#"{"objects": [
            {"package": {"name": "@kit/timeline", "version": "1.0.0"}},
            {"package": {"name": "@other/maps"}},
            {"package": {"name": "@kit/Bad"}},
            {"package": {"name": "@kit/cards"}},
            {"score": 1}
        ], "total": 4}"#;
        let names: Vec<String> = searched(body, "@kit")
            .expect("packs")
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(names, ["@kit/timeline", "@kit/cards"]);
        assert!(searched("{}", "@kit").is_err());
        assert!(searched("not json", "@kit").is_err());
    }
}
