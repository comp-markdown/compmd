use super::*;
use std::fmt::Write;
use std::path::Path;

/// A small, seeded random number generator (xorshift), so a failure repeats.
struct Random(u64);

impl Random {
    fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        usize::try_from(self.0 % bound as u64).unwrap_or(0)
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len())]
    }
}

/// What edits type and paste: characters with a meaning in Markdown or CompMD, and whole blocks.
const TYPED: &[&str] = &[
    "a", " ", "\n", "\n\n", "#", "-", "*", ":", "::", "[", "]", "`", ">", "|", "{", "}", "^", "$",
    "=", "+", "!", "\t", "\r\n", "é", "😀", "  ", "1.",
];
const PASTED: &[&str] = &[
    "\n\n::: card {title=\"x\"}\nBody.\n::: /card\n\n",
    "::: card\n",
    "::: /card\n",
    "\n```\n",
    "```compmd component=note\n---\nSee [l] and :badge[b].\n```\n",
    "\n[l]: /x\n",
    "\n[^1]: A note.\n",
    "See [l] and[^1].",
    ":: chart[Weekly [l]] {rows=(r)}\n",
    "::: + gap=s\n",
    ":: #foot\n",
    "\n- a\n- b\n\n  c\n",
    "> q\n> [!NOTE]\n",
    "\n---\n",
    "| a | b |\n|---|---|\n| 1 | 2 |\n",
    "$$\nx\n$$\n",
    "<div>\n:: x\n</div>\n",
    "{{md.doc.title}}",
    "Title\n===\n",
    "    code\n",
];

fn random_edit(text: &str, random: &mut Random) -> TextEdit {
    let length = text.chars().count();
    let start = random.below(length + 1);
    let (end, text) = match random.below(6) {
        0 | 1 => (start, random.pick(TYPED).to_string()),
        2 => (start, random.pick(PASTED).to_string()),
        3 => ((start + 1 + random.below(3)).min(length), String::new()),
        4 => (
            (start + random.below(40)).min(length),
            random.pick(TYPED).to_string(),
        ),
        _ => (
            (start + random.below(200)).min(length),
            random.pick(PASTED).to_string(),
        ),
    };
    TextEdit { start, end, text }
}

/// Folders of what is installed or built, not written: an example app's packages and its build.
const SKIPPED: [&str; 3] = ["node_modules", "target", "dist"];

/// The documents of the conformance set and the examples.
fn documents() -> Vec<(String, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut found = Vec::new();
    let mut folders = vec![root.join("conformance"), root.join("examples")];
    while let Some(folder) = folders.pop() {
        let entries = std::fs::read_dir(&folder).expect("a folder");
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if !SKIPPED.iter().any(|skipped| entry.file_name() == *skipped) {
                    folders.push(path);
                }
            } else if path.to_string_lossy().ends_with(".comp.md") {
                let text = std::fs::read_to_string(&path).expect("a document");
                found.push((path.display().to_string(), text));
            }
        }
    }
    found.sort();
    found
}

/// The units of `parsed`, each with its text from `text`.
fn units_of(parsed: &Parsed, text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut written = String::new();
    for (index, unit) in parsed.units.iter().enumerate() {
        let _ = writeln!(
            written,
            "  {index}: {}..{} {:?}, {} blocks, {} diagnostics",
            unit.start.offset,
            unit.end.offset,
            chars[unit.start.offset..unit.end.offset]
                .iter()
                .collect::<String>(),
            unit.nodes,
            unit.diagnostics
        );
    }
    written
}

/// `text` after `edit`, parsed again from `previous`, the parse of `text`.
fn reparsed(
    previous: Parsed,
    text: &str,
    edit: &TextEdit,
    options: &ParseOptions,
) -> (String, Parsed, Change) {
    let mut lines = LineStarts::new(text);
    let reparsed = reparse(previous, text, &mut lines, edit, options);
    assert_eq!(lines, LineStarts::new(&reparsed.0), "the line starts");
    reparsed
}

/// How `edit` of `text`, parsed as `previous`, parses incrementally: the units before, each slice it tried and
/// what came of it, and the units a parse of the whole new text gives.
fn explain(previous: &Parsed, text: &str, edit: &TextEdit, options: &ParseOptions) -> String {
    let lines = LineStarts::new(text);
    let (new_text, bom, applied) = apply(text, &lines, edit);
    let mut tried = Vec::new();
    let outcome = Slicer {
        previous,
        old: &text[bom..],
        new: &new_text[bom..],
        lines: &lines,
        edit: applied,
    }
    .parse(Some(&mut tried));
    let mut slices = String::new();
    for tried in &tried {
        let _ = writeln!(
            slices,
            "  units {}..={}: {}",
            tried.first, tried.last, tried.outcome
        );
    }
    format!(
        "the units before:\n{}the slices tried:\n{slices}{}\nthe units of the whole new text:\n{}",
        units_of(previous, text),
        if outcome.is_none() {
            "  parsed the whole text\n"
        } else {
            ""
        },
        units_of(&parse(&new_text, options), &new_text),
    )
}

/// Why `change`, from `previous` to `parsed`, is wrong: a block outside its changed blocks isn't as it was, or
/// moved.
fn wrong_blocks(previous: &Parsed, parsed: &Parsed, change: &Change) -> Option<String> {
    let Change::Blocks(spliced) = change else {
        return None;
    };
    let (old, new) = (&previous.root.children, &parsed.root.children);
    if old.len() - spliced.blocks.len() != new.len() - spliced.inserted {
        return Some(format!("{spliced:?} doesn't count the blocks"));
    }
    if old[..spliced.blocks.start] != new[..spliced.blocks.start] {
        return Some(format!("a block before {spliced:?} changed"));
    }
    let after = &old[spliced.blocks.end..];
    if after.iter().any(|block| {
        !block
            .position()
            .is_some_and(|at| spliced.moved.moves(at.start))
    }) {
        return Some(format!(
            "a block after {spliced:?} isn't after where it moves"
        ));
    }
    let mut moved = after.to_vec();
    moved.shift(spliced.moved.by);
    (moved[..] != new[spliced.blocks.start + spliced.inserted..])
        .then(|| format!("a block after {spliced:?} didn't move"))
}

