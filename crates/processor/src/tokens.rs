//! Syntax tokens: where each span of CompMD's own syntax is in a processed document, and what it is, by the names
//! every grammar and the language server share (`compmd-grammars`' `syntax/tokens.json`). A language server gives them
//! as semantic tokens, which know more than a grammar can: whether a name stands for a component the document can use.
//!
//! Like positional hints, the tokens are a lookup, not a pass: each line is read again as the parser reads it, once
//! the tree says it is text, so a line being typed, or one whose param list broke, has its tokens too.

pub(crate) mod reader;

use crate::diagnostic::Range;
use serde::Serialize;

/// What a span is: a token of `syntax/tokens.json` that a language server colours. Markdown's own spans, and the
/// punctuation a grammar's colours are enough for (brackets, braces, `=`), aren't tokens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TokenKind {
    /// A component line's marker and its form's sign: `:::`, `::`, an inline component's `:`, a closer's `/`, a
    /// continuation's `+`, a slot's `#`.
    ComponentMarker,
    /// A component's name, where it is used, closed or defined (`component=card`).
    ComponentName,
    /// A slot's name on its slot line.
    SlotName,
    /// A param's key, and a flag.
    ParamKey,
    /// A breakpoint before a param's key (`md` of `md:gap`).
    Breakpoint,
    /// A class param, `.wide`, with its dot.
    ParamClass,
    /// An id param, `#intro`, with its `#`.
    ParamId,
    /// A quoted param value, with its quotes; the interpolations in it are tokens of their own.
    ParamString,
    /// A bare param value.
    ParamBare,
    /// The parentheses around an expression param.
    ExpressionParen,
    /// An interpolation's `{{` and `}}`.
    InterpolationBrace,
    /// An interpolation's path, `md.doc.title`.
    InterpolationPath,
    /// The frontmatter's `md` key.
    FrontmatterKey,
}

impl TokenKind {
    /// Every kind.
    pub const ALL: [Self; 13] = [
        Self::ComponentMarker,
        Self::ComponentName,
        Self::SlotName,
        Self::ParamKey,
        Self::Breakpoint,
        Self::ParamClass,
        Self::ParamId,
        Self::ParamString,
        Self::ParamBare,
        Self::ExpressionParen,
        Self::InterpolationBrace,
        Self::InterpolationPath,
        Self::FrontmatterKey,
    ];

    /// Its name in `syntax/tokens.json`, `component-name`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ComponentMarker => "component-marker",
            Self::ComponentName => "component-name",
            Self::SlotName => "slot-name",
            Self::ParamKey => "param-key",
            Self::Breakpoint => "breakpoint",
            Self::ParamClass => "param-class",
            Self::ParamId => "param-id",
            Self::ParamString => "param-string",
            Self::ParamBare => "param-bare",
            Self::ExpressionParen => "expression-paren",
            Self::InterpolationBrace => "interpolation-brace",
            Self::InterpolationPath => "interpolation-path",
            Self::FrontmatterKey => "frontmatter-key",
        }
    }
}

/// A span of CompMD's syntax: what it is and where. A token is on one line; its range's columns count code points, its
/// end is past its last code point.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SyntaxToken {
    /// What it is.
    pub kind: TokenKind,
    /// Where it is.
    pub range: Range,
    /// For a component's name, that it stands for no component the document can use (F002).
    pub unknown: bool,
}

/// The tokens of `document`, in the order they start.
pub(crate) fn of(document: &crate::resolve::Document) -> Vec<SyntaxToken> {
    reader::read(document).tokens
}

#[cfg(test)]
mod tests;
