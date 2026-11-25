use compmd::frontmatter::Written;
use lsp_types::CodeAction;

use super::KIND;
use crate::features::code_actions::tests::{actions_of, applied, caret_at};

/// The actions that add keys to `text`, at `line`.
fn offered(text: &str, line: u32) -> Vec<CodeAction> {
    actions_of(KIND, text, caret_at(line, 0))
}

/// `text` with the key `name` added.
fn with_key(text: &str, name: &str) -> String {
    let title = format!("Add `{name}` to the frontmatter");
    let action = offered(text, 1)
        .into_iter()
        .find(|action| action.title == title)
        .unwrap_or_else(|| panic!("{title}"));
    let added = applied(text, &action);
    assert!(Written::of(&added).is_some(), "doesn't read:\n{added}");
    added
}

fn titles(actions: &[CodeAction]) -> Vec<&str> {
    actions.iter().map(|action| action.title.as_str()).collect()
}

#[test]
fn the_common_keys_a_document_lacks_are_offered() {
    let text = "---\ntitle: Notes\ntags: [a]\n---\n";
    assert_eq!(
        titles(&offered(text, 1)),
        [
            "Add `description` to the frontmatter",
            "Add `date` to the frontmatter",
            "Add `md.lang` to the frontmatter",
        ]
    );
    assert_eq!(offered(text, 4), []);
    let complete = "---\ntitle: a\ndescription: b\ndate: c\ntags: []\nmd:\n  lang: de\n---\n";
    assert_eq!(offered(complete, 1), []);
}

#[test]
fn a_key_goes_at_the_top_and_md_lang_under_md() {
    let yaml = "---\ntitle: Notes\nmd:\n    version: 1\n---\n";
    assert_eq!(
        with_key(yaml, "date"),
        "---\ndate: \"\"\ntitle: Notes\nmd:\n    version: 1\n---\n"
    );
    assert_eq!(
        with_key(yaml, "md.lang"),
        "---\ntitle: Notes\nmd:\n    lang: en\n    version: 1\n---\n"
    );
    assert_eq!(
        with_key("---\ntitle: Notes\n---\n", "md.lang"),
        "---\ntitle: Notes\nmd:\n  lang: en\n---\n"
    );
    assert_eq!(
        with_key("---\nmd: {version: 1}\n---\n", "md.lang"),
        "---\nmd:\n  version: 1\n  lang: en\n---\n"
    );
}

#[test]
fn toml_gets_its_own_syntax() {
    assert_eq!(
        with_key("+++\ntitle = \"Notes\"\n+++\n", "tags"),
        "+++\ntags = []\ntitle = \"Notes\"\n+++\n"
    );
    assert_eq!(
        with_key(
            "+++\ntitle = \"Notes\"\n[md]\nversion = 1\n+++\n",
            "md.lang"
        ),
        "+++\ntitle = \"Notes\"\n[md]\nlang = \"en\"\nversion = 1\n+++\n"
    );
    assert_eq!(
        with_key(
            "+++\ntitle = \"Notes\"\n[[md.import]]\nfrom = \"./a.comp.md\"\nnames = [\"a\"]\n+++\n",
            "md.lang"
        ),
        "+++\nmd.lang = \"en\"\ntitle = \"Notes\"\n[[md.import]]\nfrom = \"./a.comp.md\"\nnames = [\"a\"]\n+++\n"
    );
    assert_eq!(
        with_key("+++\nmd = { version = 1 }\n+++\n", "md.lang"),
        "+++\n[md]\nversion = 1\nlang = \"en\"\n+++\n"
    );
}

#[test]
fn a_document_without_a_frontmatter_gets_one_from_its_first_line() {
    let text = "# Notes\n";
    let actions = offered(text, 0);
    assert_eq!(actions.len(), 5);
    assert_eq!(
        applied(text, &actions[0]),
        "---\ntitle: \"\"\n---\n\n# Notes\n"
    );
    assert_eq!(
        applied(text, &actions[4]),
        "---\nmd:\n  lang: en\n---\n\n# Notes\n"
    );
    assert_eq!(offered(text, 1), []);
}
