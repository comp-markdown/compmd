// Shiki's tokens as the renderer's highlighter gives them (`@compmd/render`'s `CodeHighlighter`): each token's text,
// its colour in the light and the dark theme, and its font style.

import type { CodeLines, CodeToken } from "@compmd/render";
import type { ThemedTokenWithVariants } from "@shikijs/core";

/** A block's lines as Shiki's `codeToTokensWithThemes` gives them, with the themes `light` and `dark`. */
export type ShikiLines = ThemedTokenWithVariants[][];

/** Shiki's `FontStyle` bits. */
const ITALIC = 1;
const BOLD = 2;
const UNDERLINE = 4;

/**
 * `lines`, Shiki's tokens in the themes `light` and `dark`, as the renderer's: a token's colour when both themes give
 * one, and the light theme's font style.
 *
 * @example
 * const highlight: CodeHighlighter = (code, language) => codeTokens(shiki.codeToTokensWithThemes(code, { lang: language, themes: { light: "github-light", dark: "github-dark" } }));
 */
export function codeTokens(lines: ShikiLines): CodeLines {
  return lines.map((line) =>
    line.map(({ content, variants: { light, dark } }): CodeToken => {
      const style = light?.fontStyle ?? 0;
      return {
        text: content,
        ...(light?.color && dark?.color ? { color: { light: light.color, dark: dark.color } } : {}),
        ...(style & ITALIC ? { italic: true } : {}),
        ...(style & BOLD ? { bold: true } : {}),
        ...(style & UNDERLINE ? { underline: true } : {}),
      };
    }),
  );
}
