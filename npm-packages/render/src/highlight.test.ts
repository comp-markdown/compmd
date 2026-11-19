// A code block's highlighting: a highlighter gives tokens, and the page draws them into the block's own lines, a
// block at a time (`highlight-queue.ts`); the page has drawn once they are.
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CodeHighlighter, CodeLines } from "./highlight.ts";
import { createPage, type Page } from "./page.ts";
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
const lines = (root: ParentNode = area) => [...root.querySelectorAll(".compmd-line")].map((line) => [line.getAttribute("data-line"), [...line.querySelectorAll(":scope > span")].map((span) => span.textContent), line.textContent]);
/** The blocks' tokens' texts. */
const tokens = (root: ParentNode = area) => [...root.querySelectorAll(".compmd-line > span")].map((span) => span.textContent);
/** A task later. */
const nextTask = () => new Promise((resolve) => setTimeout(resolve, 0));

/** A page of `children` in `area`, highlighted with `highlight`, once it has drawn. */
async function drawn(highlight: CodeHighlighter, ...children: ExpandedNode[]): Promise<Page> {
  const page = createPage(area, { highlight });
  page.render(tree(...children));
  await page.drawn();
  return page;
}

describe("a highlighter's tokens", () => {
  it("are drawn in the block's lines, which keep their numbers and their line breaks, a blank line too", async () => {
    const text = "const a = 1;\n\nconst b = 2;";
    await drawn(words, block(text));
    expect(lines()).toEqual([
      ["5", ["const ", "a ", "= ", "1;"], "const a = 1;\n"],
      ["6", [], "\n"],
      ["7", ["const ", "b ", "= ", "2;"], "const b = 2;"],
    ]);
    expect(area.textContent).toBe(text);
  });

  it("show in their colour in each scheme, and their font style", async () => {
    const highlight: CodeHighlighter = () => [[{ text: "a", color: { light: "#111111", dark: "#eeeeee" }, italic: true, underline: true }, { text: "b" }]];
    // happy-dom doesn't know `light-dark()` and drops it, so what the page sets is read where it sets it.
    const set = vi.spyOn(CSSStyleDeclaration.prototype, "setProperty");
    await drawn(highlight, block("ab"));
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

  it("are drawn when they come, the block showing its text until then, and the page has drawn once they are", async () => {
    let give: (lines: CodeLines) => void = () => undefined;
    const page = createPage(area, { highlight: () => new Promise((resolve) => (give = resolve)) });
    page.render(tree(block("let a;")));
    let done = false;
    void page.drawn().then(() => (done = true));
    await nextTask();
    expect([tokens(), area.textContent, done]).toEqual([[], "let a;", false]);
    give([[{ text: "let " }, { text: "a;" }]]);
    await page.drawn();
    expect(tokens()).toEqual(["let ", "a;"]);
  });

  it("are dropped when they come after the block's text changed", async () => {
    const given: ((lines: CodeLines) => void)[] = [];
    const page = createPage(area, { highlight: () => new Promise((resolve) => given.push(resolve)) });
    page.render(tree(block("let older;")));
    await nextTask();
    page.render(tree(block("let newer;")));
    given[0]?.([[{ text: "let older;" }]]);
    await nextTask();
    await nextTask();
    given[1]?.([[{ text: "let newer;" }]]);
    await page.drawn();
    expect([tokens(), area.textContent]).toEqual([["let newer;"], "let newer;"]);
  });
});

describe("a page of many blocks", () => {
  it("asks for one block's tokens at a time, the next once they are drawn and a task later", async () => {
    const asked: string[] = [];
    let give: (lines: CodeLines) => void = () => undefined;
    const page = createPage(area, {
      highlight: (code) => {
        asked.push(code);
        return new Promise((resolve) => (give = resolve));
      },
    });
    page.render(tree(block("a"), block("b"), block("c")));
    await nextTask();
    expect(asked).toEqual(["a"]);
    give([[{ text: "a" }]]);
    await nextTask();
    await nextTask();
    expect([asked, tokens()]).toEqual([["a", "b"], ["a"]]);
    give([[{ text: "b" }]]);
    await nextTask();
    await nextTask();
    give([[{ text: "c" }]]);
    await page.drawn();
    expect(tokens()).toEqual(["a", "b", "c"]);
  });

  it("asks for the blocks on screen first", async () => {
    const observers: { callback: IntersectionObserverCallback; observed: Element[] }[] = [];
    vi.stubGlobal(
      "IntersectionObserver",
      class {
        readonly #entry: { callback: IntersectionObserverCallback; observed: Element[] };
        constructor(callback: IntersectionObserverCallback) {
          this.#entry = { callback, observed: [] };
          observers.push(this.#entry);
        }
        observe(target: Element) {
          this.#entry.observed.push(target);
        }
        unobserve() {
          return undefined;
        }
        disconnect() {
          return undefined;
        }
      },
    );
    // A document of its own, so its queue's observer is the one stubbed here.
    const other = document.implementation.createHTMLDocument();
    Object.defineProperty(other, "defaultView", { value: window });
    const root = other.body.appendChild(other.createElement("main"));
    const asked: string[] = [];
    const page = createPage(root, {
      highlight: (code) => {
        asked.push(code);
        return [[{ text: code }]];
      },
    });
    page.render(tree(block("a"), block("b"), block("c")));
    const [observer] = observers;
    const c = observer?.observed.find((target) => target.textContent === "c");
    observer?.callback([{ target: c, isIntersecting: true } as IntersectionObserverEntry], {} as IntersectionObserver);
    await page.drawn();
    expect(asked).toEqual(["c", "a", "b"]);
    expect(tokens(root)).toEqual(["a", "b", "c"]);
    vi.unstubAllGlobals();
  });

  it("never asks for a block that is gone, and the page has drawn", async () => {
    const asked: string[] = [];
    const page = createPage(area, {
      highlight: (code) => {
        asked.push(code);
        return [[{ text: code }]];
      },
    });
    page.render(tree(block("a"), block("b")));
    page.render(tree(block("a")));
    await page.drawn();
    expect(asked).toEqual(["a"]);
  });
});

describe("a block keeps its text", () => {
  const kept = '<pre><code class="language-js"><span class="compmd-line" data-line="5">let a;</span></code></pre>';
  /** What a page of the block `let a;` shows with `highlight`, once it has drawn. */
  async function shown(highlight: CodeHighlighter): Promise<string> {
    const root = area.appendChild(document.createElement("div"));
    const page = createPage(root, { highlight });
    page.render(tree(block("let a;")));
    await page.drawn();
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
