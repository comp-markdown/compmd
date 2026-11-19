// Code blocks highlighted by Shiki, in both page modes (in the page's frame and use frames, or in the host's page), in
// GitHub's light and dark themes. Shiki's core and its regex engine load the first time a page has a code block:
// Oniguruma where the page compiles WebAssembly, the JavaScript engine in Chromium and where it doesn't (`engineFor`);
// each language's grammar loads the first time a block of it shows. It gives each token's text and its colour in each
// theme (`tokens.ts`); the renderer draws them.

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

/** The regex engine Shiki runs grammars with. */
export type Engine = "oniguruma" | "javascript";

/** The smallest WebAssembly module: its header alone. */
const EMPTY_MODULE = new Uint8Array([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);

/**
 * Whether this page may compile WebAssembly: a policy without `'wasm-unsafe-eval'` (or `'unsafe-eval'`) refuses it,
 * as the frames' does. Asked with an empty module, so Oniguruma's 622 KB load only where it can run.
 */
export async function compilesWebAssembly(): Promise<boolean> {
  try {
    await WebAssembly.compile(EMPTY_MODULE);
    return true;
  } catch {
    return false;
  }
}

/** What the engine is chosen by: the page's browser, and whether the page compiles WebAssembly. */
export interface EnginePage {
  /** Whether the browser is Chromium's, which runs the JavaScript engine fast. */
  chromium: boolean;
  compiles(): Promise<boolean>;
}

/**
 * This page: Chromium's browsers alone have `navigator.userAgentData`, which a secure context has; a Chromium that
 * hides it is taken for another browser, which costs time, never highlighting.
 */
const THIS_PAGE: EnginePage = {
  chromium: typeof navigator !== "undefined" && "userAgentData" in navigator,
  compiles: compilesWebAssembly,
};

/**
 * The engine a highlighter runs (docs/webassembly-in-the-frames.md, decision 2): Oniguruma, as WebAssembly, when it
 * may (`webAssembly`), the browser isn't Chromium's and the page compiles it; otherwise the JavaScript engine. In
 * WebKit the JavaScript engine compiles each of a grammar's regexes the first time it runs, which blocks the page for
 * a second on a first TypeScript block, where Oniguruma takes 60 ms; in Chromium the JavaScript engine is as fast,
 * and Oniguruma's compiling only costs more, in each frame.
 */
export async function engineFor(webAssembly: boolean, page: EnginePage = THIS_PAGE): Promise<Engine> {
  return webAssembly && !page.chromium && (await page.compiles()) ? "oniguruma" : "javascript";
}

/** Loads Shiki's core, `engine` and the two themes, and Markdown with CompMD's syntax. */
async function loadShikis(engine: Engine): Promise<Shikis> {
  const [{ createHighlighterCore }, light, dark, markdown] = await Promise.all([import("@shikijs/core"), import("@shikijs/themes/github-light"), import("@shikijs/themes/github-dark"), grammarOf("markdown")]);
  const regexes =
    engine === "oniguruma"
      ? import("@shikijs/engine-oniguruma").then(({ createOnigurumaEngine }) => createOnigurumaEngine(import("@shikijs/engine-oniguruma/wasm-inlined")))
      : // Forgiving: a pattern the engine can't translate matches nothing, instead of failing the whole grammar.
        import("@shikijs/engine-javascript").then(({ createJavaScriptRegexEngine }) => createJavaScriptRegexEngine({ forgiving: true }));
  const shiki = (langs: LanguageRegistration[]) => createHighlighterCore({ themes: [light.default, dark.default], langs, engine: regexes });
  const [md, others] = await Promise.all([shiki([...markdown, COMPMD]), shiki([])]);
  return { markdown: md, others };
}

/** How a highlighter runs. */
export interface ShikiOptions {
  /**
   * Oniguruma, as WebAssembly, where the page compiles it and the browser isn't Chromium's (`engineFor`); the
   * JavaScript engine otherwise, as without. A page allows it with `'wasm-unsafe-eval'` in its policy's `script-src`.
   * Its WebAssembly is inlined in a chunk of its own (230 KB gzipped), loaded with the first code block where it is used.
   */
  webAssembly?: boolean;
}

/**
 * The highlighter both page modes use: Shiki, loaded the first time a page has a code block. Each call makes a Shiki
 * of its own, which loads each grammar again: a host that renders many pages makes one and gives it to them all.
 *
 * @example
 * const highlight = shikiHighlight({ webAssembly: true });
 * for (const run of runs) createPage(run.element, { highlight }).render(run.tree);
 */
export function shikiHighlight(options: ShikiOptions = {}): CodeHighlighter {
  return highlightWith(async () => loadShikis(await engineFor(options.webAssembly === true)));
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
