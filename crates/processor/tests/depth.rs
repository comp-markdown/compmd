//! How deep a document nests (spec §8.5): blocks nested deeper than the processor allows are F038, shown as their
//! text, and so are invocations, which render their fallbacks; a document nested far deeper than that is refused, not
//! a stack overflow.

use compmd::context::RenderContext;
use compmd::expand::Options;
use compmd::{Ambient, ParseOptions, Processor};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::rc::Rc;

/// The depth every processor renders.
const NESTING: usize = 100;

fn context() -> RenderContext {
    RenderContext {
        now: 0,
        tz: "UTC".into(),
        time: Rc::new(()),
        inputs: serde_json::Map::new(),
    }
}

/// What processing `text` reports, as `(code, line)`, and its expanded tree's text.
fn processed(text: &str) -> (Vec<(String, usize)>, String) {
    let files: HashMap<String, String> = HashMap::new();
    let processor = Processor::new("deep.comp.md", text)
        .parse(&ParseOptions::default())
        .resolve(files, (), &Ambient::default())
        .validate(&[])
        .load(context(), (), ())
        .expand(&Options::default());
    let found = processor
        .diagnostics()
        .iter()
        .map(|diagnostic| (diagnostic.code.to_string(), diagnostic.range.start.line))
        .collect();
    let tree = serde_json::to_string(processor.expanded()).expect("the tree serializes");
    (found, tree)
}

fn f038(line: usize) -> (String, usize) {
    ("F038".to_string(), line)
}

#[test]
fn ten_thousand_quote_markers_on_one_line_are_refused() {
    let (found, tree) = processed(&format!("{}Deep.\n", "> ".repeat(10_000)));
    assert_eq!(found, [f038(1)]);
    assert!(tree.contains("Deep."));
}

#[test]
fn blocks_nest_up_to_the_limit() {
    let quotes = format!("{}At the limit.\n", "> ".repeat(NESTING));
    assert_eq!(processed(&quotes).0, []);
    let lists = format!("{}At the limit.\n", "- ".repeat(NESTING));
    assert_eq!(processed(&lists).0, []);
}

#[test]
fn a_list_past_the_limit_is_shown_as_its_text() {
    let (found, tree) = processed(&format!("{}Deep.\n", "- ".repeat(NESTING + 1)));
    assert_eq!(found, [f038(1)]);
    assert!(tree.contains("Deep."));
}

#[test]
fn containers_count_toward_the_depth_of_what_they_hold() {
    let openers = "::: box\n".repeat(NESTING);
    let closers = "::: /box\n".repeat(NESTING);
    // A hundred containers: the quote in them is one too deep.
    let quoted = format!("{openers}> A quote.\n{closers}");
    let (found, tree) = processed(&quoted);
    assert_eq!(found, [f038(NESTING + 1)]);
    assert!(tree.contains("A quote."));
    // The hundred and first container is shown as its lines, its content in the hundredth, and what is inside it
    // isn't reported again.
    let contained = format!("{openers}::: box\n> Inside.\n::: /box\n{closers}");
    let (found, tree) = processed(&contained);
    assert_eq!(found, [f038(NESTING + 1)]);
    assert!(tree.contains("::: box") && tree.contains("Inside.") && tree.contains("::: /box"));
}

#[test]
fn invocations_nest_up_to_the_limit() {
    // `d0` invokes `d1`, …, each the next, `levels` deep, the last saying "Leaf.".
    let chain = |levels: usize| {
        let mut text = (0..levels).fold(String::new(), |mut text, n| {
            let _ = write!(
                text,
                "```compmd component=d{n}\n---\n:: d{}[Fallback {n}]\n```\n\n",
                n + 1
            );
            text
        });
        let _ = write!(
            text,
            "```compmd component=d{levels}\n---\nLeaf.\n```\n\n:: d0[Start]\n"
        );
        text
    };
    let (found, tree) = processed(&chain(NESTING - 1));
    assert_eq!(found, [], "a hundred invocations under way");
    assert!(tree.contains("Leaf."));
    let (found, tree) = processed(&chain(NESTING));
    let codes: Vec<&str> = found.iter().map(|(code, _)| code.as_str()).collect();
    assert_eq!(
        codes,
        ["F038"],
        "the hundred and first renders its fallback"
    );
    assert!(tree.contains(&format!("Fallback {}", NESTING - 1)) && !tree.contains("Leaf."));
}
