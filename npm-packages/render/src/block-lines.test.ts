import { afterEach, describe, expect, it } from "vitest";
import type { Mount, SlotItem } from "./contract.ts";
import { createPage, type Page, type PageOptions } from "./page.ts";
import type { ExpandedComponent, ExpandedNode, Origin } from "./tree.ts";

/** An origin from line `from` to line `to`, each at column 1, with no offsets. */
const lines = (from: number, to = from): Origin => ({ start: { line: from, column: 1 }, end: { line: to, column: 1 }, occurrence: 0 });
const text = (value: string): ExpandedNode => ({ type: "text", value });
const paragraph = (value: string, origin: Origin): ExpandedNode => ({ type: "paragraph", children: [text(value)], origin });

let root: HTMLElement;
function page(options: PageOptions = {}): Page {
  root = document.body.appendChild(document.createElement("main"));
  return createPage(root, options);
}
afterEach(() => {
  root.remove();
});

/** The `data-line` of each of the page's top-level elements. */
const marks = () => [...root.children].map((element) => element.getAttribute("data-line"));

/** A component that places its default slot's items in its box. */
const card: Mount = (target, { slots }) => {
  for (const item of (slots.default ?? []) as SlotItem[]) target.append(item.element);
};

describe("each block of the page", () => {
  it("carries the line it starts on", () => {
    // "# Notes\n\nIntro.\n\n- a\n- b\n"
    page().render({
      type: "root",
      children: [
        { type: "heading", depth: 1, children: [text("Notes")], origin: lines(1) },
        paragraph("Intro.", lines(3)),
        { type: "list", ordered: false, children: [{ type: "listItem", children: [paragraph("a", lines(5))], origin: lines(5) }], origin: lines(5, 6) },
      ],
    });
    expect(marks()).toEqual(["1", "3", "5"]);
  });

  it("a container carries its line, and what its slots hold doesn't", () => {
    const container: ExpandedComponent = { type: "componentContainer", name: "card", params: [], children: [paragraph("Inside.", lines(4))], slots: [], origin: lines(3, 5) };
    page({ components: { card } }).render({ type: "root", children: [paragraph("Intro.", lines(1)), container] });
    expect(marks()).toEqual(["1", "3"]);
    expect(root.querySelector(".compmd-component p")?.hasAttribute("data-line")).toBe(false);
  });

  it("a definition's use carries the use's line, not the definition's", () => {
    // A `compmd` component's use is a box holding its body, whose nodes come from the definition, on lines 1 to 3.
    const box: ExpandedComponent = { type: "componentContainer", name: "box", params: [], children: [paragraph("The body.", lines(2))], slots: [], origin: lines(8) };
    page().render({ type: "root", children: [box] });
    expect(marks()).toEqual(["8"]);
  });

  it("an example carries the line of its fence", () => {
    const example: ExpandedNode = { type: "code", lang: "compmd", meta: null, value: "Rated.", render: "output", document: { type: "root", children: [paragraph("Rated.", lines(1))], example: 1 }, origin: lines(4, 6) };
    page().render({ type: "root", children: [paragraph("Intro.", lines(1)), example] });
    expect(marks()).toEqual(["1", "4"]);
  });

  it("is marked again when a change before it moves it, the same element", () => {
    const p = page();
    p.render({ type: "root", children: [paragraph("Intro.", lines(1)), paragraph("Then.", lines(3))] });
    const then = root.children[1];
    // A line typed above the first paragraph: the second, kept, is one line on.
    p.splice([{ from: 0, to: 0, nodes: [paragraph("New line.", lines(1))], lines: 1, offset: 10 }]);
    expect(marks()).toEqual(["1", "2", "4"]);
    expect(root.children[2]).toBe(then);
  });

  it("carries none where the tree has no origins", () => {
    page().render({ type: "root", children: [{ type: "paragraph", children: [text("No origin.")] }] });
    expect(marks()).toEqual([null]);
  });
});
