//! Stage 1, parsing (spec §11): text in, the document tree (spec §12) and the parse diagnostics out.
//!
//! The fork of markdown-rs parses CommonMark, GFM and CompMD's syntax into mdast, with each component line, inline
//! component, interpolation and wiki link as a raw node. [`Converter`] turns that into the tree: it builds
//! containers, slots and leaves from the component lines, reads params, fallbacks, definitions and signatures, and
//! counts positions in code points.

mod blocks;
pub(crate) mod definitions;
mod edit;
pub(crate) mod frontmatter;
mod inlines;
pub(crate) mod line;
mod nesting;
pub(crate) mod params;
mod source;
mod toml;
pub(crate) mod yaml;

pub(crate) use definitions::CODE;
pub use edit::TextEdit;
pub(crate) use edit::{Change, Spliced, reparse};

use crate::diagnostic::{Code, Diagnostic, Location};
use crate::nesting::NESTING;
use crate::settings::Settings;
use crate::tree::{Point, Root};
use crate::unresolved_references::UnresolvedReference;
use blocks::Marks;
use edit::Unit;
use markdown::mdast;
use serde_json::Value;
use source::{START, Source};
use std::mem;
use std::rc::Rc;

/// What a parse needs besides the text.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParseOptions {
    /// Metadata a host keeps apart from the text, equivalent to frontmatter (spec §2.1). A document with its own
    /// frontmatter uses only that, and W115 reports the metadata as ignored.
    pub metadata: Option<Value>,
    /// Whether the file is opened as plain Markdown, a `.md` file, rather than as CompMD (a `.comp.md` file, or the
    /// media type): it is then CompMD only when its frontmatter, or the host's metadata, has an `md` key, and
    /// otherwise CommonMark and GFM, which CompMD processing never applies to (spec §2).
    pub plain_markdown: bool,
}

/// The result of parsing a document.
#[derive(Clone, Debug)]
pub(crate) struct Parsed {
    /// The document tree, which resolving shares rather than copies.
    pub(crate) root: Rc<Root>,
    /// The settings its frontmatter gives: none without one, or when CompMD processing doesn't apply to it.
    pub(crate) settings: Settings,
    /// The parse diagnostics, in the order they were found.
    pub(crate) diagnostics: Vec<Diagnostic>,
    /// The units of the tree's blocks, which an edit parses again on their own.
    units: Vec<Unit>,
    syntax: Syntax,
}

impl Parsed {
    /// Whether the document is plain CommonMark and GFM, which CompMD processing doesn't apply to: a file opened as
    /// plain Markdown without an `md` key, F001, F033.
    pub(crate) fn plain_markdown(&self) -> bool {
        self.syntax.plain_markdown
    }

    /// The references in `text`, the parsed text, whose label no definition of it defines, as the fork sees them
    /// parsing it whole with the document's syntax. One in an inline component's fallback, which is parsed on its own,
    /// isn't among them.
    pub(crate) fn unresolved_references(&self, text: &str) -> Vec<UnresolvedReference> {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let options = self.syntax.fork_options(true);
        let Ok((_, undefined)) = markdown::to_mdast_with_undefined_references(text, &options)
        else {
            return Vec::new();
        };
        let source = Source::new(text, START);
        undefined
            .into_iter()
            .filter_map(|reference| {
                Some(UnresolvedReference {
                    position: source.position(Some(&reference.position))?,
                    label: reference.label,
                })
            })
            .collect()
    }

