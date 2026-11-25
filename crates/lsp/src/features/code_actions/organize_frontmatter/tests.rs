use super::KIND;
use crate::features::code_actions::tests::{actions_of, applied, caret_at};

/// The document `text` with its frontmatter organized; `None` when there is nothing to organize.
fn organized(text: &str) -> Option<String> {
    let [action] = actions_of(KIND, text, caret_at(5, 0)).try_into().ok()?;
    assert_eq!(action.title, "Organize the frontmatter");
    Some(applied(text, &action))
}

#[test]
fn the_authors_keys_come_first_then_md_in_the_specs_order() {
    let text = "---\nmd:\n  lang: en\n  # its own\n  import:\n    - from: ./a.comp.md\n      names: [card]\n  unknown: 1\n  version: 1\ntitle: Release notes\nversion: \"1.4\"\n---\n\n:: card[x]\n";
    assert_eq!(
        organized(text).as_deref(),
        Some(
            "---\ntitle: Release notes\nversion: \"1.4\"\nmd:\n  version: 1\n  # its own\n  import:\n    - from: ./a.comp.md\n      names: [card]\n  lang: en\n  unknown: 1\n---\n\n:: card[x]\n"
        )
    );
}

#[test]
fn keys_in_order_offer_nothing() {
    assert_eq!(
        organized("---\ntitle: Notes\nmd:\n  version: 1\n  lang: de\n---\n"),
        None
    );
    assert_eq!(
        organized("---\nmd: 1\ntitle: x\n---\n").as_deref(),
        Some("---\ntitle: x\nmd: 1\n---\n")
    );
    assert_eq!(organized("# No frontmatter\n"), None);
}
