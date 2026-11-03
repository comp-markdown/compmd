// @vitest-environment happy-dom
import { describe, expect, it, vi } from "vitest";
import { highlightWith, shikiHighlight, type Shikis } from "./highlight.ts";
import { LANGUAGE_NAMES } from "./languages.ts";

/** A block's `<code>`, holding its text, as the renderer gives it. */
const codeOf = (text: string) => Object.assign(document.createElement("code"), { textContent: text });
/** The texts of `target`'s tokens. */
const tokens = (target: HTMLElement) => [...target.querySelectorAll("span")].map((span) => span.textContent);

describe("shikiHighlight", () => {
  it("draws a block's tokens, its text as it was, loading Shiki once", async () => {
    const highlight = shikiHighlight();
    const [first, second] = [codeOf("const a = 1;"), codeOf("print('hi')")];
    highlight("const a = 1;", "js", first);
    highlight("print('hi')", "python", second);
    expect(first.textContent).toBe("const a = 1;");
    await vi.waitFor(() => {
      expect(tokens(first)).toContain("const");
      expect(tokens(second)).toContain("print");
    });
    expect(first.textContent).toBe("const a = 1;");
    expect(second.textContent).toBe("print('hi')");
  });

  it("shows CompMD's syntax in Markdown, its frontmatter as YAML, and a definition's signature as YAML", async () => {
    const highlight = shikiHighlight();
    const markdown = '---\nheading: A\n---\n::: card {title="Setup" gap=m}\n- {{md.doc.heading}}\n::: /card';
    const definition = "params:\n  a: string\n---\n::: box {gap=s}\n{{a}}\n::: /box";
    const [md, compmd] = [codeOf(markdown), codeOf(definition)];
    highlight(markdown, "md", md);
    highlight(definition, "compmd", compmd);
    await vi.waitFor(() => {
      expect(tokens(md)).toEqual(expect.arrayContaining(["heading", "card", "gap", "{{md.doc.heading}}"]));
      expect(tokens(compmd)).toEqual(expect.arrayContaining(["params", "box"]));
    });
    expect(md.textContent).toBe(markdown);
    expect(compmd.textContent).toBe(definition);
  });

  it("leaves a block of a language it doesn't highlight, or of none, as it is", async () => {
    const highlight = shikiHighlight();
    const [cobol, none] = [codeOf("DISPLAY 'HI'."), codeOf("plain")];
    highlight("DISPLAY 'HI'.", "cobol", cobol);
    highlight("plain", "", none);
    await new Promise((resolve) => setTimeout(resolve, 100));
    expect([cobol.innerHTML, none.innerHTML]).toEqual(["DISPLAY 'HI'.", "plain"]);
  });

  it("loads and draws every language it lists, with the JavaScript engine", { timeout: 30_000 }, async () => {
    const highlight = shikiHighlight();
    const blocks = LANGUAGE_NAMES.map((name) => [name, codeOf("x = 1\ny")] as const);
    for (const [name, target] of blocks) highlight("x = 1\ny", name, target);
    await vi.waitFor(
      () => {
        expect(blocks.filter(([, target]) => target.querySelector("span") === null).map(([name]) => name)).toEqual([]);
      },
      { timeout: 25_000 },
    );
    for (const [, target] of blocks) expect(target.textContent).toBe("x = 1\ny");
  });
});

describe("highlightWith", () => {
  it("draws only a block's newest text, whichever tokens come first", async () => {
    const highlight = shikiHighlight();
    const target = codeOf("let older = 1;");
    // TypeScript's grammar loads after JavaScript's, so the older block's tokens could come last.
    highlight("let older = 1;", "ts", target);
    target.textContent = "let newer = 2;";
    highlight("let newer = 2;", "js", target);
    await vi.waitFor(() => {
      expect(tokens(target).some((token) => token.includes("newer"))).toBe(true);
    });
    await new Promise((resolve) => setTimeout(resolve, 200));
    expect(target.textContent).toBe("let newer = 2;");
  });

  it("keeps a block's text when Shiki doesn't load", async () => {
    const highlight = highlightWith((): Promise<Shikis> => Promise.reject(new Error("offline")));
    const target = codeOf("const a = 1;");
    highlight("const a = 1;", "js", target);
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(target.innerHTML).toBe("const a = 1;");
  });
});
