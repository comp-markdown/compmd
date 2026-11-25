use compmd::frontmatter::Syntax;
use lsp_types::{Position, Range};

use super::Frontmatter;
use crate::features::testing::{asking, processed};

/// The frontmatter of `text`, given to `check`.
fn with_frontmatter(text: &str, check: impl FnOnce(Option<&Frontmatter>)) {
    let processor = processed(text);
    asking(text, &processor, &[], |asked| {
        check(Frontmatter::of(asked).as_ref());
    });
}

#[test]
fn comments_on_lines_of_their_own_stay_with_the_key_after_them() {
    let text = "---\n# The post's\ntitle: Notes # not kept\nbody: |\n  # a heading, not a comment\n\n# CompMD's\nmd:\n  # its language\n  lang: de\n# the end\n---\n\n# Notes\n";
    with_frontmatter(text, |frontmatter| {
        let frontmatter = frontmatter.expect("a frontmatter");
        let comments = &frontmatter.comments;
        assert_eq!(comments.above["/title"], ["The post's"]);
        assert_eq!(comments.above["/md"], ["CompMD's"]);
        assert_eq!(comments.above["/md/lang"], ["its language"]);
        assert_eq!(comments.above.len(), 3);
        assert_eq!(comments.end, ["the end"]);
    });
}

#[test]
fn the_rewrite_replaces_the_whole_frontmatter_in_either_syntax() {
    let text = "---\n# The post's\ntitle: Notes\n---\n\n# Notes\n";
    with_frontmatter(text, |frontmatter| {
        let frontmatter = frontmatter.expect("a frontmatter");
        assert_eq!(frontmatter.syntax, Syntax::Yaml);
        assert_eq!(frontmatter.key_lines["/title"], 2);
        assert!(frontmatter.holds(Position {
            line: 3,
            character: 3
        }));
        assert!(!frontmatter.holds(Position {
            line: 4,
            character: 0
        }));
        assert_eq!(
            (frontmatter.first_line(), frontmatter.closing_line()),
            (1, 3)
        );
        let edit = frontmatter
            .rewritten(Syntax::Toml, &frontmatter.value)
            .expect("TOML holds it");
        assert_eq!(
            edit.range,
            Range {
                start: Position {
                    line: 0,
                    character: 0
                },
                end: Position {
                    line: 3,
                    character: 3
                },
            }
        );
        assert_eq!(edit.new_text, "+++\n# The post's\ntitle = \"Notes\"\n+++");
    });
}

#[test]
fn none_without_a_frontmatter_that_is_a_mapping() {
    with_frontmatter("# Notes\n", |frontmatter| assert!(frontmatter.is_none()));
    with_frontmatter("---\njust text\n---\n", |frontmatter| {
        assert!(frontmatter.is_none());
    });
}
