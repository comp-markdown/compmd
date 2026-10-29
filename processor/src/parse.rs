//! Stage 1, parsing (spec §11): text in, the document tree (spec §12) and the parse diagnostics out.
//!
//! The fork of markdown-rs parses CommonMark, GFM and CompMD's syntax into mdast, with each component line, inline
//! component, interpolation and wiki link as a raw node. [`Converter`] turns that into the tree: it builds
//! containers, slots and leaves from the component lines, reads params, fallbacks, definitions and signatures, and
//! counts positions in code points.

mod blocks;
pub(crate) mod definitions;
mod frontmatter;
mod inlines;
mod line;
pub(crate) mod params;
mod source;
pub(crate) mod yaml;

use crate::diagnostic::{Code, Diagnostic, Location};
use crate::settings::Settings;
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
#[derive(Clone, Debug)]
pub(crate) struct Parsed {
    /// The document tree.
    pub(crate) root: Root,
    /// The settings its frontmatter gives: none without one, or when CompMD processing doesn't apply to it.
    pub(crate) settings: Settings,
    /// The parse diagnostics, in the order they were found.
    pub(crate) diagnostics: Vec<Diagnostic>,
}

/// Parses a CompMD document: stage 1 of spec §11.
///
/// Parsing never fails: a mistake is a diagnostic, and the recovery rules of spec §10.3 decide the tree. A document
/// whose frontmatter makes CompMD processing not apply (F001, F033) is parsed as plain CommonMark and GFM.
pub(crate) fn parse(text: &str, options: &ParseOptions) -> Parsed {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut diagnostics = Vec::new();
    let mut metadata = options.metadata.clone();
    let mut parsed = None;
    let mut settings = Settings::default();
    let mut compmd = true;
    if let Some(yaml) = frontmatter::find(text) {
        if options.metadata.is_some() {
            diagnostics.push(Diagnostic::new(
                Code::W115,
                Location::new(1, 1),
                "the document has its own frontmatter, so the host's metadata is ignored",
            ));
        }
        let document = frontmatter::read(yaml, &mut diagnostics);
        compmd = document.is_some();
        parsed = document.as_ref().map(frontmatter::tree_value);
        metadata.clone_from(&parsed);
        if let Some(document) = document {
            settings = Settings::read(document, yaml);
        }
    }
    let syntax = Syntax {
        compmd,
        wiki_links: metadata.as_ref().is_some_and(frontmatter::wiki_links),
    };
    let root = Converter {
        source: Source::new(text, START),
        syntax,
        labels: &Labels::NONE,
        frontmatter: parsed,
        diagnostics: &mut diagnostics,
    }
    .document(true);
    Parsed {
        root,
        settings,
        diagnostics,
    }
}

/// The syntax a document is parsed with.
#[derive(Clone, Copy, Debug)]
struct Syntax {
    /// CompMD's syntax; without it, plain CommonMark and GFM (F001, F033).
    compmd: bool,
    /// Wiki links (spec §2.3), opted in by `md.syntax`.
    wiki_links: bool,
}

/// The labels a text defines or sees: those of its link reference definitions and footnote definitions, as the
/// fork's mdast `identifier`s.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Labels {
    pub(crate) links: Vec<String>,
    pub(crate) footnotes: Vec<String>,
}

impl Labels {
    const NONE: Self = Self {
        links: Vec::new(),
        footnotes: Vec::new(),
    };

    /// These labels and those `root` defines, at any depth.
    fn with(&self, root: &mdast::Root) -> Self {
        let mut labels = self.clone();
        labels.collect(&root.children);
        labels
    }

    fn collect(&mut self, nodes: &[mdast::Node]) {
        for node in nodes {
            match node {
                mdast::Node::Definition(definition) => {
                    self.links.push(definition.identifier.clone());
                }
                mdast::Node::FootnoteDefinition(definition) => {
                    self.footnotes.push(definition.identifier.clone());
                }
                _ => {}
            }
            if let Some(children) = node.children() {
                self.collect(children);
            }
        }
    }
}

impl Syntax {
    /// The fork's mdast for `text`, with `labels` defined outside it. Without MDX the fork has no syntax errors, so it
    /// always gives a root.
    fn mdast(self, text: &str, frontmatter: bool, labels: &Labels) -> mdast::Root {
        let options = markdown::ParseOptions {
            definitions: labels.links.clone(),
            gfm_footnote_definitions: labels.footnotes.clone(),
            ..self.fork_options(frontmatter)
        };
        match markdown::to_mdast(text, &options) {
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
    /// The labels the text sees: its document's, for a fallback, or those of its own lines.
    labels: &'d Labels,
    /// The parsed frontmatter, for the `yaml` node.
    frontmatter: Option<Value>,
    diagnostics: &'d mut Vec<Diagnostic>,
}

impl Converter<'_, '_> {
    /// The tree of the whole text, with frontmatter when it may have some.
    fn document(self, frontmatter: bool) -> Root {
        let root = self
            .syntax
            .mdast(self.source.text(), frontmatter, self.labels);
        let labels = self.labels.with(&root);
        let mut converter = Converter {
            labels: &labels,
            ..self
        };
        let position = converter.source.position(root.position.as_ref());
        let end = position.map_or(START, |position| position.end);
        Root {
            children: converter.blocks(root.children, end, &[]).0,
            position,
        }
    }

    /// A converter for `text`, parsed on its own and placed at `origin`, with the same syntax and the labels it sees:
    /// for a fallback, those of its document; for a definition's body, none, as it resolves its own (spec §6.3).
    fn placed<'t, 'c>(
        &'c mut self,
        text: &'t str,
        origin: Point,
        labels: &'c Labels,
    ) -> Converter<'t, 'c> {
        Converter {
            source: Source::new(text, origin),
            syntax: self.syntax,
            labels,
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
