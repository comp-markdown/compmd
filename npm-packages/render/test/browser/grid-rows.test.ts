// A grid's rows of one height (spec §9): with `grid-auto-rows=1fr`, cards over two rows are all as tall as the tallest;
// without it, each row is as tall as its own tallest card.
import { afterEach, describe, expect, it } from "vitest";
import { createPage } from "../../src/page.ts";
import type { ExpandedNode } from "../../src/tree.ts";

let root: HTMLElement;
afterEach(() => {
  root.remove();
});

const param = (key: string, value: unknown) => ({ key, breakpoint: null, value });

/** A card: a box with a border and a paragraph of `words`. */
const card = (words: string): ExpandedNode => ({
  type: "componentContainer",
  name: "box",
  params: [param("border", "subtle"), param("padding", "s")],
  children: [{ type: "paragraph", children: [{ type: "text", value: words }] }],
});

/** The heights of four cards, two to a row, in a grid with `rows` as `grid-auto-rows`, if any. */
function cardHeights(rows?: string): number[] {
  root = document.body.appendChild(document.createElement("main"));
  root.style.cssText = "width: 468px; font: 16px/1.5 sans-serif";
  const grid: ExpandedNode = {
    type: "componentContainer",
    name: "box",
    params: [param("display", "grid"), param("grid-template-columns", "repeat(auto-fill, minmax(14rem, 1fr))"), ...(rows ? [param("grid-auto-rows", rows)] : [])],
    children: [card("The harbour at dusk, with the old quay, its tides and the boats that come in an hour later."), card("The quay."), card("Tides."), card("Boats.")],
  };
  createPage(root, {}).render({ type: "root", children: [grid] });
  return [...root.querySelectorAll("main > * > *")].map((element) => Math.round(element.getBoundingClientRect().height));
}

describe("grid-auto-rows", () => {
  it("makes every row as tall as the tallest with 1fr", () => {
    const heights = cardHeights("1fr");
    expect(heights).toHaveLength(4);
    expect(new Set(heights).size).toBe(1);
  });

  it("leaves each row its own height without it", () => {
    const [first, second, third, fourth] = cardHeights();
    expect(first).toBe(second);
    expect(third).toBe(fourth);
    expect(third).toBeLessThan(first ?? 0);
  });
});
