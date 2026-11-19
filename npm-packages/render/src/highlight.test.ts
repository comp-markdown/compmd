// A code block's highlighting: a highlighter gives tokens, and the page draws them into the block's own lines.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CodeHighlighter, CodeLines } from "./highlight.ts";
import { createPage } from "./page.ts";
import type { ExpandedNode, ExpandedRoot } from "./tree.ts";

let area: HTMLElement;
beforeEach(() => {
  area = document.body.appendChild(document.createElement("main"));
});
afterEach(() => {
  area.remove();
});

const origin = { start: { line: 4, column: 1 }, end: { line: 8, column: 4 }, occurrence: 0 };
/** A fenced block of `value`, its fence on line 4. */
const block = (value: string, lang = "js"): ExpandedNode => ({ type: "code", lang, meta: null, value, origin });
const tree = (...children: ExpandedNode[]): ExpandedRoot => ({ type: "root", children });

/** Each word of each line a token, the first of a line bold. */
const words: CodeHighlighter = (code) => code.split("\n").map((line) => (line === "" ? [] : line.split(/(?<= )/).map((text, i) => ({ text, ...(i === 0 ? { bold: true } : {}) }))));

/** The block's lines: each one's number, its tokens' texts, and its text. */
const lines = () => [...area.querySelectorAll(".compmd-line")].map((line) => [line.getAttribute("data-line"), [...line.querySelectorAll(":scope > span")].map((span) => span.textContent), line.textContent]);
/** The block's tokens' texts. */
const tokens = () => [...area.querySelectorAll(".compmd-line > span")].map((span) => span.textContent);
/** Until promises a highlighter gave have settled. */
const settled = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("a highlighter's tokens", () => {
  it("are drawn in the block's lines, which keep their numbers and their line breaks, a blank line too", () => {
    const text = "const a = 1;\n\nconst b = 2;";
    createPage(area, { highlight: words }).render(tree(block(text)));
    expect(lines()).toEqual([
      ["5", ["const ", "a ", "= ", "1;"], "const a = 1;\n"],
      ["6", [], "\n"],
      ["7", ["const ", "b ", "= ", "2;"], "const b = 2;"],
    ]);
    expect(area.textContent).toBe(text);
  });

  it("show in their colour in each scheme, and their font style", () => {
    const highlight: CodeHighlighter = () => [[{ text: "a", color: { light: "#111111", dark: "#eeeeee" }, italic: true, underline: true }, { text: "b" }]];
    // happy-dom doesn't know `light-dark()` and drops it, so what the page sets is read where it sets it.
    const set = vi.spyOn(CSSStyleDeclaration.prototype, "setProperty");
    createPage(area, { highlight }).render(tree(block("ab")));
    expect(set.mock.calls).toEqual([
      ["color", "light-dark(#111111, #eeeeee)"],
      ["font-style", "italic"],
      ["text-decoration", "underline"],
    ]);
    set.mockRestore();
    const [a, b] = [...area.querySelectorAll<HTMLElement>(".compmd-line > span")];
    expect([a?.style.getPropertyValue("font-style"), a?.style.getPropertyValue("text-decoration")]).toEqual(["italic", "underline"]);
    expect(b?.getAttribute("style")).toBeNull();
  });

  it("are drawn when they come, the block showing its text until then", async () => {
    createPage(area, { highlight: (code, language) => Promise.resolve(words(code, language)) }).render(tree(block("let a;")));
    expect([tokens(), area.textContent]).toEqual([[], "let a;"]);
    await settled();
    expect(tokens()).toEqual(["let ", "a;"]);
  });

  it("are dropped when they come after a newer text's, whichever comes first", async () => {
    const given: ((lines: CodeLines) => void)[] = [];
    const page = createPage(area, { highlight: () => new Promise((resolve) => given.push(resolve)) });
    page.render(tree(block("let older;")));
    page.render(tree(block("let newer;")));
    given[1]?.([[{ text: "let newer;" }]]);
    given[0]?.([[{ text: "let older;" }]]);
    await settled();
    expect([tokens(), area.textContent]).toEqual([["let newer;"], "let newer;"]);
  });
});

describe("a block keeps its text", () => {
  const kept = '<pre><code class="language-js"><span class="compmd-line" data-line="5">let a;</span></code></pre>';
  /** What a page of the block `let a;` shows with `highlight`, once what it gave has settled. */
  async function shown(highlight: CodeHighlighter): Promise<string> {
    const root = area.appendChild(document.createElement("div"));
    createPage(root, { highlight }).render(tree(block("let a;")));
    await settled();
    return root.innerHTML;
  }

  it("when the highlighter gives none for its language", async () => {
    expect(await shown(() => undefined)).toBe(kept);
    expect(await shown(() => Promise.resolve(undefined))).toBe(kept);
  });

  it("when the tokens aren't its text: other text, or other lines", async () => {
    for (const given of [[[{ text: "let b;" }]], [[{ text: "let " }], [{ text: "a;" }]], []]) expect(await shown(() => given)).toBe(kept);
  });

  it("when the highlighter fails, at once or later", async () => {
    expect(
      await shown(() => {
        throw new Error("no grammar");
      }),
    ).toBe(kept);
    expect(await shown(() => Promise.reject(new Error("offline")))).toBe(kept);
  });
});
