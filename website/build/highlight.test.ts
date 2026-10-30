// @vitest-environment node
import { describe, expect, it } from "vitest";
import { highlighter, splitYaml } from "./highlight.ts";

const highlight = await highlighter();
/** The light colors of `html`'s text, by the text without the spaces Shiki puts in a token beside it. */
const colors = (html: string): Record<string, string> =>
  Object.fromEntries([...html.matchAll(/<span style="color:(#\w+);[^"]*">([^<]*)<\/span>/g)].map(([, color = "", text = ""]) => [text.trim(), color]));

describe("splitYaml", () => {
  it("splits frontmatter, and a signature before its body", () => {
    expect(splitYaml("---\na: 1\n---\n# T", true)).toEqual(["---\na: 1\n---", "# T"]);
    expect(splitYaml("# T\n\n---\n", true)).toEqual(["", "# T\n\n---\n"]);
    expect(splitYaml("params:\n  a: string\n---\n**{{a}}**", false)).toEqual(["params:\n  a: string\n---", "**{{a}}**"]);
    expect(splitYaml("params:\n  a: string", false)).toEqual(["params:\n  a: string", ""]);
  });
});

describe("highlighter", () => {
  it("shows CompMD's syntax in Markdown, and the frontmatter as YAML", () => {
    const html = highlight('---\nheading: A\n---\n::: card {title="Setup" gap=m}\n- {{md.doc.heading}}\n::: /card', "md") ?? "";
    const color = colors(html);
    expect(color["heading"]).toBe("#22863A");
    expect(color["card"]).toBe("#22863A");
    expect(color["gap"]).toBe("#6F42C1");
    expect(color['"Setup"']).toBe("#032F62");
    expect(color["{{md.doc.heading}}"]).toBe("#005CC5");
    expect(html.match(/<pre/g)).toHaveLength(1);
    expect(html.match(/class="line"/g)).toHaveLength(6);
  });

  it("shows a definition's signature as YAML and its body as Markdown", () => {
    const color = colors(highlight("params:\n  a: string\n---\n::: box {gap=s}\n{{a}}\n::: /box", "compmd") ?? "");
    expect(color["params"]).toBe("#22863A");
    expect(color["box"]).toBe("#22863A");
  });

  it("highlights the other languages, and leaves the rest", () => {
    expect(highlight("rule = %x41", "abnf")).toContain("#6F42C1");
    expect(highlight("const a = 1;", "js")).toContain("shiki");
    expect(highlight("x", "cobol")).toBeUndefined();
  });
});
