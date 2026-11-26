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
        format!(r#"{{"name": "@site/components", "version": "{version}", "description": "The site's own."}}"#),
    )
    .expect("written");
    fs::write(
        root.join("components/index.comp.md"),
        "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=note\n---\nA note.\n```\n",
    )
    .expect("written");
    root
}

/// A suggested source holding `@comp-markdown/badge@1.0.0`, which exports `badge`, with a signature.
fn suggested() -> Box<dyn Packs> {
    let release: compmd::host::LoadedRelease = serde_json::from_value(serde_json::json!({
        "version": "1.0.0",
        "files": {
            "package.json": r#"{"name": "@comp-markdown/badge", "version": "1.0.0", "description": "Badges."}"#,
            "index.comp.md": "---\nmd:\n  export:\n    names: \"*\"\n---\n\n```compmd component=badge\nparams:\n  label: string\n---\n{{label}}\n```\n"
        }
    }))
    .expect("a release");
    Box::new(std::collections::HashMap::from([(
        "@comp-markdown/badge".to_owned(),
        vec![release],
    )]))
}

#[test]
fn a_packs_components_are_found_with_what_an_import_writes() {
    let root = project_with_notes("found", "1.2.0");
    let [note] = pack_components(&root, Box::new(()))
        .try_into()
        .expect("one");
    assert_eq!(
        (
            note.name.as_str(),
            note.from.as_str(),
            note.description.as_deref()
        ),
        ("note", "@site/components@1", Some("The site's own."))
    );
    let before_one = project_with_notes("zero", "0.3.1");
    assert_eq!(
        pack_components(&before_one, Box::new(()))[0].from,
        "@site/components@0.3"
    );
    fs::write(root.join("compmd.config.json"), r#"{"fetch": "proxy"}"#).expect("written");
    assert_eq!(
        pack_components(&root, suggested()),
        [],
        "a project that can't be read"
    );
}

#[test]
fn the_suggested_packs_components_come_after_the_projects_with_their_signatures() {
    let root = project_with_notes("suggested", "1.2.0");
    let found = pack_components(&root, suggested());
    let names: Vec<(&str, &str)> = found
        .iter()
        .map(|component| (component.name.as_str(), component.from.as_str()))
        .collect();
    assert_eq!(
        names,
        [
            ("note", "@site/components@1"),
            ("badge", "@comp-markdown/badge@1")
        ]
    );
    let signature = found[1].signature.as_ref().expect("a signature");
    assert_eq!(signature.params[0].name, "label");
    assert_eq!(found[1].description.as_deref(), Some("Badges."));
}
