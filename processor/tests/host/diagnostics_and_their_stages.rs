//! Diagnostics: the stage that found each, a host's metadata (used, or ignored with W115), and names from ambient
//! packs (W113 when the host asks for it).

use super::*;
use compmd::diagnostic::{Diagnostic, Stage};
use std::collections::BTreeMap;

#[test]
fn each_diagnostic_says_which_stage_found_it() {
    // One mistake for each stage. The last only evaluating finds: the data is loaded after checking.
    let text = "---\nmd:\n  data:\n    - from: ./missing.json\n      as: missing\n    - from: ./items.json\n      as: items\n---\n\n:: box[B] {title=\"open\n\n:: nope[N]\n\n::: box {border=wiggly}\nX\n::: /box\n\nThe sixth: :value[none]{of=(md.data.items[5])}.\n";
    let host = Host {
        files: HashMap::from([("items.json".to_string(), "[1]".to_string())]),
        ..Host::default()
    };
    let processed = host.process(text);
    let stages: BTreeMap<String, Stage> = processed
        .diagnostics()
        .iter()
        .map(|diagnostic: &Diagnostic| (diagnostic.code.to_string(), diagnostic.stage))
        .collect();
    assert_eq!(
        stages,
        BTreeMap::from([
            ("F011".to_string(), Stage::Parse),
            ("F002".to_string(), Stage::Resolve),
            ("F009".to_string(), Stage::Validate),
            ("F026".to_string(), Stage::Load),
            ("F024".to_string(), Stage::Evaluate),
        ])
    );
}

#[test]
fn a_hosts_metadata_is_a_document_without_frontmatter_s_frontmatter() {
    let host = Host {
        parse: ParseOptions {
            metadata: Some(json!({ "title": "From the host" })),
        },
        ..Host::default()
    };
    let processed = host.process("{{md.doc.title}}\n");
    assert_eq!(diagnostics(&processed), Vec::<String>::new());
    assert_eq!(shown(&processed), "From the host");
}

#[test]
fn a_hosts_metadata_gives_every_md_key_a_frontmatter_would_imports_too() {
    let host = Host {
        parse: ParseOptions {
            metadata: Some(
                json!({ "md": { "import": [{ "from": "@acme/kit@1", "names": "*" }] } }),
            ),
        },
        ..Host::default().with(
            "@acme/kit",
            release("@acme/kit", "1.2.0", &card_pack("A card."), json!({})),
        )
    };
    let processed = host.process(":: card[Card]\n");
    assert_eq!(diagnostics(&processed), Vec::<String>::new());
    assert_eq!(shown(&processed), "A card.");
}

#[test]
fn what_is_wrong_in_a_hosts_metadata_is_reported_at_line_1_since_it_isnt_in_the_text() {
    let host = Host {
        parse: ParseOptions {
            metadata: Some(json!({ "md": { "imports": [] } })),
        },
        ..Host::default()
    };
    let processed = host.process("Text.\n");
    let [unknown] = processed.diagnostics() else {
        panic!("one diagnostic")
    };
    assert_eq!(
        (
            unknown.code.to_string(),
            unknown.range.start.line,
            unknown.range.start.column
        ),
        ("W111".to_string(), 1, 1)
    );
    assert!(unknown.message.contains("did you mean `import`?"));
}

#[test]
fn a_hosts_metadata_for_a_document_with_frontmatter_is_ignored_with_w115() {
    let host = Host {
        parse: ParseOptions {
            metadata: Some(json!({ "title": "From the host" })),
        },
        ..Host::default()
    };
    let processed = host.process("---\ntitle: Its own\n---\n\n{{md.doc.title}}\n");
    assert_eq!(codes(&processed), ["W115"]);
    assert_eq!(shown(&processed), "Its own");
}

#[test]
fn a_name_from_an_ambient_pack_is_used_silently_unless_the_host_asks_for_w113() {
    let ambient = release("@host/c", "1.0.0", &card_pack("A card."), json!({}));
    let host = |report: bool| Host {
        ambient: Ambient {
            packs: vec!["@host/c".into()],
            report,
        },
        ..Host::default().with("@host/c", ambient.clone())
    };
    assert_eq!(
        codes(&host(false).process(":: card[Card]\n")),
        Vec::<String>::new()
    );
    assert_eq!(codes(&host(true).process(":: card[Card]\n")), ["W113"]);
}