    /// The tree, shared, and the settings, for resolving: the caller keeps the diagnostics and the units.
    pub(crate) fn for_resolving(&self) -> Self {
        Self {
            root: Rc::clone(&self.root),
            settings: self.settings.clone(),
            diagnostics: Vec::new(),
            units: Vec::new(),
            syntax: self.syntax,
        }
    }
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
    let mut plain_markdown = false;
    if let Some(found) = frontmatter::find(text) {
        if options.metadata.is_some() {
            diagnostics.push(Diagnostic::new(
                Code::W115,
                Location::new(1, 1),
                "the document has its own frontmatter, so the host's metadata is ignored",
            ));
        }
        let document = frontmatter::read(found, &mut diagnostics);
        plain_markdown = document.is_none();
        parsed = document.as_ref().map(frontmatter::tree_value);
        metadata.clone_from(&parsed);
        if let Some(document) = document {
            settings = Settings::read(document, found.text);
        }
    } else if let Some(given) = &options.metadata {
        // A document without frontmatter takes the host's metadata as its frontmatter (spec §2.1).
        let document = frontmatter::metadata(given, &mut diagnostics);
        plain_markdown = document.is_none();
        if let Some(document) = document {
            settings = Settings::read(document, "");
        }
    }
    if options.plain_markdown
        && metadata
            .as_ref()
            .and_then(|value| value.get("md"))
            .is_none()
    {
        // Opened as plain Markdown, without an `md` key: nothing CompMD reads or reports applies to it.
        plain_markdown = true;
        settings = Settings::default();
        diagnostics.clear();
    }
    let syntax = Syntax {
        plain_markdown,
        wiki_links: metadata.as_ref().is_some_and(frontmatter::wiki_links),
    };
    let (root, units) = Converter {
        source: Source::new(text, START),
        syntax,
        labels: &Labels::NONE,
        frontmatter: parsed,
        diagnostics: &mut diagnostics,
        depth: 0,
        cut: Vec::new(),
    }
    .units(true);
    Parsed {
        root: Rc::new(root),
        settings,
        diagnostics,
        units,
        syntax,
    }
}

