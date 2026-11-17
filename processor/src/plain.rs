//! A document as plain text and plain HTML (spec §14): what a plain CommonMark and GFM renderer shows of it, for an
//! editor that shows how a document degrades, and its text as a reader reads it, for a search index or a preview.

use crate::parse::{ParseOptions, parse};
use crate::tree::Node;

/// The HTML a plain GFM renderer gives for `text`: the text through markdown-rs with GFM on and CompMD's syntax off.
/// Raw HTML is escaped and unsafe link protocols are dropped, as markdown-rs does by default.
///
/// ```
/// let html = compmd::plain::html(":: chart[Weekly installs] {src=a.csv}\n\n~~old~~\n");
/// assert_eq!(html, "<p>:: chart[Weekly installs] {src=a.csv}</p>\n<p><del>old</del></p>\n");
/// ```
#[must_use]
pub fn html(text: &str) -> String {
    // Markdown has no syntax errors; only MDX, which GFM doesn't turn on, gives one.
    markdown::to_html_with_options(text, &markdown::Options::gfm())
        .unwrap_or_else(|message| message.to_string())
}

/// A document's text as a reader reads it (spec §14), one line per block, so its first line is its first block's
/// text: syntax markers, URLs, raw HTML, the frontmatter, component lines and definitions are left out; link and
/// wiki-link labels, images' alt text, code, math and components' fallback text are kept. Interpolations are left
/// out, since their values are a render's. `options` gives what a parse takes besides the text, such as a host's
/// metadata that turns on wiki links.
///
/// ```
/// use compmd::ParseOptions;
///
/// let text = compmd::plain::text(
///     "# Day 1\n\n:: chart[Pages read] {rows=x}\n\n[Rome](https://example.com) and **more**\n",
///     &ParseOptions::default(),
/// );
/// assert_eq!(text, "Day 1\nPages read\nRome and more");
/// ```
#[must_use]
pub fn text(text: &str, options: &ParseOptions) -> String {
    let mut lines = Vec::new();
    blocks(&parse(text, options).root.children, &mut lines);
    lines.join("\n")
}

/// Adds the lines of `nodes`, blocks, to `lines`.
fn blocks(nodes: &[Node], lines: &mut Vec<String>) {
    for node in nodes {
        match node {
            Node::Paragraph(_) | Node::Heading(_) | Node::ComponentLeaf(_) => {
                push(lines, &inline_text(node.children()));
            }
            Node::Code(code) => code.value.lines().for_each(|line| push(lines, line)),
            Node::Math(math) => math.value.lines().for_each(|line| push(lines, line)),
            Node::Table(table) => {
                for row in &table.children {
                    let cells: Vec<String> = row
                        .children()
                        .iter()
                        .map(|cell| inline_text(cell.children()))
                        .collect();
                    push(lines, &cells.join(" "));
                }
            }
            Node::ComponentContainer(container) => {
                // A container's fallback: its default slot, then each named slot, as Markdown (spec §4.5).
                blocks(&container.children, lines);
                for slot in &container.slots {
                    blocks(&slot.children, lines);
                }
            }
            Node::Blockquote(_)
            | Node::List(_)
            | Node::ListItem(_)
            | Node::FootnoteDefinition(_)
            | Node::ComponentSlot(_) => {
                blocks(node.children(), lines);
            }
            // The frontmatter, definitions and signatures, link definitions, raw HTML and thematic breaks show no text.
            _ => {}
        }
    }
}

/// The text of `nodes`, inline content, on one line.
fn inline_text(nodes: &[Node]) -> String {
    let mut text = String::new();
    inline(nodes, &mut text);
    text
}

/// Adds the text of `nodes`, inline content, to `text`.
fn inline(nodes: &[Node], text: &mut String) {
    for node in nodes {
        match node {
            Node::Text(literal) | Node::InlineCode(literal) | Node::InlineMath(literal) => {
                text.push_str(&literal.value);
            }
            Node::Break(_) => text.push(' '),
            Node::Image(image) => text.push_str(&image.alt),
            Node::ImageReference(image) => text.push_str(&image.alt),
            Node::WikiLink(link) => text.push_str(&link.label),
            Node::Html(_) | Node::Interpolation(_) | Node::FootnoteReference(_) => {}
            // Emphasis, strong, strikethrough, links and inline components: their text, or their fallback.
            other => inline(other.children(), text),
        }
    }
}

/// Adds `line` to `lines`, its white space runs made one space, unless it has no text.
fn push(lines: &mut Vec<String>, line: &str) {
    let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if !line.is_empty() {
        lines.push(line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn plain(document: &str) -> String {
        text(document, &ParseOptions::default())
    }

    #[test]
    fn component_lines_stay_one_line_of_text() {
        let html = html(
            "::: card {title=\"Setup\"}\nInside.\n::: /card\n\n| a |\n|---|\n| 1 |\n\n<script>x</script>\n",
        );
        assert!(
            html.contains("<p>::: card {title=&quot;Setup&quot;}\nInside.\n::: /card</p>"),
            "{html}"
        );
        assert!(html.contains("<table>"), "GFM tables: {html}");
        assert!(
            html.contains("&lt;script&gt;"),
            "raw HTML is escaped: {html}"
        );
    }

    #[test]
    fn syntax_markers_go() {
        let document = "## A *very* **big** ~~day~~\n\n- one\n- [x] two\n  1. nested\n\n> [!NOTE]\n> Quoted\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n---\n";
        assert_eq!(
            plain(document),
            "A very big day\none\ntwo\nnested\nQuoted\na b\n1 2"
        );
    }

    #[test]
    fn labels_stay_and_urls_go() {
        let document = "[Rome](https://example.com \"title\"), ![a map](map.png), [ref][r] and <https://bare.example>\n\n[r]: https://example.org\n";
        assert_eq!(plain(document), "Rome, a map, ref and https://bare.example");
        let wiki = text(
            "See [[Books|my books]] and [[Rome]].\n",
            &ParseOptions {
                metadata: Some(json!({"md": {"syntax": ["wiki-links"]}})),
            },
        );
        assert_eq!(wiki, "See my books and Rome.");
    }

    #[test]
    fn code_and_math_stay_as_text() {
        let document =
            "Use `npm i` and $x^2$.\n\n```mermaid\ngraph LR\n  a --> b\n```\n\n$$\ny = 1\n$$\n";
        assert_eq!(
            plain(document),
            "Use npm i and x^2.\ngraph LR\na --> b\ny = 1"
        );
    }

    #[test]
    fn components_give_their_fallback_and_definitions_nothing() {
        let document = "::: card {title=\"Setup\"}\nInside.\n\n:: #footer\nThe end.\n::: /card\n\n:: chart[Pages read] {rows=x}\n\nA :badge[beta]{tone=info} release.\n\n```compmd component=note\nslots: [default]\n---\nHidden body\n```\n\n```compmd component=stars signature\nparams:\n  value: number\n```\n";
        assert_eq!(
            plain(document),
            "Inside.\nThe end.\nPages read\nA beta release."
        );
    }

    #[test]
    fn frontmatter_html_and_interpolations_show_nothing() {
        let document = "---\ntitle: Notes\n---\n\n<div>\n\nHello {{md.doc.title}}, <b>you</b>.[^1]\n\n</div>\n\n[^1]: A note.\n";
        assert_eq!(plain(document), "Hello , you.\nA note.");
    }

    #[test]
    fn one_line_per_block() {
        assert_eq!(
            plain("First line\nsame paragraph  \nhard break\n\nSecond\n"),
            "First line same paragraph hard break\nSecond"
        );
        assert_eq!(plain(""), "");
    }
}
