// Code highlighted at build time by Shiki, in GitHub's light and dark themes (docs/blog-tool.md, decision 5), as the
// renderer's `highlight` takes it: each code block's `<code>` gets Shiki's lines, its `<pre>` Shiki's colours.
//
// Markdown and CompMD show CompMD's syntax through the grammar the sandbox's highlighter injects into Markdown's
// (`@compmd/sandbox/grammar`). Shiki loses that injection when YAML's grammar is loaded beside Markdown's, so Markdown
// has a highlighter of its own, and a Markdown example's frontmatter is highlighted as YAML by the other.

import type { CodeHighlighter } from "@compmd/render";
import { COMPMD, splitYaml } from "@compmd/sandbox/grammar";
import { bundledLanguages, createHighlighter, type BundledLanguage, type Highlighter } from "shiki";

const THEMES = { themes: { light: "github-light", dark: "github-dark" }, defaultColor: "light" } as const;
const MARKDOWN = new Set(["md", "markdown", "compmd"]);

/** A highlighter, and the languages it should load before a render that uses them. */
export interface Highlight {
  /** The renderer's `highlight`: a block in a language not loaded keeps its text as it is. */
  highlight: CodeHighlighter;
  /** Loads `languages`, fence languages as written, those Shiki has. */
  load(languages: Iterable<string>): Promise<void>;
}

/** Shiki's name of a fence language it has: its own names and aliases (`ts`, `sh`). */
const known = (language: string): BundledLanguage | undefined => (language in bundledLanguages ? (language as BundledLanguage) : undefined);

/** Shiki's `<pre>` for `code`: its class and style, and its `<code>`'s content. */
function parts(shiki: Highlighter, code: string, lang: string): { style: string; lines: string } {
  const html = shiki.codeToHtml(code, { lang, ...THEMES });
  return { style: /<pre[^>]*style="([^"]*)"/.exec(html)?.[1] ?? "", lines: /<code>([\s\S]*)<\/code>/.exec(html)?.[1] ?? "" };
}

let shared: Promise<Highlight> | undefined;

/**
 * The highlighter, one for every build in the process (`cblo dev` builds for each request); it loads Markdown's
 * grammars once, and each other language when a post first needs it.
 */
export function highlighter(): Promise<Highlight> {
  shared ??= create();
  return shared;
}

/** A new highlighter. */
async function create(): Promise<Highlight> {
  const [markdown, others] = await Promise.all([
    createHighlighter({ themes: ["github-light", "github-dark"], langs: ["markdown", COMPMD] }),
    createHighlighter({ themes: ["github-light", "github-dark"], langs: ["yaml"] }),
  ]);
  const loaded = new Set(others.getLoadedLanguages());
  const write = (target: HTMLElement, { style, lines }: { style: string; lines: string }) => {
    target.innerHTML = lines;
    const pre = target.parentElement;
    if (pre?.tagName === "PRE") {
      pre.classList.add("shiki");
      pre.setAttribute("style", style);
    }
  };
  return {
    highlight(code, language, target) {
      if (MARKDOWN.has(language)) {
        const [head, rest] = splitYaml(code, language !== "compmd");
        const blocks = [head ? parts(others, head, "yaml") : undefined, rest ? parts(markdown, rest, "markdown") : undefined].filter((block) => block !== undefined);
        const [first] = blocks;
        if (first) write(target, { style: first.style, lines: blocks.map(({ lines }) => lines).join("\n") });
        return;
      }
      const lang = known(language);
      if (lang !== undefined && loaded.has(lang)) write(target, parts(others, code, lang));
    },
    async load(languages) {
      for (const language of languages) {
        const lang = known(language);
        if (lang === undefined || loaded.has(lang) || MARKDOWN.has(language)) continue;
        await others.loadLanguage(lang);
        for (const name of others.getLoadedLanguages()) loaded.add(name);
      }
    },
  };
}
