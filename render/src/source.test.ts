import { afterEach, describe, expect, it, vi } from "vitest";
import type { Mount, SlotItem } from "./contract.ts";
import { createPage, type Page, type PageOptions } from "./page.ts";
import { createRegion } from "./region.ts";
import { reveal, REVEALED_MS } from "./source.ts";
import type { ExpandedComponent, ExpandedNode, Origin } from "./tree.ts";

/** An origin on line `line`, from `start` to `end` (offsets), the `occurrence`th of its range in its list. */
const at = (start: number, end: number, line = 1, occurrence = 0): Origin => ({
  start: { line, column: 1, offset: start },
  end: { line, column: 1 + end - start, offset: end },
  occurrence,
});
const text = (value: string, origin?: Origin): ExpandedNode => ({ type: "text", value, ...(origin ? { origin } : {}) });
const node = (type: string, origin: Origin, children: ExpandedNode[], extra: Record<string, unknown> = {}): ExpandedNode => ({ type, children, origin, ...extra });

let root: HTMLElement;
function page(options: PageOptions = {}): Page {
  root = document.body.appendChild(document.createElement("main"));
  return createPage(root, options);
}
afterEach(() => {
  root.remove();
});

// "# A\n\nSome *em* text.": the heading at 0-3, the paragraph at 5-20 on line 3, `*em*` at 10-14.
const heading = () => node("heading", at(0, 3), [text("A", at(2, 3))], { depth: 1 });
const paragraph = (line = 3, shift = 0) =>
  node("paragraph", at(5 + shift, 20 + shift, line), [text("Some ", at(5 + shift, 10 + shift, line)), node("emphasis", at(10 + shift, 14 + shift, line), [text("em", at(11 + shift, 13 + shift, line))]), text(" text.", at(14 + shift, 20 + shift, line))]);
const tree = (...children: ExpandedNode[]) => ({ type: "root" as const, children });

describe("sourceOf", () => {
  it("gives the range of the node an element or a text was drawn for", () => {
    const p = page();
    p.render(tree(heading(), paragraph()));
    const em = root.querySelector("em")!;
    expect(p.sourceOf(em)).toEqual({ start: { line: 3, column: 1, offset: 10 }, end: { line: 3, column: 5, offset: 14 } });
    expect(p.sourceOf(em.firstChild!)?.start.offset).toBe(11);
    expect(p.sourceOf(root.querySelector("h1")!)?.end.offset).toBe(3);
  });

  it("gives what the renderer adds its parent's range, and nothing outside the blocks", () => {
    const p = page();
    p.render(tree(node("listItem", at(0, 9), [node("paragraph", at(2, 9), [text("Done", at(2, 9))])], { checked: true })));
    expect(p.sourceOf(root.querySelector("input")!)?.start.offset).toBe(0);
    expect(p.sourceOf(root)).toBeUndefined();
  });

  it("gives a component's own DOM the component's range, and a slot's items their own", () => {
    const card: Mount = (target, { slots }) => {
      target.appendChild(document.createElement("header")).textContent = "Card";
      for (const item of (slots.default ?? []) as SlotItem[]) target.append(item.element);
    };
    const p = page({ components: { card } });
    const component: ExpandedComponent = { type: "componentContainer", name: "card", params: [], children: [node("paragraph", at(10, 16, 2), [text("Inside", at(10, 16, 2))])], slots: [], origin: at(0, 26) };
    p.render(tree(component));
    expect(p.sourceOf(root.querySelector("header")!)?.end.offset).toBe(26);
    expect(p.sourceOf(root.querySelector("p")!)?.start.offset).toBe(10);
  });

  it("follows the kept blocks through a splice", () => {
    const p = page();
    p.render(tree(heading(), paragraph()));
    // "# A" became "# ABC": the paragraph is 2 code points on, on the same line.
    p.splice([{ from: 0, to: 1, nodes: [node("heading", at(0, 5), [text("ABC", at(2, 5))], { depth: 1 })], lines: 0, offset: 2 }]);
    expect(p.sourceOf(root.querySelector("em")!)?.start).toEqual({ line: 3, column: 1, offset: 12 });
    // A line typed above it: one line and 4 code points on.
    p.splice([{ from: 0, to: 0, nodes: [node("paragraph", at(0, 2), [text("Hi", at(0, 2))])], lines: 1, offset: 4 }]);
    expect(p.sourceOf(root.querySelector("em")!)?.start).toEqual({ line: 4, column: 1, offset: 16 });
  });

  it("gives each footnote's item in the list at the end its definition's range", () => {
    const p = page();
    p.render(
      tree(
        node("paragraph", at(0, 5), [{ type: "footnoteReference", identifier: "n", origin: at(1, 5) }]),
        node("footnoteDefinition", at(7, 17, 3), [node("paragraph", at(13, 17, 3), [text("Note", at(13, 17, 3))])], { identifier: "n" }),
      ),
    );
    expect(p.sourceOf(root.querySelector(".compmd-footnote-back")!)?.start.offset).toBe(7);
    expect(p.sourceOf(root.querySelector(".compmd-footnotes p")!)?.start.offset).toBe(13);
  });

  it("gives nothing for a tree without origins", () => {
    const p = page();
    p.render(tree({ type: "paragraph", children: [text("Plain")] }));
    expect(p.sourceOf(root.querySelector("p")!)).toBeUndefined();
    expect(p.elementsAt(0)).toEqual([]);
  });
});

