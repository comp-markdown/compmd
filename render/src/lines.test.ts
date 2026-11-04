import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Mount } from "./contract.ts";
import type { Line } from "./lines.ts";
import { createPage, type Page, type PageOptions } from "./page.ts";
import { createRegion } from "./region.ts";
import type { ExpandedComponent, ExpandedNode, Origin } from "./tree.ts";

/** An origin from line `line` at `start` to line `endLine` at `end` (offsets). */
const span = (line: number, start: number, endLine: number, end: number): Origin => ({
  start: { line, column: 1, offset: start },
  end: { line: endLine, column: 1, offset: end },
  occurrence: 0,
});
const text = (value: string, origin: Origin): ExpandedNode => ({ type: "text", value, origin });
const node = (type: string, origin: Origin, children: ExpandedNode[], extra: Record<string, unknown> = {}): ExpandedNode => ({ type, children, origin, ...extra });
const tree = (...children: ExpandedNode[]) => ({ type: "root" as const, children });

/** What the page measured, in order: a range's text, or an element's tag. */
let measured: string[];
/** Texts whose ranges show nothing, as hidden text doesn't. */
let hidden: Set<string>;
let root: HTMLElement;

/** A box for the `n`th measure: its `y` says in which order it was measured. */
const box = (n: number, width: number) => ({ x: 0, y: n * 20, width, height: 18 });

beforeEach(() => {
  measured = [];
  hidden = new Set();
  vi.spyOn(Range.prototype, "getClientRects").mockImplementation(function (this: Range) {
    const shown = this.toString();
    measured.push(shown);
    return (hidden.has(shown) ? [] : [box(measured.length, shown.length)]) as unknown as DOMRectList;
  });
  vi.spyOn(Element.prototype, "getBoundingClientRect").mockImplementation(function (this: Element) {
    if (this === root) return { x: 0, y: 0, width: 800, height: 600 } as DOMRect;
    measured.push(`<${this.tagName.toLowerCase()}>`);
    return box(measured.length, 100) as DOMRect;
  });
});

afterEach(() => {
  vi.restoreAllMocks();
  root.remove();
});

function page(options: PageOptions = {}): Page {
  root = document.body.appendChild(document.createElement("main"));
  return createPage(root, options);
}

// "# A\n\nSome *em*\ntext.": the heading on line 1, the paragraph on lines 3 and 4, its soft line break in the text after `*em*`.
const heading = () => node("heading", span(1, 0, 1, 3), [text("A", span(1, 2, 1, 3))], { depth: 1 });
const paragraph = () =>
  node("paragraph", span(3, 5, 4, 20), [text("Some ", span(3, 5, 3, 10)), node("emphasis", span(3, 10, 3, 14), [text("em", span(3, 11, 3, 13))]), text("\ntext.", span(3, 14, 4, 20))]);

describe("lines", () => {
  it("places each line by its text: a paragraph's later lines too, and leaves out lines that draw nothing", () => {
    const p = page();
    p.render(tree(heading(), paragraph()));
    expect([...p.lines()].map(({ line }) => line)).toEqual([1, 3, 4]);
    expect(measured).toEqual(["A", "Some em", "text."]);
  });

  it("gives each line's boxes from the root's top left corner, its scrolled content's", () => {
    const p = page();
    p.render(tree(heading()));
    root.scrollTop = 30;
    vi.spyOn(Element.prototype, "getBoundingClientRect").mockImplementation(function (this: Element) {
      return (this === root ? { x: 100, y: 50, width: 800, height: 600 } : box(1, 1)) as DOMRect;
    });
    const [first] = p.lines();
    expect(first?.rects).toEqual([{ x: -100, y: 20 - 50 + root.scrollTop, width: 1, height: 18 }]);
  });

  it("places a code block's lines in its block, an empty one by its line break, and not its fences", () => {
    const p = page();
    // "```js\na\n\nb\n```" on lines 6 to 10.
    p.render(tree(node("code", span(6, 30, 10, 45), [], { lang: "js", value: "a\n\nb" })));
    expect([...p.lines()].map(({ line }) => line)).toEqual([7, 8, 9]);
    expect(measured).toEqual(["a", "\n", "b"]);
  });

  it("finds a code block's lines across the highlighter's tokens", () => {
    const p = page({
      highlight: (code, _language, target) => {
        target.replaceChildren(...code.split(/(?<=\s)/).map((token) => Object.assign(document.createElement("span"), { textContent: token })));
      },
    });
    p.render(tree(node("code", span(1, 0, 4, 30), [], { lang: "js", value: "let a = 1;\nlet b;" })));
    expect([...p.lines()].map(({ line }) => line)).toEqual([2, 3]);
    expect(measured).toEqual(["let a = 1;", "let b;"]);
  });

  it("places a line with no text of its own by the box of what its range drew, and nothing inside it after", () => {
    const chart: Mount = (target) => {
      target.appendChild(document.createElement("svg"));
    };
    const p = page({ components: { chart } });
    const use: ExpandedComponent = { type: "componentLeaf", name: "chart", params: [], children: [], slots: [], origin: span(2, 4, 4, 30) };
    p.render(tree(use));
    expect([...p.lines()].map(({ line }) => line)).toEqual([2]);
    expect(measured).toEqual(["<div>"]);
  });

  it("places a line of an each's iterations by the first that shows", () => {
    const p = page();
    const iteration = (value: string) => node("paragraph", span(2, 4, 2, 12), [text(value, span(2, 4, 2, 12))]);
    hidden.add("Apples");
    p.render(tree(iteration("Apples"), iteration("Pears")));
    expect([...p.lines()].map(({ line }) => line)).toEqual([2]);
    expect(measured).toEqual(["Apples", "Pears"]);
  });

  it("places only the first line of text whose line breaks aren't the text's, as an interpolation's value", () => {
    const p = page();
    p.render(tree(node("paragraph", span(1, 0, 1, 10), [text("one\ntwo", span(1, 0, 1, 10))])));
    expect([...p.lines()].map(({ line }) => line)).toEqual([1]);
    expect(measured).toEqual(["one"]);
  });

  it("starts at a line, and measures no further than its caller goes", () => {
    const p = page();
    p.render(tree(heading(), paragraph()));
    const first: IteratorResult<Line> = p.lines(3).next();
    expect(first.done ? undefined : first.value.line).toBe(3);
    expect(measured).toEqual(["Some em"]);
  });

  it("gives a region's lines in its target", () => {
    root = document.body.appendChild(document.createElement("div"));
    const region = createRegion(root);
    region.render([paragraph()]);
    expect([...region.lines()].map(({ line }) => line)).toEqual([3, 4]);
  });
});

describe("onLayout", () => {
  it("is called once after a render is painted, however many renders came in that frame", async () => {
    const onLayout = vi.fn();
    const p = page({ onLayout });
    p.render(tree(heading()));
    p.render(tree(heading(), paragraph()));
    await vi.waitFor(() => {
      expect(onLayout).toHaveBeenCalledTimes(1);
    });
    p.destroy();
  });
});
