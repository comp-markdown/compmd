use crate::features::code_actions::tests::{actions_of, applied, caret_at};

/// The document `text` with its imports organized; `None` when there is nothing to organize.
fn organized(text: &str) -> Option<String> {
    let actions = actions_of("source.organizeImports", text, caret_at(0, 0));
    let [action] = actions.try_into().ok()?;
    assert_eq!(action.title, "Organize imports");
    Some(applied(text, &action))
}

#[test]
fn imports_are_sorted_by_source_merged_and_unused_names_drop() {
    let text = "---\ntitle: Notes\nmd:\n  import:\n    - from: \"@kit/ui@1\"\n      names: [card]\n    - from: ./local.comp.md\n      names: [note, gone]\n    - from: \"@kit/ui@1\"\n      names: [badge]\n---\n\n::: card\n:: note[x]\n:: badge[y]\n:::\n";
    assert_eq!(
        organized(text).as_deref(),
        Some(
            "---\ntitle: Notes\nmd:\n  import:\n    - from: ./local.comp.md\n      names: [note]\n    - from: \"@kit/ui@1\"\n      names: [badge, card]\n---\n\n::: card\n:: note[x]\n:: badge[y]\n:::\n"
        )
    );
}

#[test]
fn organized_imports_offer_nothing() {
    let text =
        "---\nmd:\n  import:\n    - from: ./a.comp.md\n      names: [card]\n---\n\n:: card[x]\n";
    assert_eq!(organized(text), None);
    assert_eq!(organized("# No frontmatter\n"), None);
    assert_eq!(organized("---\ntitle: No imports\n---\n"), None);
}

#[test]
fn all_names_fonts_renames_and_re_exports_are_kept() {
    let text = "---\nmd:\n  import:\n    - from: ./b.comp.md\n      names: [{name: chart, as: trend}, unused]\n      fonts: [serif]\n    - from: ./a.comp.md\n      names: [card]\n    - from: ./a.comp.md\n      names: \"*\"\n    - from: ./c.comp.md\n      names: [gone]\n    - from: ./d.comp.md\n      names: [shared]\n  export:\n    names: [shared]\n---\n\n:: trend[x]\n";
    assert_eq!(
        organized(text).as_deref(),
        Some(
            "---\nmd:\n  import:\n    - from: ./a.comp.md\n      names: \"*\"\n    - from: ./b.comp.md\n      names:\n        - name: chart\n          as: trend\n      fonts: [serif]\n    - from: ./d.comp.md\n      names: [shared]\n  export:\n    names: [shared]\n---\n\n:: trend[x]\n"
        )
    );
}

#[test]
fn imports_that_differ_in_more_than_names_stay_apart_and_toml_stays_toml() {
    let text = "+++\n[[md.import]]\nfrom = \"@kit/ui@1.0.0\"\nintegrity = \"sha512-a\"\nnames = [\"card\"]\n\n[[md.import]]\nfrom = \"@kit/ui@1.0.0\"\nnames = [\"badge\"]\n\n[[md.import]]\nfrom = \"./b.comp.md\"\nnames = [\"note\"]\n+++\n\n:: card[x]\n:: badge[y]\n:: note[z]\n";
    assert_eq!(
        organized(text).as_deref(),
        Some(
            "+++\n[[md.import]]\nfrom = \"./b.comp.md\"\nnames = [\"note\"]\n\n[[md.import]]\nfrom = \"@kit/ui@1.0.0\"\nintegrity = \"sha512-a\"\nnames = [\"card\"]\n\n[[md.import]]\nfrom = \"@kit/ui@1.0.0\"\nnames = [\"badge\"]\n+++\n\n:: card[x]\n:: badge[y]\n:: note[z]\n"
        )
    );
}

#[test]
fn an_import_of_fonts_takes_the_names_of_another_of_its_source() {
    let text = "---\nmd:\n  import:\n    - from: ./a.comp.md\n      fonts: [serif]\n    - from: ./a.comp.md\n      names: [card]\n---\n\n:: card[x]\n";
    assert_eq!(
        organized(text).as_deref(),
        Some(
            "---\nmd:\n  import:\n    - from: ./a.comp.md\n      fonts: [serif]\n      names: [card]\n---\n\n:: card[x]\n"
        )
    );
}

#[test]
fn what_isnt_an_import_is_left_for_its_diagnostic() {
    let text = "---\nmd:\n  import:\n    - ./a.comp.md\n    - from: ./b.comp.md\n      names: [gone]\n---\n";
    assert_eq!(organized(text), None);
}
