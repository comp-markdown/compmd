//! What an expansion may spend (spec §8.5): the nodes it makes, the work it does, iterations and invocations, so one
//! that makes nothing still ends, and examples counting toward their document's limits.

use super::*;
use crate::common::at;
use std::fmt::Write as _;

fn limited(input: &str, options: expand::Options) -> Rendered {
    render(
        input,
        Setup {
            options,
            ..Setup::default()
        },
    )
}

#[test]
fn the_node_limit_stops_templates_and_invocations() {
    let input = "```compmd component=pair\n---\nA\n\nB\n```\n\n::: each {in=([1, 2, 3]) as=i}\n:: pair[P]\n::: /each\n\n```compmd component=wide\nparams:\n  rows: array\n  as: name\nslots:\n  default: {each: rows, as: as}\n---\n{{md.slot}}\n```\n\n::: wide {rows=([1, 2, 3]) as=r}\nRow {{r}}\n::: /wide\n";
    let nodes = |limit| {
        limited(
            input,
            expand::Options {
                limit,
                ..expand::Options::default()
            },
        )
    };
    let rendered = nodes(4);
    assert_eq!(rendered.found, [at("F034", 9, 1)]);
    assert_eq!(text(&rendered.tree), "P|Row {{r}}");
    let unlimited = nodes(1000);
    assert_eq!(unlimited.found, []);
    assert_eq!(text(&unlimited.tree), "AB|AB|AB|Row 1Row 2Row 3");
}

#[test]
fn nested_loops_that_make_nothing_end_at_the_work_limit() {
    // Five `each` over 100 items, the innermost empty: 10^10 iterations, and not one node.
    let items: Vec<String> = (0..100).map(|n| n.to_string()).collect();
    let loops = ["a", "b", "c", "d", "e"]
        .map(|name| format!("::: each {{in=(md.doc.l) as={name}}}\n"))
        .concat();
    let input = format!(
        "---\nl: [{}]\n---\n\n{loops}{}After.\n",
        items.join(", "),
        "::: /each\n".repeat(5)
    );
    // A lower limit than the processor's keeps the test fast; the conformance case crosses the processor's.
    let options = expand::Options {
        work: 10_000,
        ..expand::Options::default()
    };
    let rendered = limited(&input, options);
    assert_eq!(
        rendered.found,
        [at("F034", 9, 1)],
        "at the innermost `each`"
    );
    assert_eq!(text(&rendered.tree), "After.");
}

/// `levels` definitions with no box of their own, each invoking the next twice, so the last is invoked 2^`levels`
/// times, making nothing; then the first invoked, and a paragraph.
fn doubling(levels: usize) -> String {
    let definitions = (0..levels).fold(String::new(), |mut text, n| {
        let next = n + 1;
        let _ = write!(
            text,
            "```compmd component=d{n} {{display=contents}}\n---\n:: d{next}[d]\n\n:: d{next}[d]\n```\n\n"
        );
        text
    });
    format!(
        "{definitions}```compmd component=d{levels} {{display=contents}}\n---\n```\n\n:: d0[Start]\n\nAfter.\n"
    )
}

#[test]
fn a_doubling_chain_of_definitions_ends_at_the_work_limit() {
    let options = expand::Options {
        work: 10_000,
        ..expand::Options::default()
    };
    let rendered = limited(&doubling(30), options.clone());
    let codes: Vec<&str> = rendered
        .found
        .iter()
        .map(|(code, ..)| code.as_str())
        .collect();
    assert_eq!(codes, ["F034"]);
    // The invocation that crossed it, and each that would start after, renders its fallback, then the paragraph.
    let text = text(&rendered.tree);
    let (fallbacks, after) = text.rsplit_once('|').expect("blocks");
    assert!(fallbacks.split('|').all(|block| block == "d"), "{text}");
    assert_eq!(after, "After.");
    // 2^12 - 1 invocations: under the limit, the chain expands to nothing, with nothing to report.
    assert_eq!(limited(&doubling(11), options).found, []);
}

#[test]
fn examples_count_toward_their_document_s_limits() {
    // Each example's `each` is four units of work: the `each`, and its three iterations.
    let example = "````compmd {render=output}\n::: each {in=([1, 2, 3]) as=i}\nRow {{i}}\n::: /each\n````\n\n";
    let input = format!(
        "{}::: each {{in=([1, 2]) as=i}}\nMine {{{{i}}}}\n::: /each\n",
        example.repeat(3)
    );
    let work = |work| {
        limited(
            &input,
            expand::Options {
                work,
                ..expand::Options::default()
            },
        )
    };
    // Twelve units for the examples and three for the document's own: with ten, the third example crosses the
    // limit, and the document's `each` has nothing left.
    assert_eq!(work(10).found, [at("F034", 14, 1), at("F034", 19, 1)]);
    assert_eq!(work(15).found, []);
}
