//! Editing an expanded processor: after every edit, it gives the trees and diagnostics a new processor of the new
//! text gives, through every stage.

use compmd::context::Context;
use compmd::expand::Options;
use compmd::{Ambient, ParseOptions, Processor, TextEdit};
use std::collections::HashMap;
use std::path::Path;

/// The complete example's folder.
fn example() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/complete")
}

/// The files of the complete example, by name.
fn files() -> HashMap<String, String> {
    std::fs::read_dir(example())
        .expect("the example")
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            std::fs::read_to_string(entry.path())
                .ok()
                .map(|text| (name, text))
        })
        .collect()
}

/// A small, seeded random number generator (xorshift), so a failure repeats.
struct Random(u64);

impl Random {
    fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        usize::try_from(self.0 % bound as u64).unwrap_or(0)
    }
}

const TYPED: &[&str] = &[
    "a",
    " ",
    "\n",
    "\n\n",
    "::",
    "{{md.doc.title}}",
    ":badge[x]",
    "[",
    "`",
    "\n\n::: card\nBody.\n::: /card\n\n",
    "\n\n:: chart[Installs]\n\n",
];

#[test]
fn an_edited_processor_gives_what_a_new_one_gives() {
    let files = files();
    let context = Context {
        now: 0,
        tz: "UTC".into(),
        time: &(),
        inputs: serde_json::Map::default(),
    };
    let process = |text: &str| {
        Processor::new("release.comp.md", text)
            .parse(&ParseOptions::default())
            .resolve(&files, &(), &Ambient::default())
            .validate(&[])
            .load(&context, &(), &())
            .expand(&Options::default())
    };
    let mut text = files["release.comp.md"].clone();
    let mut processor = process(&text);
    let mut random = Random(0x2545_f491_4f6c_dd1d);
    for _ in 0..60 {
        let length = text.chars().count();
        let start = random.below(length + 1);
        let end = (start + random.below(4)).min(length);
        let typed = TYPED[random.below(TYPED.len())];
        let edit = TextEdit {
            start,
            end,
            text: if random.below(3) == 0 {
                String::new()
            } else {
                typed.to_string()
            },
        };
        let chars: Vec<char> = text.chars().collect();
        text = chars[..start]
            .iter()
            .chain(edit.text.chars().collect::<Vec<_>>().iter())
            .chain(chars[end..].iter())
            .collect();
        processor = processor.edit(&edit);
        let fresh = process(&text);
        assert_eq!(processor.tree(), fresh.tree(), "the tree after {edit:?}");
        assert_eq!(
            processor.diagnostics(),
            fresh.diagnostics(),
            "the diagnostics after {edit:?}"
        );
        assert_eq!(
            serde_json::to_value(processor.expanded()).unwrap(),
            serde_json::to_value(fresh.expanded()).unwrap(),
            "the expanded tree after {edit:?}"
        );
    }
}

#[test]
fn an_edit_past_the_end_appends() {
    let context = Context {
        now: 0,
        tz: "UTC".into(),
        time: &(),
        inputs: serde_json::Map::default(),
    };
    let files: HashMap<String, String> = HashMap::new();
    let processor = Processor::new("a.comp.md", "\u{feff}One.\n")
        .parse(&ParseOptions::default())
        .resolve(&files, &(), &Ambient::default())
        .validate(&[])
        .load(&context, &(), &())
        .expand(&Options::default())
        .edit(&TextEdit {
            start: 99,
            end: 50,
            text: "\nTwo.\n".into(),
        });
    let json = serde_json::to_value(processor.tree()).unwrap();
    assert_eq!(json["children"][1]["children"][0]["value"], "Two.");
}
