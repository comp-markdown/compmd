// The ways to get Shiki's tokens, as the variants try them: the JavaScript engine the sandbox uses today, Shiki's
// precompiled languages with its raw JavaScript engine (the regexes converted ahead), and Oniguruma as WebAssembly.
// Each is GitHub's light and dark themes, TypeScript, as the sandbox loads them.

import type { HighlighterCore, ThemedTokenWithVariants } from "@shikijs/core";

/** A block's lines, each a list of tokens with their colours in each theme. */
export type Lines = ThemedTokenWithVariants[][];

export type Engine = "js" | "precompiled" | "oniguruma";

export const THEMES = { light: "github-light", dark: "github-dark" } as const;

/** Shiki with TypeScript, by `engine`, and how long it took to make. */
export async function loadShiki(engine: Engine): Promise<{ shiki: HighlighterCore; ms: number }> {
  const started = performance.now();
  const [{ createHighlighterCore }, light, dark] = await Promise.all([import("@shikijs/core"), import("@shikijs/themes/github-light"), import("@shikijs/themes/github-dark")]);
  const themes = [light.default, dark.default];
  let shiki: HighlighterCore;
  if (engine === "precompiled") {
    const [{ createJavaScriptRawEngine }, typescript] = await Promise.all([import("@shikijs/engine-javascript/raw"), import("@shikijs/langs-precompiled/typescript")]);
    shiki = await createHighlighterCore({ themes, langs: [typescript.default], engine: createJavaScriptRawEngine() });
  } else if (engine === "oniguruma") {
    const [{ createOnigurumaEngine }, typescript] = await Promise.all([import("@shikijs/engine-oniguruma"), import("@shikijs/langs/typescript")]);
    shiki = await createHighlighterCore({ themes, langs: [typescript.default], engine: createOnigurumaEngine(import("@shikijs/engine-oniguruma/wasm-inlined")) });
  } else {
    const [{ createJavaScriptRegexEngine }, typescript] = await Promise.all([import("@shikijs/engine-javascript"), import("@shikijs/langs/typescript")]);
    shiki = await createHighlighterCore({ themes, langs: [typescript.default], engine: createJavaScriptRegexEngine({ forgiving: true }) });
  }
  return { shiki, ms: performance.now() - started };
}

/** `code`'s tokens, as the sandbox's highlighter asks Shiki for them. */
export const tokensOf = (shiki: HighlighterCore, code: string): Lines => shiki.codeToTokensWithThemes(code, { lang: "typescript", themes: THEMES });
