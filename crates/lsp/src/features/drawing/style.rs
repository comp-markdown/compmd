//! The styles a mark names ([a document drawn in the editor], Terms): what a mark is, never a colour. Each editor maps
//! them to its own highlight group, face or theme colour.
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use compmd::tree::Alert;

/// A code block's language.
pub(crate) const CODE_LANGUAGE: &str = "compmd.code.language";
/// Inline code.
pub(crate) const CODE_INLINE: &str = "compmd.code.inline";
/// A thematic break.
pub(crate) const RULE: &str = "compmd.rule";
/// A table's pipes, delimiter row and borders.
pub(crate) const TABLE_BORDER: &str = "compmd.table.border";
/// A checked task's checkbox.
pub(crate) const CHECKBOX_CHECKED: &str = "compmd.checkbox.checked";
/// An unchecked task's checkbox.
pub(crate) const CHECKBOX_UNCHECKED: &str = "compmd.checkbox.unchecked";
/// A container's gutter, the `│` before its lines, and its markers (`:::`, `:: #`, its closer).
pub(crate) const CONTAINER_BORDER: &str = "compmd.container.border";
/// A component's name: a container's, a leaf's.
pub(crate) const COMPONENT: &str = "compmd.component";
/// A slot's name.
pub(crate) const SLOT: &str = "compmd.slot";
/// A component's params, set back.
pub(crate) const PARAM: &str = "compmd.param";
/// The frontmatter's `md` key, set back.
pub(crate) const FRONTMATTER_MD: &str = "compmd.frontmatter.md";
/// A code block's fence marks, set back.
pub(crate) const CODE_FENCE: &str = "compmd.code.fence";
/// A heading's level marker, `H1` to `H6`, set back.
pub(crate) const HEADING_MARKER: &str = "compmd.heading.marker";
/// The frontmatter, set back while the cursor is outside it.
pub(crate) const FRONTMATTER: &str = "compmd.frontmatter";

/// A heading of `depth`.
pub(crate) fn heading(depth: u8) -> String {
    format!("compmd.heading.{depth}")
}

/// A list bullet at `depth`, from 1.
pub(crate) fn bullet(depth: usize) -> String {
    format!("compmd.bullet.{depth}")
}

/// A block quote's bar at `depth`, from 1.
pub(crate) fn quote(depth: usize) -> String {
    format!("compmd.quote.{depth}")
}

/// A GitHub alert of `alert`'s type: its title and its bars.
pub(crate) fn callout(alert: Alert) -> String {
    format!("compmd.callout.{}", alert_name(alert))
}

/// The name of `alert`'s type, as glyphs and styles name it.
pub(crate) const fn alert_name(alert: Alert) -> &'static str {
    match alert {
        Alert::Note => "note",
        Alert::Tip => "tip",
        Alert::Important => "important",
        Alert::Warning => "warning",
        Alert::Caution => "caution",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leveled_styles_name_their_level() {
        assert_eq!(heading(2), "compmd.heading.2");
        assert_eq!(bullet(1), "compmd.bullet.1");
        assert_eq!(quote(3), "compmd.quote.3");
        let callouts: Vec<String> = [
            Alert::Note,
            Alert::Tip,
            Alert::Important,
            Alert::Warning,
            Alert::Caution,
        ]
        .into_iter()
        .map(callout)
        .collect();
        assert_eq!(
            callouts,
            [
                "compmd.callout.note",
                "compmd.callout.tip",
                "compmd.callout.important",
                "compmd.callout.warning",
                "compmd.callout.caution"
            ]
        );
    }
}
