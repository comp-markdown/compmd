// Code blocks highlighted by Shiki, in both page modes (in the page's frame and use frames, or in the host's page), in
// GitHub's light and dark themes. Shiki's core and its JavaScript regex engine load the first time a page has a code
// block, so a strict frame's policy needs no WebAssembly; each language's grammar loads the first time a block of it
// shows. It gives each token's text and its colour in each theme (`tokens.ts`); the renderer draws them.

import type { CodeHighlighter } from "@compmd/render";
import type { HighlighterCore, LanguageRegistration } from "@shikijs/core";
import { COMPMD, splitYaml } from "./grammar.ts";
import { grammarOf, languageOf } from "./languages.ts";
import { codeTokens, type ShikiLines as Lines } from "./tokens.ts";

/** Two Shikis: Markdown's, with CompMD's syntax injected, and the other languages', YAML among them (`grammar.ts`). */
export interface Shikis {
  markdown: HighlighterCore;
  others: HighlighterCore;
}

const THEMES = { light: "github-light", dark: "github-dark" } as const;

/** Loads Shiki's core, its JavaScript engine and the two themes, and Markdown with CompMD's syntax. */
async function loadShikis(): Promise<Shikis> {
  const [{ createHighlighterCore }, { createJavaScriptRegexEngine }, light, dark, markdown] = await Promise.all([
    import("@shikijs/core"),
    import("@shikijs/engine-javascript"),
    import("@shikijs/themes/github-light"),
    import("@shikijs/themes/github-dark"),
    grammarOf("markdown"),
  ]);
  // Forgiving: a pattern the engine can't translate matches nothing, instead of failing the whole grammar.
  const shiki = (langs: LanguageRegistration[]) => createHighlighterCore({ themes: [light.default, dark.default], langs, engine: createJavaScriptRegexEngine({ forgiving: true }) });
  const [md, others] = await Promise.all([shiki([...markdown, COMPMD]), shiki([])]);
  return { markdown: md, others };
}

/** The highlighter both page modes use: Shiki, loaded the first time a page has a code block. */
export function shikiHighlight(): CodeHighlighter {
  return highlightWith(loadShikis);
}

/**
 * A highlighter for the page, with the Shikis `load` gives, loaded once. It gives a block's tokens once Shiki and the
 * block's grammar have loaded, none for a language it doesn't highlight, and a rejected promise when they can't be
 * had: the renderer keeps the block's text then.
 *
 * @example
 * serveFrame(port, { container, highlight: highlightWith(loadMyShikis) });
 */
export function highlightWith(load: () => Promise<Shikis>): CodeHighlighter {
  let shikis: Promise<Shikis> | undefined;
  /** The grammars the other languages' Shiki has, or is loading, by name. */
  const loaded = new Map<string, Promise<void>>();

  const tokens = async (code: string, name: string): Promise<Lines> => {
    const { others } = await (shikis ??= load());
    let grammar = loaded.get(name);
    if (!grammar) {
      grammar = grammarOf(name).then((langs) => others.loadLanguage(...langs));
      loaded.set(name, grammar);
    }
    await grammar;
    return others.codeToTokensWithThemes(code, { lang: name, themes: THEMES });
  };
  /** Markdown, with its YAML (frontmatter, or a definition's signature) highlighted as YAML. */
  const markdown = async (code: string, frontmatter: boolean): Promise<Lines> => {
    const [yaml, rest] = splitYaml(code, frontmatter);
    const { markdown: md } = await (shikis ??= load());
    const head = yaml === "" ? [] : await tokens(yaml, "yaml");
    const body = yaml === code ? [] : md.codeToTokensWithThemes(rest, { lang: "markdown", themes: THEMES });
    return [...head, ...body];
  };
  const linesOf = (code: string, language: string): Promise<Lines> | undefined => {
    const name = language === "compmd" ? "compmd" : languageOf(language);
    if (name === "markdown") return markdown(code, true);
    if (name === "compmd") return markdown(code, false);
    return name === undefined ? undefined : tokens(code, name);
  };

  return (code, language) => linesOf(code, language)?.then(codeTokens);
}
