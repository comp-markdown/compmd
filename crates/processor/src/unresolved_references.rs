//! The references no definition resolves, for an editor (the language server, Q10): `[text][label]`, `[label][]` and
//! `[label]` whose label no link reference definition of the document defines. CommonMark makes such a reference text,
//! so the tree has no node for it; the fork of markdown-rs says where each one is as it parses.

use crate::tree::Position;
use serde::Serialize;

/// A reference whose label no definition of the document defines: it is text, not a link.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UnresolvedReference {
    /// Its label, as written.
    pub label: String,
    /// Where the label's text is, between its brackets.
    pub position: Position,
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::context::RenderContext;
    use crate::processor::Expanded;
    use crate::{Ambient, ParseOptions, Processor, TextEdit};

    /// `text`, the file `name`, processed: a `.md` file as plain Markdown.
    fn processed(name: &str, text: &str) -> Processor<Expanded> {
        let context = RenderContext {
            now: 0,
            tz: "UTC".into(),
            time: Rc::new(()),
            inputs: serde_json::Map::new(),
        };
        let options = ParseOptions {
            plain_markdown: name == "notes.md",
            ..ParseOptions::default()
        };
        Processor::new(name, text)
            .parse(&options)
            .resolve((), (), &Ambient::default())
            .validate(&[])
            .load(context, (), ())
            .expand(&crate::expand::Options::default())
    }

    /// Each unresolved reference's label, line and column.
    fn unresolved(processor: &Processor<Expanded>) -> Vec<(String, usize, usize)> {
        processor
            .unresolved_references()
            .into_iter()
            .map(|reference| {
                let start = reference.position.start;
                (reference.label, start.line, start.column)
            })
            .collect()
    }

    fn at(label: &str, line: usize, column: usize) -> (String, usize, usize) {
        (label.to_owned(), line, column)
    }

    #[test]
    fn references_in_containers_and_plain_markdown_files_by_code_points() {
        let text = "::: box\nThe [tide tables][tables].\n::: /box\n\n- [knots]\n";
        assert_eq!(
            unresolved(&processed("notes.comp.md", text)),
            [at("tables", 2, 19), at("knots", 5, 4)]
        );
        let plain = "\u{feff}Café [crème][menu].\n";
        assert_eq!(
            unresolved(&processed("notes.md", plain)),
            [at("menu", 1, 14)],
            "a byte order mark and an é are no columns of their own"
        );
    }

    #[test]
    fn a_definition_anywhere_resolves_it_and_an_edit_is_followed() {
        let text = "See [the tides][tides].\n";
        let processor = processed("notes.comp.md", text);
        assert_eq!(unresolved(&processor), [at("tides", 1, 17)]);
        let defined = processor.edit(&TextEdit {
            start: text.chars().count(),
            end: text.chars().count(),
            text: "\n::: box\n[tides]: ./tides.md\n::: /box\n".into(),
        });
        assert_eq!(unresolved(&defined), [], "defined in a container, after it");
    }
}
