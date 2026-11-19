import type { CodeLines } from "@compmd/render";
import { describe, expect, it } from "vitest";
import { highlightWith, shikiHighlight, type Shikis } from "./highlight.ts";
import { LANGUAGE_NAMES } from "./languages.ts";

/** The texts of `lines`' tokens. */
const texts = (lines: CodeLines | undefined) => (lines ?? []).flatMap((line) => line.map(({ text }) => text));
/** `lines` as the text they are of. */
const textOf = (lines: CodeLines | undefined) => (lines ?? []).map((line) => line.map(({ text }) => text).join("")).join("\n");

/** Shiki and a grammar load well under a second alone, but take many seconds while every package's tests run at once. */
const TEST = { timeout: 15_000 };

describe("shikiHighlight", () => {
  it("gives a block's tokens, which are its text, in both themes' colours, loading Shiki once", TEST, async () => {
    const highlight = shikiHighlight();
    const [js, python] = await Promise.all([highlight("const a = 1;", "js"), highlight("print('hi')", "python")]);
    expect(texts(js)).toContain("const");
    expect(texts(python)).toContain("print");
    expect([textOf(js), textOf(python)]).toEqual(["const a = 1;", "print('hi')"]);
    const keyword = js?.[0]?.find(({ text }) => text === "const");
    expect(keyword?.color).toEqual({ light: expect.stringMatching(/^#/) as string, dark: expect.stringMatching(/^#/) as string });
    expect(keyword?.color?.light).not.toBe(keyword?.color?.dark);
  });

  it("gives a line of tokens for each line of the text, a blank line too", TEST, async () => {
    const lines = await shikiHighlight()("const a = 1;\n\nconst b = 2;", "js");
    expect(lines?.map((line) => line.map(({ text }) => text).join(""))).toEqual(["const a = 1;", "", "const b = 2;"]);
  });

  it("shows CompMD's syntax in Markdown, its frontmatter as YAML, and a definition's signature as YAML", TEST, async () => {
    const highlight = shikiHighlight();
    const markdown = '---\nheading: A\n---\n::: card {title="Setup" gap=m}\n- {{md.doc.heading}}\n::: /card';
    const definition = "params:\n  a: string\n---\n::: box {gap=s}\n{{a}}\n::: /box";
    const [md, compmd] = await Promise.all([highlight(markdown, "md"), highlight(definition, "compmd")]);
    expect(texts(md)).toEqual(expect.arrayContaining(["heading", "card", "gap", "{{md.doc.heading}}"]));
    expect(texts(compmd)).toEqual(expect.arrayContaining(["params", "box"]));
    expect([textOf(md), textOf(compmd)]).toEqual([markdown, definition]);
  });

  it("gives none for a language it doesn't highlight, or for none", () => {
    const highlight = shikiHighlight();
    expect([highlight("DISPLAY 'HI'.", "cobol"), highlight("plain", "")]).toEqual([undefined, undefined]);
  });

  it("loads and gives every language it lists, with the JavaScript engine", { timeout: 30_000 }, async () => {
    const highlight = shikiHighlight();
    const given = await Promise.all(LANGUAGE_NAMES.map(async (name) => [name, await highlight("x = 1\ny", name)] as const));
    expect(given.filter(([, lines]) => textOf(lines) !== "x = 1\ny").map(([name]) => name)).toEqual([]);
  });
});

describe("highlightWith", () => {
  it("rejects when Shiki doesn't load, so the renderer keeps the block's text", async () => {
    const highlight = highlightWith((): Promise<Shikis> => Promise.reject(new Error("offline")));
    await expect(highlight("const a = 1;", "js")).rejects.toThrow("offline");
  });
});
