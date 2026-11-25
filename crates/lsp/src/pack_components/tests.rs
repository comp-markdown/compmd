use std::fs;
use std::path::PathBuf;

use super::*;

/// A project whose configuration lists the folder pack `@site/components`, at `version`, which exports `note`.
fn project_with_notes(name: &str, version: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("compmd-lsp-packs-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("components")).expect("a folder");
    fs::write(
        root.join("compmd.config.json"),
        r#"{"packs": {"./components": {}}}"#,
    )
    .expect("written");
    fs::write(
        root.join("components/package.json"),
        format!(r#"{{"name": "@site/components", "version": "{version}"}}"#),
    )
    .expect("written");
    fs::write(
        root.join("components/index.comp.md"),
        "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=note\n---\nA note.\n```\n",
    )
    .expect("written");
    root
}

#[test]
fn a_packs_components_are_found_with_what_an_import_writes() {
    let root = project_with_notes("found", "1.2.0");
    assert_eq!(
        pack_components(&root),
        [PackComponent {
            name: "note".into(),
            from: "@site/components@1".into()
        }]
    );
    let before_one = project_with_notes("zero", "0.3.1");
    assert_eq!(pack_components(&before_one)[0].from, "@site/components@0.3");
    fs::write(root.join("compmd.config.json"), r#"{"fetch": "proxy"}"#).expect("written");
    assert_eq!(pack_components(&root), [], "a project that can't be read");
}
