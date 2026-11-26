use compmd::Language;
use serde_json::json;

use super::file_import;

/// The local name and language `import` gives its component file's component.
fn taken(import: &serde_json::Value) -> Option<(String, Language)> {
    file_import(0, import).map(|import| (import.name, import.language))
}

#[test]
fn a_component_file_import_names_its_component_by_the_stem_or_its_as() {
    assert_eq!(
        taken(&json!({ "from": "./parts/Chart.tsx", "names": "*" })),
        Some(("Chart".into(), Language::Tsx))
    );
    assert_eq!(
        taken(&json!({ "from": "./Badge.jsx", "names": ["Badge"] })),
        Some(("Badge".into(), Language::Jsx))
    );
    assert_eq!(
        taken(&json!({ "from": "./Card.svelte", "names": [{ "name": "Card", "as": "card" }] })),
        Some(("card".into(), Language::Svelte))
    );
    assert_eq!(
        taken(&json!({ "from": "./Card.svelte", "names": [{ "name": "Card" }] })),
        Some(("Card".into(), Language::Svelte))
    );
}

#[test]
fn anything_else_is_no_component_file_import() {
    assert_eq!(
        taken(&json!({ "from": "./ui.comp.md", "names": "*" })),
        None
    );
    assert_eq!(taken(&json!({ "from": "./mount.js", "names": "*" })), None);
    assert_eq!(
        taken(&json!({ "from": "./Chart.tsx", "names": ["Other"] })),
        None
    );
    assert_eq!(
        taken(&json!({ "from": "./Chart.tsx", "fonts": ["serif"] })),
        None
    );
    assert_eq!(taken(&json!({ "from": "./Chart.tsx", "names": 3 })), None);
    assert_eq!(taken(&json!({ "from": "noextension", "names": "*" })), None);
}