/// The syntax a document is parsed with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Syntax {
    /// Plain CommonMark and GFM, without CompMD's syntax: a file opened as plain Markdown without an `md` key, F001,
    /// F033.
    plain_markdown: bool,
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
        let constructs = if self.plain_markdown {
            markdown::Constructs {
                frontmatter,
                ..markdown::Constructs::gfm()
            }
        } else {
            markdown::Constructs {
                compmd_wiki_link: self.wiki_links,
                frontmatter,
                ..markdown::Constructs::compmd()
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
    /// The quotes, list items and footnote definitions around the blocks being converted.
    depth: usize,
    /// Where each block cut from the fork's mdast as too deep starts, in bytes, until its F038 is reported (spec
    /// §8.5); a block the converter cuts takes those inside it, so one cut is reported once.
    cut: Vec<usize>,
}

impl Converter<'_, '_> {
    /// The tree of the whole text, with frontmatter when it may have some.
    fn document(self, frontmatter: bool) -> Root {
        self.convert(frontmatter, None).0
    }

    /// [`Self::document`], and the units of its blocks.
    fn units(self, frontmatter: bool) -> (Root, Vec<Unit>) {
        let mut marks = Marks::default();
        let (root, diagnostics) = self.convert(frontmatter, Some(&mut marks));
        let units = Unit::from_marks(&marks, &root.children, diagnostics);
        (root, units)
    }

    /// The tree, and how many diagnostics there are after it.
    fn convert(self, frontmatter: bool, marks: Option<&mut Marks>) -> (Root, usize) {
        let mut root = self
            .syntax
            .mdast(self.source.text(), frontmatter, self.labels);
        let cut = nesting::cut_deep(&mut root.children);
        let labels = self.labels.with(&root);
        let mut converter = Converter {
            labels: &labels,
            cut,
            ..self
        };
        let position = converter.source.position(root.position.as_ref());
        let end = position.map_or(START, |position| position.end);
        let children = converter.blocks(root.children, end, &[], marks).0;
        for offset in mem::take(&mut converter.cut) {
            let at = converter.source.point(offset);
            converter.too_deep(at);
        }
        (Root { children, position }, converter.diagnostics.len())
    }

    /// F038 at `at`, a block nested deeper than the processor allows, shown as its text (spec §8.5, §10.3).
    fn too_deep(&mut self, at: Point) {
        self.diagnose(
            Code::F038,
            at,
            format!("blocks nest deeper than {NESTING} here; this one is shown as its text"),
        );
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
            depth: 0,
            cut: Vec::new(),
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
            ..ParseOptions::default()
        };
        let tree = serde_json::to_value(&*parse("[[Home]]", &options).root).unwrap();
        assert_eq!(tree["children"][0]["children"][0]["type"], "wikiLink");
    }

    #[test]
    fn host_metadata_is_the_frontmatter_of_a_document_without_one_and_is_reported_at_line_1() {
        let options = ParseOptions {
            metadata: Some(serde_json::json!({
                "title": "Notes",
                "md": { "import": [{ "from": "@acme/kit@1", "names": "*" }], "imports": [] }
            })),
            ..ParseOptions::default()
        };
        let parsed = parse("Text.\n", &options);
        assert_eq!(parsed.settings.doc["title"], "Notes");
        assert_eq!(parsed.settings.imports[0].from.value, "@acme/kit@1");
        assert_eq!(parsed.settings.imports[0].from.at, Location::new(1, 1));
        let [unknown] = &parsed.diagnostics[..] else {
            panic!("one diagnostic");
        };
        assert_eq!(
            (unknown.code, unknown.range.start),
            (Code::W111, Location::new(1, 1))
        );
    }

    #[test]
    fn host_metadata_of_another_major_version_makes_the_document_plain() {
        let options = ParseOptions {
            metadata: Some(serde_json::json!({ "md": { "version": 2 } })),
            ..ParseOptions::default()
        };
        let parsed = parse("::: box\nText.\n::: /box\n", &options);
        assert_eq!(parsed.diagnostics[0].code, Code::F001);
        assert!(!matches!(
            parsed.root.children[0],
            Node::ComponentContainer(_)
        ));
    }

    #[test]
    fn a_markdown_file_is_compmd_only_with_an_md_key() {
        let markdown = ParseOptions {
            plain_markdown: true,
            ..ParseOptions::default()
        };
        let box_ = "::: box\nText {{md.doc.title}}.\n::: /box\n";
        let container = |parsed: &Parsed| {
            parsed
                .root
                .children
                .iter()
                .any(|node| matches!(node, Node::ComponentContainer(_)))
        };
        let plain = parse(box_, &markdown);
        assert!(
            plain.plain_markdown() && !container(&plain),
            "no frontmatter"
        );
        let authors = parse(&format!("---\ntitle: A\nmd2: x\n---\n{box_}"), &markdown);
        assert!(
            authors.plain_markdown() && !container(&authors),
            "the author's keys only"
        );
        assert_eq!(authors.diagnostics, [], "no W111 for an author's key");
        assert!(
            matches!(authors.root.children[0], Node::Yaml(_)),
            "the frontmatter stays"
        );
        let compmd = parse(&format!("---\nmd:\n  version: 1\n---\n{box_}"), &markdown);
        assert!(!compmd.plain_markdown() && container(&compmd));
        let toml = parse(&format!("+++\n[md]\nversion = 1\n+++\n{box_}"), &markdown);
        assert!(!toml.plain_markdown());
        let host = ParseOptions {
            metadata: Some(serde_json::json!({ "md": {} })),
            plain_markdown: true,
        };
        assert!(
            !parse(box_, &host).plain_markdown(),
            "the host's metadata has `md`"
        );
        let unknown = ParseOptions {
            metadata: Some(serde_json::json!({ "title": "A" })),
            plain_markdown: true,
        };
        assert!(parse(box_, &unknown).plain_markdown());
        assert!(
            !parse(box_, &ParseOptions::default()).plain_markdown(),
            "a `.comp.md` file"
        );
    }
}
