//! What a plain CommonMark and GFM renderer shows of a document (spec §14), for an editor that shows how a document
//! degrades: the text through markdown-rs with GFM on and CompMD's syntax off. Raw HTML is escaped and unsafe link
//! protocols are dropped, as markdown-rs does by default.

/// The HTML a plain GFM renderer gives for `text`.
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
