//! The glyphs a mark's text comes from ([a document drawn in the editor], decision 6): a set of them (`unicode`,
//! `nerd` or `ascii`), each by its name, and the writer's own text for any of them (`render.glyph`).
//!
//! [a document drawn in the editor]: https://github.com/comp-markdown/compmd/blob/main/docs/editor-rendering.md

use std::collections::BTreeMap;

use crate::settings::GlyphSet;

/// The glyphs of a set, with the writer's own.
pub(crate) struct Glyphs<'s> {
    set: GlyphSet,
    own: &'s BTreeMap<String, String>,
}

impl<'s> Glyphs<'s> {
    /// The glyphs of `set`, those `own` names as the writer writes them.
    pub(crate) const fn new(set: GlyphSet, own: &'s BTreeMap<String, String>) -> Self {
        Self { set, own }
    }

    /// The glyph `name` names; empty for a name the set has no glyph for.
    pub(crate) fn get(&self, name: &str) -> String {
        self.own
            .get(name)
            .cloned()
            .unwrap_or_else(|| of_set(self.set, name).to_owned())
    }
}

/// The glyph `name` names in `set`.
fn of_set(set: GlyphSet, name: &str) -> &'static str {
    if let Some(language) = name.strip_prefix("language.") {
        return match set {
            GlyphSet::Nerd => language_icon(language),
            GlyphSet::Unicode | GlyphSet::Ascii => "",
        };
    }
    let (unicode, nerd, ascii) = match name {
        "heading.1" => ("H1", "\u{F0CA1}", "#"),
        "heading.2" => ("H2", "\u{F0CA3}", "##"),
        "heading.3" => ("H3", "\u{F0CA5}", "###"),
        "heading.4" => ("H4", "\u{F0CA7}", "####"),
        "heading.5" => ("H5", "\u{F0CA9}", "#####"),
        "heading.6" => ("H6", "\u{F0CAB}", "######"),
        "bullet.1" => ("●", "●", "*"),
        "bullet.2" => ("○", "○", "-"),
        "bullet.3" => ("◆", "◆", "+"),
        "bullet.4" => ("◇", "◇", "-"),
        "checkbox.unchecked" => ("☐", "\u{F0131}", "[ ]"),
        "checkbox.checked" => ("☑", "\u{F0C52}", "[x]"),
        "quote" => ("▋", "▋", "|"),
        "callout.note" => ("ℹ", "\u{F02FD}", "i"),
        "callout.tip" => ("✦", "\u{F0336}", "*"),
        "callout.important" => ("‼", "\u{F017E}", "!"),
        "callout.warning" => ("⚠", "\u{F002A}", "!"),
        "callout.caution" => ("✖", "\u{F0CE6}", "x"),
        "rule" | "box.horizontal" => ("─", "─", "-"),
        "box.vertical" => ("│", "│", "|"),
        "box.top-left" => ("┌", "┌", "+"),
        "box.top" => ("┬", "┬", "+"),
        "box.top-right" => ("┐", "┐", "+"),
        "box.left" => ("├", "├", "+"),
        "box.cross" => ("┼", "┼", "+"),
        "box.right" => ("┤", "┤", "+"),
        "box.bottom-left" => ("└", "└", "+"),
        "box.bottom" => ("┴", "┴", "+"),
        "box.bottom-right" => ("┘", "┘", "+"),
        _ => ("", "", ""),
    };
    match set {
        GlyphSet::Unicode => unicode,
        GlyphSet::Nerd => nerd,
        GlyphSet::Ascii => ascii,
    }
}

/// A language's Nerd Font icon, by its name in a code block's info string.
fn language_icon(language: &str) -> &'static str {
    match language {
        "rust" => "\u{E7A8}",
        "python" => "\u{E73C}",
        "javascript" | "js" => "\u{E74E}",
        "typescript" | "ts" => "\u{E628}",
        "lua" => "\u{E620}",
        "json" => "\u{E60B}",
        "sh" | "bash" | "shell" => "\u{E795}",
        "html" => "\u{E736}",
        "css" => "\u{E749}",
        "markdown" | "md" | "compmd" => "\u{E73E}",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_set_has_its_glyphs() {
        let none = BTreeMap::new();
        let unicode = Glyphs::new(GlyphSet::Unicode, &none);
        let nerd = Glyphs::new(GlyphSet::Nerd, &none);
        let ascii = Glyphs::new(GlyphSet::Ascii, &none);
        assert_eq!(unicode.get("heading.2"), "H2");
        assert_eq!(nerd.get("heading.2"), "\u{F0CA3}");
        assert_eq!(ascii.get("heading.2"), "##");
        assert_eq!(
            (unicode.get("box.cross"), ascii.get("box.cross")),
            ("┼".into(), "+".into())
        );
        assert_eq!(unicode.get("nothing"), "");
    }

    #[test]
    fn languages_have_icons_in_nerd_only() {
        let none = BTreeMap::new();
        let nerd = Glyphs::new(GlyphSet::Nerd, &none);
        let icons: Vec<String> = [
            "rust", "python", "js", "ts", "lua", "json", "bash", "html", "css", "md",
        ]
        .iter()
        .map(|language| nerd.get(&format!("language.{language}")))
        .collect();
        assert!(icons.iter().all(|icon| !icon.is_empty()), "{icons:?}");
        assert_eq!(nerd.get("language.cobol"), "");
        assert_eq!(
            Glyphs::new(GlyphSet::Unicode, &none).get("language.rust"),
            ""
        );
    }

    #[test]
    fn the_writers_own_glyph_comes_first() {
        let own = BTreeMap::from([
            ("heading.2".to_owned(), "§".to_owned()),
            ("language.rust".to_owned(), "rs".to_owned()),
        ]);
        let glyphs = Glyphs::new(GlyphSet::Ascii, &own);
        assert_eq!(glyphs.get("heading.2"), "§");
        assert_eq!(glyphs.get("language.rust"), "rs");
        assert_eq!(glyphs.get("heading.1"), "#");
    }
}
