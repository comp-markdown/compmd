use super::*;
use crate::features::testing::processed;

/// The folding ranges of `text`, as their first and last lines, from 0.
fn folded(text: &str) -> Vec<(u32, u32)> {
    folding(&processed(text).tree().children)
        .iter()
        .map(|range| (range.start_line, range.end_line))
        .collect()
}

#[test]
fn the_frontmatter_containers_fences_and_code_blocks_fold() {
    let text = "---\ntitle: A\n---\n\n::: box\n::: box\nIn.\n::: /box\n\n:: #footer\n```sh\nls\n```\n::: /box\n\n```compmd component=note\n---\nA note.\n```\n\n```sh\none line\n```\n";
    assert_eq!(
        folded(text),
        [(0, 2), (4, 13), (5, 7), (10, 12), (15, 18), (20, 22)]
    );
    assert_eq!(folded("+++\ntitle = \"A\"\n+++\n"), [(0, 2)]);
    assert_eq!(
        folded("# A\n\n::: box\n"),
        [],
        "a container recovery closed, on one line"
    );
}
