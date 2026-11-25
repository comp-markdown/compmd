//! Semantic tokens ([the language server], *Semantic tokens*): the processor's syntax tokens by the shared token names'
//! LSP types (`compmd-grammars`' `syntax/tokens.json`), a name that stands for no component with the modifier
//! `unknown` (Q6).
//!
//! [the language server]: https://github.com/comp-markdown/compmd/blob/main/docs/language-server.md#semantic-tokens

use compmd::tokens::{SyntaxToken, TokenKind};
use lsp_types::{
    Position, Range, SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens,
    SemanticTokensLegend,
};

use crate::encoding::Encoding;

/// The types the tokens are given, in the legend's order.
const TYPES: [SemanticTokenType; 8] = [
    SemanticTokenType::OPERATOR,
    SemanticTokenType::TYPE,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::MODIFIER,
    SemanticTokenType::STRING,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::NAMESPACE,
];

/// A name that stands for no component (F002), which themes can dim or underline.
const UNKNOWN: &str = "unknown";

/// The legend the server's capabilities give.
pub(crate) fn legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TYPES.to_vec(),
        token_modifiers: vec![SemanticTokenModifier::new(UNKNOWN)],
    }
}

/// The LSP type of `kind`, as `syntax/tokens.json`'s `semantic` names it.
pub(crate) fn type_of(kind: TokenKind) -> SemanticTokenType {
    match kind {
        TokenKind::ComponentMarker | TokenKind::ExpressionParen | TokenKind::InterpolationBrace => {
            SemanticTokenType::OPERATOR
        }
        TokenKind::ComponentName => SemanticTokenType::TYPE,
        TokenKind::SlotName => SemanticTokenType::PARAMETER,
        TokenKind::ParamKey | TokenKind::ParamClass | TokenKind::ParamId => {
            SemanticTokenType::PROPERTY
        }
        TokenKind::Breakpoint => SemanticTokenType::MODIFIER,
        TokenKind::ParamString | TokenKind::ParamBare => SemanticTokenType::STRING,
        TokenKind::InterpolationPath => SemanticTokenType::VARIABLE,
        TokenKind::FrontmatterKey => SemanticTokenType::NAMESPACE,
    }
}

/// The index of `kind`'s type in the legend.
fn index_of(kind: TokenKind) -> u32 {
    let token_type = type_of(kind);
    TYPES
        .iter()
        .position(|known| *known == token_type)
        .and_then(|index| u32::try_from(index).ok())
        .unwrap_or(0)
}

/// `tokens` of the document whose text is `text`, those starting in `range` when one is given, as LSP encodes them:
/// each relative to the one before, in `encoding`'s units.
pub(crate) fn encode(
    tokens: &[SyntaxToken],
    text: &str,
    encoding: Encoding,
    range: Option<Range>,
) -> SemanticTokens {
    let mut data = Vec::new();
    let mut previous = Position {
        line: 0,
        character: 0,
    };
    for token in tokens {
        let Range { start, end } = encoding.range(text, &token.range);
        if range.is_some_and(|range| start < range.start || start >= range.end) {
            continue;
        }
        let delta_line = start.line - previous.line;
        data.push(SemanticToken {
            delta_line,
            delta_start: if delta_line == 0 {
                start.character - previous.character
            } else {
                start.character
            },
            length: end.character - start.character,
            token_type: index_of(token.kind),
            token_modifiers_bitset: u32::from(token.unknown),
        });
        previous = start;
    }
    SemanticTokens {
        result_id: None,
        data,
    }
}

#[cfg(test)]
mod tests;
