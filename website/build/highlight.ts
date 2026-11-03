// Code highlighted at build time by Shiki, in GitHub's light and dark themes, for the spec page.
//
// Markdown examples show CompMD's syntax through the grammar the sandbox's highlighter injects into Markdown's
// (`@compmd/sandbox/grammar`). Shiki loses that injection when YAML's grammar is loaded beside Markdown's, so Markdown
// has a highlighter of its own, and YAML is highlighted by the other: a Markdown example's frontmatter, and a `compmd`
// definition's signature, are YAML, and the rest is Markdown.
import { COMPMD, splitYaml } from "@compmd/sandbox/grammar";
import { createHighlighter, type Highlighter } from "shiki";
import { ABNF } from "./grammars.ts";

/** Highlights `code` written in the fence language `language`; `undefined` for a language it doesn't highlight. */
export type Highlight = (code: string, language: string) => string | undefined;

/** Fence languages as Shiki names them, other than Markdown's. */
const OTHERS: Record<string, string> = { yaml: "yaml", js: "javascript", ts: "typescript", json: "json", sh: "shellscript", svelte: "svelte", abnf: "abnf" };

const THEMES = { themes: { light: "github-light", dark: "github-dark" }, defaultColor: "light" } as const;

/** One `<pre>` with the lines of each of `blocks` in turn, each a `<pre>` as Shiki writes it. */
function joined(blocks: string[]): string {
  const [first] = blocks;
  if (first === undefined || blocks.length === 1) return first ?? "";
  const lines = blocks.map((block) => /<code>([\s\S]*)<\/code>/.exec(block)?.[1] ?? "");
  return first.replace(/<code>[\s\S]*<\/code>/, () => `<code>${lines.join("\n")}</code>`);
}

/** The highlighter; it loads its grammars once. */
export async function highlighter(): Promise<Highlight> {
  const [markdown, others] = await Promise.all([
    createHighlighter({ themes: ["github-light", "github-dark"], langs: ["markdown", COMPMD] }),
    createHighlighter({ themes: ["github-light", "github-dark"], langs: ["yaml", "javascript", "typescript", "json", "shellscript", "svelte", ABNF] }),
  ]);
  const html = (shiki: Highlighter, lang: string) => (code: string) => shiki.codeToHtml(code, { lang, ...THEMES });
  const [yaml, md] = [html(others, "yaml"), html(markdown, "markdown")];
  const parts = (head: string, rest: string) => joined([head ? yaml(head) : "", rest ? md(rest) : ""].filter(Boolean));
  return (code, language) => {
    if (language === "md" || language === "markdown") return parts(...splitYaml(code, true));
    if (language === "compmd") return parts(...splitYaml(code, false));
    const lang = OTHERS[language];
    return lang === undefined ? undefined : html(others, lang)(code);
  };
}
