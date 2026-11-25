use super::*;
use crate::features::testing::processed;

/// `text` with the import of `note` from `@kit/notes@1` added.
fn imported(text: &str) -> Option<String> {
    let edit = adding_import(text, processed(text).tree(), "@kit/notes@1", "note")?;
    let mut lines: Vec<&str> = text.split('\n').collect();
    let line = edit.range.start.line as usize;
    let inserted = edit
        .new_text
        .strip_suffix('\n')
        .unwrap_or(&edit.new_text)
        .to_owned();
    lines.insert(line, &inserted);
    Some(lines.join("\n"))
}

const ITEM: &str = "- from: \"@kit/notes@1\"\n";

#[test]
fn a_document_without_frontmatter_gets_one() {
    assert_eq!(
        imported("# A\n").as_deref(),
        Some(
            "---\nmd:\n  import:\n    - from: \"@kit/notes@1\"\n      names: [note]\n---\n\n# A\n"
        )
    );
}

#[test]
fn yaml_gets_md_import_or_an_item_where_it_is() {
    let no_md = imported("---\ntitle: A\n---\n# A\n").expect("an edit");
    assert_eq!(
        no_md,
        format!("---\ntitle: A\nmd:\n  import:\n    {ITEM}      names: [note]\n---\n# A\n")
    );
    let no_import = imported("---\nmd:\n    version: 1\n---\n").expect("an edit");
    assert_eq!(
        no_import,
        format!("---\nmd:\n    import:\n      {ITEM}        names: [note]\n    version: 1\n---\n")
    );
    let listed = "---\nmd:\n  import:\n  - from: ./a.comp.md\n    names: \"*\"\n\n  lang: en\ntitle: A\n---\n";
    assert_eq!(
        imported(listed).expect("an edit"),
        format!(
            "---\nmd:\n  import:\n  - from: ./a.comp.md\n    names: \"*\"\n  {ITEM}    names: [note]\n\n  lang: en\ntitle: A\n---\n"
        )
    );
    assert_eq!(
        imported("---\nmd: {lang: en}\n---\n"),
        None,
        "md on one line"
    );
    assert_eq!(
        imported("---\nmd:\n  import: []\n---\n"),
        None,
        "md.import on one line"
    );
}

#[test]
fn toml_gets_a_table() {
    assert_eq!(
        imported("+++\ntitle = \"A\"\n+++\n").as_deref(),
        Some(
            "+++\ntitle = \"A\"\n[[md.import]]\nfrom = \"@kit/notes@1\"\nnames = [\"note\"]\n+++\n"
        )
    );
}
