// @vitest-environment happy-dom
import { CODE_LINE } from "@compmd/render";
import { describe, expect, it, vi } from "vitest";
import { highlightWith, shikiHighlight, type Shikis } from "./highlight.ts";
import { LANGUAGE_NAMES } from "./languages.ts";

/** `target` holding `text` as the renderer draws it: an element per line, each with its line break but the last. */
function drawLines(target: HTMLElement, text: string): HTMLElement {
  const lines = text.split("\n");
  target.replaceChildren(...lines.map((line, i) => Object.assign(document.createElement("span"), { className: CODE_LINE, textContent: i < lines.length - 1 ? `${line}\n` : line })));
  return target;
}
/** A block's `<code>`, holding its text, as the renderer gives it. */
const codeOf = (text: string) => drawLines(document.createElement("code"), text);
/** The texts of `target`'s tokens, the spans in its lines. */
const tokens = (target: HTMLElement) => [...target.querySelectorAll(`.${CODE_LINE} > span`)].map((span) => span.textContent);

/**
 * How long to wait for Shiki and a grammar to load: well under a second alone, but many seconds while every package's
 * tests run at once; and the test's own limit above it.
 */
const LOADED = { timeout: 10_000 };
const TEST = { timeout: 15_000 };

describe("shikiHighlight", () => {
  it("draws a block's tokens, its text as it was, loading Shiki once", TEST, async () => {
    const highlight = shikiHighlight();
    const [first, second] = [codeOf("const a = 1;"), codeOf("print('hi')")];
    highlight("const a = 1;", "js", first);
    highlight("print('hi')", "python", second);
    expect(first.textContent).toBe("const a = 1;");
    await vi.waitFor(() => {
      expect(tokens(first)).toContain("const");
      expect(tokens(second)).toContain("print");
    }, LOADED);
    expect(first.textContent).toBe("const a = 1;");
    expect(second.textContent).toBe("print('hi')");
  });

  it("shows CompMD's syntax in Markdown, its frontmatter as YAML, and a definition's signature as YAML", TEST, async () => {
    const highlight = shikiHighlight();
    const markdown = '---\nheading: A\n---\n::: card {title="Setup" gap=m}\n- {{md.doc.heading}}\n::: /card';
    const definition = "params:\n  a: string\n---\n::: box {gap=s}\n{{a}}\n::: /box";
    const [md, compmd] = [codeOf(markdown), codeOf(definition)];
    highlight(markdown, "md", md);
    highlight(definition, "compmd", compmd);
    await vi.waitFor(() => {
      expect(tokens(md)).toEqual(expect.arrayContaining(["heading", "card", "gap", "{{md.doc.heading}}"]));
      expect(tokens(compmd)).toEqual(expect.arrayContaining(["params", "box"]));
    }, LOADED);
    expect(md.textContent).toBe(markdown);
    expect(compmd.textContent).toBe(definition);
  });

  it("leaves a block of a language it doesn't highlight, or of none, as it is", async () => {
    const highlight = shikiHighlight();
    const [cobol, none] = [codeOf("DISPLAY 'HI'."), codeOf("plain")];
    highlight("DISPLAY 'HI'.", "cobol", cobol);
    highlight("plain", "", none);
    await new Promise((resolve) => setTimeout(resolve, 100));
    expect([tokens(cobol), tokens(none)]).toEqual([[], []]);
    expect([cobol.textContent, none.textContent]).toEqual(["DISPLAY 'HI'.", "plain"]);
  });

  it("puts each line's tokens in its line's element, which keeps its attributes, a blank line's too", TEST, async () => {
    const highlight = shikiHighlight();
    const text = "const a = 1;\n\nconst b = 2;";
    const target = codeOf(text);
    for (const [i, line] of [...target.children].entries()) line.setAttribute("data-line", String(4 + i));
    const lines = [...target.children];
    highlight(text, "js", target);
    await vi.waitFor(() => {
      expect(tokens(target)).toContain("const");
    }, LOADED);
    expect([...target.children]).toEqual(lines);
    expect(lines.map((line) => [line.getAttribute("data-line"), line.textContent])).toEqual([
      ["4", "const a = 1;\n"],
      ["5", "\n"],
      ["6", "const b = 2;"],
    ]);
    expect(target.textContent).toBe(text);
  });

  it("loads and draws every language it lists, with the JavaScript engine", { timeout: 30_000 }, async () => {
    const highlight = shikiHighlight();
    const blocks = LANGUAGE_NAMES.map((name) => [name, codeOf("x = 1\ny")] as const);
    for (const [name, target] of blocks) highlight("x = 1\ny", name, target);
    await vi.waitFor(
      () => {
        expect(blocks.filter(([, target]) => tokens(target).length === 0).map(([name]) => name)).toEqual([]);
      },
      { timeout: 25_000 },
    );
    for (const [, target] of blocks) expect(target.textContent).toBe("x = 1\ny");
  });
});

describe("highlightWith", () => {
  it("draws only a block's newest text, whichever tokens come first", TEST, async () => {
    const highlight = shikiHighlight();
    const target = codeOf("let older = 1;");
    // TypeScript's grammar loads after JavaScript's, so the older block's tokens could come last.
    highlight("let older = 1;", "ts", target);
    drawLines(target, "let newer = 2;");
    highlight("let newer = 2;", "js", target);
    await vi.waitFor(() => {
      expect(tokens(target).some((token) => token.includes("newer"))).toBe(true);
    }, LOADED);
    await new Promise((resolve) => setTimeout(resolve, 200));
    expect(target.textContent).toBe("let newer = 2;");
  });

  it("keeps a block's text when Shiki doesn't load", async () => {
    const highlight = highlightWith((): Promise<Shikis> => Promise.reject(new Error("offline")));
    const target = codeOf("const a = 1;");
    highlight("const a = 1;", "js", target);
    await new Promise((resolve) => setTimeout(resolve, 50));
    expect(target.innerHTML).toBe(`<span class="${CODE_LINE}">const a = 1;</span>`);
  });

  it("keeps a block's text when its lines aren't an element each, as another renderer's are", TEST, async () => {
    const highlight = shikiHighlight();
    const target = Object.assign(document.createElement("code"), { textContent: "const a = 1;" });
    const other = codeOf("const b = 2;");
    highlight("const a = 1;", "js", target);
    highlight("const b = 2;", "js", other);
    await vi.waitFor(() => {
      expect(tokens(other)).toContain("const");
    }, LOADED);
    expect(target.innerHTML).toBe("const a = 1;");
  });
});
