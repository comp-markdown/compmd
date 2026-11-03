// Code blocks highlighted by Shiki, in both page modes (in the page's frame and use frames, or in the host's page), in
// GitHub's light and dark themes. Shiki's core and its JavaScript regex engine load the first time a page has a code
// block, so a strict frame's policy needs no WebAssembly; each language's grammar loads the first time a block of it
// shows. Each token's colour is `light-dark(light, dark)`, set through the CSSOM, which a frame's policy allows, and
// which follows the `color-scheme` a frame takes from the page around it.

import type { CodeHighlighter } from "@compmd/render";
import type { HighlighterCore, LanguageRegistration, ThemedTokenWithVariants } from "@shikijs/core";
import { COMPMD, splitYaml } from "./grammar.ts";
import { grammarOf, languageOf } from "./languages.ts";

/** A block's lines, each a list of tokens with their colours in each theme. */
type Lines = ThemedTokenWithVariants[][];

/** Two Shikis: Markdown's, with CompMD's syntax injected, and the other languages', YAML among them (`grammar.ts`). */
export interface Shikis {
  markdown: HighlighterCore;
  others: HighlighterCore;
}

const THEMES = { light: "github-light", dark: "github-dark" } as const;

/** Shiki's `FontStyle` bits. */
const ITALIC = 1;
const BOLD = 2;
const UNDERLINE = 4;

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
 * A highlighter for the page, with the Shikis `load` gives, loaded once. A block of a language it doesn't highlight
 * stays as it is; one it does keeps its text until its tokens are there, and keeps it if they can't be had.
 *
 * @example
 * serveFrame(port, { container, highlight: highlightWith(loadMyShikis) });
 */
export function highlightWith(load: () => Promise<Shikis>): CodeHighlighter {
  let shikis: Promise<Shikis> | undefined;
  /** The grammars the other languages' Shiki has, or is loading, by name. */
  const loaded = new Map<string, Promise<void>>();
  /** What each block is to show last, so tokens that come after newer ones are dropped. */
  const latest = new WeakMap<HTMLElement, string>();

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

  return (code, language, target) => {
    const shown = `${language}\u0000${code}`;
    latest.set(target, shown);
    const lines = linesOf(code, language);
    void lines?.then(
      (drawn) => {
        if (latest.get(target) === shown) draw(drawn, code, target);
      },
      () => undefined,
    );
  };
}

/** Puts `lines` in `target` as spans, unless they aren't `code`'s text: then it keeps the text. */
function draw(lines: Lines, code: string, target: HTMLElement): void {
  if (lines.map((line) => line.map(({ content }) => content).join("")).join("\n") !== code) return;
  const document = target.ownerDocument;
  const parts: Node[] = [];
  for (const [i, line] of lines.entries()) {
    if (i > 0) parts.push(document.createTextNode("\n"));
    for (const { content, variants } of line) {
      const span = document.createElement("span");
      span.textContent = content;
      const { light, dark } = variants;
      if (light?.color && dark?.color) span.style.setProperty("color", `light-dark(${light.color}, ${dark.color})`);
      const style = light?.fontStyle ?? 0;
      if (style & ITALIC) span.style.setProperty("font-style", "italic");
      if (style & BOLD) span.style.setProperty("font-weight", "bold");
      if (style & UNDERLINE) span.style.setProperty("text-decoration", "underline");
      parts.push(span);
    }
  }
  target.replaceChildren(...parts);
}
