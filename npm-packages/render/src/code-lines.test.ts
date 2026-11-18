import { afterEach, describe, expect, it } from "vitest";
import { createPage, type Page } from "./page.ts";
import type { ExpandedNode, Origin } from "./tree.ts";

/** An origin from line `from` to line `to`, each at column 1, with no offsets. */
const lines = (from: number, to: number): Origin => ({ start: { line: from, column: 1 }, end: { line: to, column: 1 }, occurrence: 0 });

const code = (value: string, origin?: Origin): ExpandedNode => ({ type: "code", lang: "ts", meta: null, value, ...(origin ? { origin } : {}) });
const paragraph = (value: string, origin: Origin): ExpandedNode => ({ type: "paragraph", children: [{ type: "text", value, origin }], origin });

let root: HTMLElement;
function page(): Page {
  root = document.body.appendChild(document.createElement("main"));
  return createPage(root);
}
afterEach(() => {
  root.remove();
});

const lineElements = () => [...root.querySelectorAll("pre > code > .compmd-line")];

describe("a code block's lines", () => {
  it("are an element each, a blank line too, each with its line break, so the block's text is the code's", () => {
    page().render({ type: "root", children: [code("let a;\n\nlet b;")] });
    expect(root.innerHTML).toBe('<pre><code class="language-ts"><span class="compmd-line">let a;\n</span><span class="compmd-line">\n</span><span class="compmd-line">let b;</span></code></pre>');
    expect(root.querySelector("code")?.textContent).toBe("let a;\n\nlet b;");
  });

  it("are numbered with their lines in the text: a fenced block's from the line after its fence", () => {
    // "Intro.\n\n```ts\nconst a = 1;\nconst b = 2;\n```": the block from line 3 to line 6.
    page().render({ type: "root", children: [paragraph("Intro.", lines(1, 1)), code("const a = 1;\nconst b = 2;", lines(3, 6))] });
    expect(lineElements().map((line) => line.getAttribute("data-line"))).toEqual(["4", "5"]);
  });

  it("are numbered from the block's first line when it has no fence, as an indented block", () => {
    page().render({ type: "root", children: [code("a\nb", lines(2, 3))] });
    expect(lineElements().map((line) => line.getAttribute("data-line"))).toEqual(["2", "3"]);
  });

  it("are numbered again when a change before the block moves it, the same elements", () => {
    const p = page();
    p.render({ type: "root", children: [paragraph("Intro.", lines(1, 1)), code("const a = 1;\nconst b = 2;", lines(3, 6))] });
    const [first] = lineElements();
    // A line typed above the paragraph: the block, kept, is one line on.
    p.splice([{ from: 0, to: 0, nodes: [paragraph("New line.", lines(1, 1))], lines: 1, offset: 10 }]);
    expect(lineElements().map((line) => line.getAttribute("data-line"))).toEqual(["5", "6"]);
    expect(lineElements()[0]).toBe(first);
  });

  it("have no numbers where the tree has no origins", () => {
    page().render({ type: "root", children: [code("a\nb")] });
    expect(lineElements().map((line) => line.hasAttribute("data-line"))).toEqual([false, false]);
  });
});

describe("lineHeights", () => {
  const minHeights = () => lineElements().map((line) => (line as HTMLElement).style.minHeight);
  const document = () => ({ type: "root" as const, children: [paragraph("Intro.", lines(1, 1)), code("const a = 1;\nconst b = 2;", lines(3, 6))] });

  it("makes the lines it names, by their line in the text, at least that tall, and only those", () => {
    const p = page();
    p.render(document());
    p.lineHeights(new Map([[5, 45]]));
    expect(minHeights()).toEqual(["", "45px"]);
    p.lineHeights(new Map([[4, 30]]));
    expect(minHeights()).toEqual(["30px", ""]);
    p.lineHeights(new Map());
    expect(minHeights()).toEqual(["", ""]);
  });

  it("keeps them as later renders draw the lines again, and gives them by line as a change moves the block", () => {
    const p = page();
    p.render(document());
    p.lineHeights(new Map([[5, 45]]));
    p.render({ type: "root", children: [paragraph("Intro.", lines(1, 1)), code("const a = 1;\nconst c = 3;", lines(3, 6))] });
    expect(minHeights()).toEqual(["", "45px"]);
    // A line typed above: line 5 is now the block's first line.
    p.splice([{ from: 0, to: 0, nodes: [paragraph("New line.", lines(1, 1))], lines: 1, offset: 10 }]);
    expect(minHeights()).toEqual(["45px", ""]);
  });

  it("leaves a line that isn't a code block's as it is", () => {
    const p = page();
    p.render(document());
    p.lineHeights(new Map([[1, 45]]));
    expect(root.querySelector("p")?.getAttribute("style")).toBeNull();
    expect(minHeights()).toEqual(["", ""]);
  });
});
