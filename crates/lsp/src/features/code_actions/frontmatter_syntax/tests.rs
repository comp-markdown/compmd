use lsp_types::CodeAction;

use crate::features::code_actions::tests::{actions_of, applied, caret_at};

/// The action that writes the frontmatter of `text` in the other syntax, at `line`.
fn converting(text: &str, line: u32) -> Option<CodeAction> {
    let [action] = actions_of("refactor.rewrite", text, caret_at(line, 0))
        .try_into()
        .ok()?;
    Some(action)
}

#[test]
fn yaml_becomes_toml_and_back_with_its_comments() {
    let yaml = "---\n# The post's\ntitle: Release notes\nmd:\n  # CompMD's\n  import:\n    - from: \"@myorg/ui@1\"\n      names: [card, callout]\n  lang: en\n---\n\n# Release notes\n";
    let to_toml = converting(yaml, 1).expect("in the frontmatter");
    assert_eq!(to_toml.title, "Write the frontmatter in TOML");
    let toml = applied(yaml, &to_toml);
    assert_eq!(
        toml,
        "+++\n# The post's\ntitle = \"Release notes\"\n\n[md]\nlang = \"en\"\n\n# CompMD's\n[[md.import]]\nfrom = \"@myorg/ui@1\"\nnames = [\"card\", \"callout\"]\n+++\n\n# Release notes\n"
    );
    let to_yaml = converting(&toml, 1).expect("in the frontmatter");
    assert_eq!(to_yaml.title, "Write the frontmatter in YAML");
    assert_eq!(
        applied(&toml, &to_yaml),
        "---\n# The post's\ntitle: Release notes\nmd:\n  lang: en\n  # CompMD's\n  import:\n    - from: \"@myorg/ui@1\"\n      names: [card, callout]\n---\n\n# Release notes\n"
    );
}

#[test]
fn only_in_the_frontmatter() {
    let text = "---\ntitle: Notes\n---\n\n# Notes\n";
    assert!(converting(text, 4).is_none());
    assert!(converting("# Notes\n", 0).is_none());
}

#[test]
fn an_empty_value_makes_it_disabled_saying_which() {
    let text = "---\ntitle: Notes\nmd:\n  tz:\n---\n";
    let action = converting(text, 1).expect("offered");
    assert_eq!(action.edit, None);
    assert_eq!(
        action.disabled.map(|disabled| disabled.reason).as_deref(),
        Some("TOML has no value for `md.tz`, which is empty or too large")
    );
}