describe("elementsAt", () => {
  it("gives the elements of the innermost range that holds the offset", () => {
    const p = page();
    p.render(tree(heading(), paragraph()));
    expect(p.elementsAt(12)).toEqual([root.querySelector("em")]);
    expect(p.elementsAt(6)).toEqual([root.querySelector("p")]);
    expect(p.elementsAt(4)).toEqual([]);
  });

  it("takes the range that starts at an offset over the one that ends there, and the one that ends there at the end", () => {
    const p = page();
    p.render(tree(heading(), paragraph()));
    expect(p.elementsAt(10)).toEqual([root.querySelector("em")]);
    expect(p.elementsAt(14)).toEqual([root.querySelector("p")]);
    expect(p.elementsAt(3)).toEqual([root.querySelector("h1")]);
    expect(p.elementsAt(20)).toEqual([root.querySelector("p")]);
  });

  it("gives every iteration of an each, and of a box and its body the box", () => {
    const p = page();
    const iteration = (occurrence: number, value: string) => node("listItem", at(2, 12, 1, occurrence), [node("paragraph", at(2, 12, 1, occurrence), [text(value, at(2, 12))])]);
    const box: ExpandedComponent = { type: "componentContainer", name: "box", params: [], children: [node("paragraph", at(14, 30, 3), [text("Tile T.", at(14, 30, 3))])], slots: [], origin: at(14, 30, 3) };
    p.render(tree(node("list", at(0, 12), [iteration(0, "Apples"), iteration(1, "Pears")], { ordered: false }), box));
    expect(p.elementsAt(5)).toEqual([...root.querySelectorAll("li")]);
    expect(p.elementsAt(20)).toEqual([root.querySelector(".compmd-box")]);
  });

  it("follows the kept blocks through a splice", () => {
    const p = page();
    p.render(tree(heading(), paragraph()));
    p.splice([{ from: 0, to: 1, nodes: [node("heading", at(0, 5), [text("ABC", at(2, 5))], { depth: 1 })], lines: 0, offset: 2 }]);
    expect(p.elementsAt(12)).toEqual([root.querySelector("em")]);
    expect(p.elementsAt(4)).toEqual([root.querySelector("h1")]);
  });
});

describe("reveal", () => {
  it("scrolls to the first element and marks each for a while, a second reveal starting the time again", () => {
    vi.useFakeTimers();
    root = document.body.appendChild(document.createElement("div"));
    const [first, second] = [root.appendChild(document.createElement("p")), root.appendChild(document.createElement("p"))];
    const scrolled = vi.spyOn(first, "scrollIntoView");
    reveal([first, second]);
    expect(scrolled).toHaveBeenCalledWith({ block: "nearest", inline: "nearest" });
    vi.advanceTimersByTime(REVEALED_MS - 100);
    reveal([first]);
    vi.advanceTimersByTime(200);
    expect([first.className, second.className]).toEqual(["compmd-revealed", ""]);
    vi.advanceTimersByTime(REVEALED_MS);
    expect(first.className).toBe("");
    vi.useRealTimers();
  });
});

describe("a region", () => {
  it("maps between its DOM and the origins of the nodes it rendered last", () => {
    root = document.body.appendChild(document.createElement("div"));
    const region = createRegion(root);
    region.render([paragraph()]);
    const em = root.querySelector("em")!;
    expect(region.sourceOf(em.firstChild!)?.start.offset).toBe(11);
    expect(region.elementsAt(12)).toEqual([em]);
    // Two code points on: `*em*` at 12-16.
    region.render([paragraph(3, 2)]);
    expect(region.elementsAt(11)).toEqual([root.querySelector("p")]);
  });
});
