//! Edits that rerun the whole document, an edit that breaks the frontmatter, and a reload with data fetched after
//! an edit asked for it.

use super::*;
use compmd::TextEdit;
use compmd::host::Fetched;

#[test]
fn an_edit_to_the_frontmatter_expands_the_whole_document_again() {
    let text = "---\ntitle: One\n---\n\n# {{md.doc.title}}\n\nText.\n";
    let processed = Host::default().process(text);
    let start = text.find("One").expect("the title");
    let edited = processed.edit(&TextEdit {
        start,
        end: start + 3,
        text: "Two".into(),
    });
    let splice = edited.splice();
    assert_eq!((splice.from, splice.to, splice.nodes.len()), (0, 2, 2));
    assert_eq!(shown(&edited), "TwoText.");
}

#[test]
fn an_edit_that_breaks_the_frontmatter_is_f033_and_the_fix_clears_it() {
    let text = "---\ntitle: One\n---\n\nText.\n";
    let processed = Host::default().process(text);
    let start = text.find("One").expect("the title");
    let broken = processed.edit(&TextEdit {
        start,
        end: start,
        text: "&anchor ".into(),
    });
    assert_eq!(codes(&broken), ["F033"]);
    let fixed = broken.edit(&TextEdit {
        start,
        end: start + "&anchor ".len(),
        text: String::new(),
    });
    assert_eq!(codes(&fixed), Vec::<String>::new());
}

#[test]
fn after_an_edit_adds_a_data_import_sources_lists_it_and_a_reload_fills_it_in() {
    let host = Host {
        allowed_origins: vec!["https://a.org".into()],
        ..Host::default()
    };
    let processed = host.process("Text.\n");
    let frontmatter = "---\nmd:\n  data:\n    - from: https://a.org/x.json\n      as: x\n---\n\n{{md.data.x.n}}\n\n";
    let edited = processed.edit(&TextEdit {
        start: 0,
        end: 0,
        text: frontmatter.into(),
    });
    assert_eq!(edited.sources().remote, ["https://a.org/x.json"]);
    // Not fetched yet: the import is F026, and its interpolation, of `null`, F024.
    assert_eq!(codes(&edited), ["F026", "F024"]);
    let fetched = HashMap::from([(
        "https://a.org/x.json".to_string(),
        Fetched {
            text: "{\"n\": 7}".into(),
            content_type: Some("application/json".into()),
        },
    )]);
    let reloaded = edited.reload(host.context.clone(), fetched, ());
    assert_eq!(codes(&reloaded), Vec::<String>::new());
    assert_eq!(shown(&reloaded), "7Text.");
}
