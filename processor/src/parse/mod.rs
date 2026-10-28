//! Stage 1, parsing (spec §11): text in, the document tree (spec §12) and the parse diagnostics out.
//!
//! The fork of markdown-rs parses CommonMark, GFM and CompMD's syntax into mdast, with each component line, inline
//! component, interpolation and wiki link as a raw node. [`Converter`] turns that into the tree: it builds
//! containers, slots and leaves from the component lines, reads params, fallbacks, definitions and signatures, and
//! counts positions in code points.

mod blocks;
mod definitions;
mod frontmatter;
mod inlines;
mod line;
mod params;
mod source;
mod yaml;

use crate::diagnostic::{Code, Diagnostic, Location};
use crate::tree::{Point, Root};
use markdown::mdast;
use serde_json::Value;
use source::{START, Source};

/// What a parse needs besides the text.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParseOptions {
    /// Metadata a host keeps apart from the text, equivalent to frontmatter (spec §2.1). A document with its own
    /// frontmatter uses only that, and W115 reports the metadata as ignored.
    pub metadata: Option<Value>,
}

/// The result of parsing a document.
#[derive(Clone, Debug, PartialEq)]
pub struct Parsed {
    /// The document tree.
    pub root: Root,
    /// The parse diagnostics, in the order they were found.
    pub diagnostics: Vec<Diagnostic>,
}

/// Parses a CompMD document: stage 1 of spec §11.
///
/// Parsing never fails: a mistake is a diagnostic, and the recovery rules of spec §10.3 decide the tree. A document
/// whose frontmatter makes CompMD processing not apply (F001, F033) is parsed as plain CommonMark and GFM.
///
/// ```
/// use compmd::{ParseOptions, parse};
/// use compmd::tree::Node;
///
/// let parsed = parse("::: card {title=\"Setup\"}\nInstall it.\n::: /card\n", &ParseOptions::default());
/// assert!(parsed.diagnostics.is_empty());
/// let Node::ComponentContainer(card) = &parsed.root.children[0] else {
///     panic!("a container");
/// };
/// assert_eq!(card.name, "card");
/// ```
#[must_use]
pub fn parse(text: &str, options: &ParseOptions) -> Parsed {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut diagnostics = Vec::new();
    let mut settings = options.metadata.clone();
    let mut parsed = None;
    let mut compmd = true;
    if let Some(yaml) = frontmatter::find(text) {
        if options.metadata.is_some() {
            diagnostics.push(Diagnostic::new(
                Code::W115,
                Location::new(1, 1),
                "the document has its own frontmatter, so the host's metadata is ignored",
            ));
        }
        parsed = frontmatter::read(yaml, &mut diagnostics);
        compmd = parsed.is_some();
        settings.clone_from(&parsed);
    }
    let syntax = Syntax {
        compmd,
        wiki_links: settings.as_ref().is_some_and(frontmatter::wiki_links),
    };
    let root = Converter {
        source: Source::new(text, START),
        syntax,
        frontmatter: parsed,
        diagnostics: &mut diagnostics,
    }
    .document(true);
    Parsed { root, diagnostics }
}

/// The syntax a document is parsed with.
#[derive(Clone, Copy, Debug)]
struct Syntax {
    /// CompMD's syntax; without it, plain CommonMark and GFM (F001, F033).
    compmd: bool,
    /// Wiki links (spec §2.3), opted in by `md.syntax`.
    wiki_links: bool,
}

impl Syntax {
    /// The fork's mdast for `text`. Without MDX the fork has no syntax errors, so it always gives a root.
    fn mdast(self, text: &str, frontmatter: bool) -> mdast::Root {
        match markdown::to_mdast(text, &self.fork_options(frontmatter)) {
            Ok(mdast::Node::Root(root)) => root,
            _ => mdast::Root {
                children: Vec::new(),
                position: None,
            },
        }
    }

    fn fork_options(self, frontmatter: bool) -> markdown::ParseOptions {
        let constructs = if self.compmd {
            markdown::Constructs {
                compmd_wiki_link: self.wiki_links,
                frontmatter,
                ..markdown::Constructs::compmd()
            }
        } else {
            markdown::Constructs {
                frontmatter,
                frontmatter_toml: false,
                ..markdown::Constructs::gfm()
            }
        };
        markdown::ParseOptions {
            constructs,
            ..markdown::ParseOptions::gfm()
        }
    }
}

/// Turns the fork's mdast for one text into the tree.
struct Converter<'s, 'd> {
    source: Source<'s>,
    syntax: Syntax,
    /// The parsed frontmatter, for the `yaml` node.
    frontmatter: Option<Value>,
    diagnostics: &'d mut Vec<Diagnostic>,
}

impl Converter<'_, '_> {
    /// The tree of the whole text, with frontmatter when it may have some.
    fn document(mut self, frontmatter: bool) -> Root {
        let root = self.syntax.mdast(self.source.text(), frontmatter);
        let position = self.source.position(root.position.as_ref());
        let end = position.map_or(START, |position| position.end);
        Root {
            children: self.blocks(root.children, end, &[]).0,
            position,
        }
    }

    /// A converter for `text`, parsed on its own and placed at `origin`, with the same syntax: for a fallback or a
    /// definition's body.
    fn placed<'t>(&mut self, text: &'t str, origin: Point) -> Converter<'t, '_> {
        Converter {
            source: Source::new(text, origin),
            syntax: self.syntax,
            frontmatter: None,
            diagnostics: self.diagnostics,
        }
    }

    fn diagnose(&mut self, code: Code, at: impl Into<Location>, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::new(code, at.into(), message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::Node;

    #[test]
    fn a_byte_order_mark_is_ignored() {
        let parsed = parse("\u{feff}---\nmd: {}\n---\n", &ParseOptions::default());
        assert!(matches!(parsed.root.children[..], [Node::Yaml(_)]));
    }

    #[test]
    fn host_metadata_opts_in_to_syntax_when_there_is_no_frontmatter() {
        let options = ParseOptions {
            metadata: Some(serde_json::json!({ "md": { "syntax": ["wiki-links"] } })),
        };
        let tree = serde_json::to_value(parse("[[Home]]", &options).root).unwrap();
        assert_eq!(tree["children"][0]["children"][0]["type"], "wikiLink");
    }
}