/// After every edit, the parse after an edit equals the parse of the whole new text: the tree, the diagnostics
/// and the units; and what it says changed did.
#[test]
fn an_edit_parses_as_the_whole_new_text_does() {
    let options = ParseOptions::default();
    let documents = documents();
    assert!(documents.len() > 100, "the conformance set");
    for (index, (path, text)) in documents.into_iter().enumerate() {
        let mut random = Random(0x9e37_79b9_7f4a_7c15 ^ (index as u64 + 1));
        let mut text = text;
        let mut parsed = parse(&text, &options);
        for _ in 0..40 {
            let edit = random_edit(&text, &mut random);
            let (new_text, incremental, change) = reparsed(parsed.clone(), &text, &edit, &options);
            let whole = parse(&new_text, &options);
            let first_difference = incremental
                .root
                .children
                .iter()
                .zip(&whole.root.children)
                .position(|(one, other)| one != other);
            let context = || {
                format!(
                    "{path}\nthe text {text:?}\nthe edit {edit:?}\nthe new text {new_text:?}\n{}the first block that differs: {first_difference:?}, {:#?} and {:#?}",
                    explain(&parsed, &text, &edit, &options),
                    first_difference.map(|index| &incremental.root.children[index]),
                    first_difference.map(|index| &whole.root.children[index]),
                )
            };
            assert!(incremental.root == whole.root, "the tree: {}", context());
            assert!(
                incremental.diagnostics == whole.diagnostics,
                "the diagnostics {:#?} and {:#?}: {}",
                incremental.diagnostics,
                whole.diagnostics,
                context()
            );
            assert!(
                incremental.units == whole.units,
                "the units {:#?} and {:#?}: {}",
                incremental.units,
                whole.units,
                context()
            );
            if let Some(wrong) = wrong_blocks(&parsed, &incremental, &change) {
                panic!("{wrong}: {}", context());
            }
            text = new_text;
            parsed = incremental;
        }
    }
}

#[test]
fn an_edit_says_which_blocks_changed() {
    let options = ParseOptions::default();
    let text = "One.\n\nTwo.\n\n```compmd component=note\n---\nHi.\n```\n\nThree.\n\nSee [l].\n";
    let change = |start, inserted: &str| {
        let edit = TextEdit {
            start,
            end: start,
            text: inserted.into(),
        };
        reparsed(parse(text, &options), text, &edit, &options).2
    };
    let Change::Blocks(typed) = change(9, "!") else {
        panic!("a block changed");
    };
    assert_eq!((typed.blocks, typed.inserted), (1..2, 1));
    assert_eq!(typed.moved.from.offset, 12, "from the definition on");
    assert_eq!(
        typed.moved.by,
        Move {
            lines: 0,
            offset: 1
        }
    );
    assert_eq!(change(43, "!"), Change::Whole, "in a definition");
    assert_eq!(
        change(0, "---\na: 1\n---\n\n"),
        Change::Whole,
        "frontmatter"
    );
    let Change::Blocks(defined) = change(50, "[l]: /y\n\n") else {
        panic!("blocks changed");
    };
    assert_eq!(
        (defined.blocks, defined.inserted, defined.moved),
        (3..5, 3, Moved::NOTHING),
        "the definition, and the reference it makes a link"
    );
}

#[test]
fn an_explanation_gives_the_units_and_the_slices_tried() {
    let options = ParseOptions::default();
    let text = "One.\n\nTwo.\n\n```\nThree.\n```\n\nFour.\n";
    let parsed = parse(text, &options);
    let typed = |start, text: &str| TextEdit {
        start,
        end: start,
        text: text.into(),
    };
    let explained = explain(&parsed, text, &typed(7, "!"), &options);
    assert!(
        explained.contains("  1: 6..10 \"Two.\", 1 blocks, 0 diagnostics\n"),
        "{explained}"
    );
    assert!(explained.contains("  units 0..=2: done\n"), "{explained}");
    let opened = explain(&parsed, text, &typed(6, "```\n"), &options);
    assert!(opened.contains("  units 0..=2: grow\n"), "{opened}");
    let defined = explain(&parsed, text, &typed(6, "[x]: /y\n\n"), &options);
    assert!(defined.contains("  units 0..=2: whole\n"), "{defined}");
    assert!(defined.contains("  parsed the whole text\n"), "{defined}");
}

#[test]
fn an_edit_changes_a_tree_nothing_else_shares_in_place() {
    let options = ParseOptions::default();
    let text = "One.\n\nTwo.\n\nThree.\n";
    let parsed = parse(text, &options);
    let tree = Rc::as_ptr(&parsed.root);
    let edit = TextEdit {
        start: 9,
        end: 9,
        text: "!".into(),
    };
    let (_, edited, _) = reparsed(parsed, text, &edit, &options);
    assert_eq!(Rc::as_ptr(&edited.root), tree);
    let shared = Rc::clone(&edited.root);
    let (_, copied, _) = reparsed(edited, "One.\n\nTwo!.\n\nThree.\n", &edit, &options);
    assert_ne!(Rc::as_ptr(&copied.root), Rc::as_ptr(&shared));
    assert_eq!(
        shared.children.len(),
        3,
        "a tree another holds stays as it was"
    );
}
