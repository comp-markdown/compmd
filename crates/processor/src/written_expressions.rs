//! The expressions a document writes, where it writes them (spec §8): each param's `(…)` and each interpolation's
//! `{{…}}`, on component lines, in definition fences and in text, as the parser reads them. Computation happens only
//! there, so nothing in the frontmatter or in prose is an expression. They come from reading each line again, as the
//! syntax tokens do, so a line being typed has its expressions too.

use crate::resolve::Document;

/// An expression as the document writes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WrittenExpression {
    /// The document's code point its source starts at.
    pub(crate) start: usize,
    /// Its source: a param's expression without its parentheses, an interpolation's without its braces.
    pub(crate) source: String,
}

/// The expressions of `document`, in the order they start.
pub(crate) fn of(document: &Document) -> Vec<WrittenExpression> {
    crate::tokens::reader::read(document).expressions
}
