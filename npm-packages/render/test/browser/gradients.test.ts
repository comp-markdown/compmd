// A box's gradient in a browser (spec §9.4): its logical sides and its angle follow the text's direction, so in
// right-to-left text it is drawn as the mirrored physical gradient, and in left-to-right text as written.
import { afterEach, describe, expect, it } from "vitest";
import { createPage } from "../../src/page.ts";
import type { ExpandedRoot } from "../../src/tree.ts";

let root: HTMLElement;
afterEach(() => {
  root.remove();
});

/** A document of one box with `background`, in `dir`. */
const boxWith = (background: string, dir: "ltr" | "rtl"): ExpandedRoot => ({
  type: "root",
  dir,
  children: [{ type: "componentContainer", name: "box", params: [{ key: "background", breakpoint: null, value: background }], children: [{ type: "paragraph", children: [{ type: "text", value: "Text." }] }] }],
});

/** The background the browser draws for the box of `tree`. */
function drawn(tree: ExpandedRoot): string {
  root = document.body.appendChild(document.createElement("main"));
  createPage(root, {}).render(tree);
  const box = root.querySelector(".compmd-box")!;
  return getComputedStyle(box).backgroundImage;
}

/** The background the browser draws for a plain element styled with `css`. */
function physical(css: string): string {
  const element = document.body.appendChild(document.createElement("div"));
  element.style.background = css;
  const image = getComputedStyle(element).backgroundImage;
  element.remove();
  return image;
}

describe("a gradient in a box", () => {
  it("cuts the block-end inline-end corner at 45°, at the bottom right in left-to-right text", () => {
    expect(drawn(boxWith("linear-gradient(315deg, transparent 1rem, red 1rem)", "ltr"))).toBe(physical("linear-gradient(315deg, transparent 1rem, red 1rem)"));
  });

  it("cuts it at the bottom left in right-to-left text, the angle mirrored", () => {
    expect(drawn(boxWith("linear-gradient(315deg, transparent 1rem, red 1rem)", "rtl"))).toBe(physical("linear-gradient(45deg, transparent 1rem, red 1rem)"));
  });

  it("turns logical sides into physical ones for the text's direction", () => {
    expect(drawn(boxWith("linear-gradient(to block-end inline-end, red, blue)", "ltr"))).toBe(physical("linear-gradient(to bottom right, red, blue)"));
    root.remove();
    expect(drawn(boxWith("linear-gradient(to block-end inline-end, red, blue)", "rtl"))).toBe(physical("linear-gradient(to bottom left, red, blue)"));
    root.remove();
    expect(drawn(boxWith("radial-gradient(circle at inline-start block-start, red, blue)", "rtl"))).toBe(physical("radial-gradient(circle at right top, red, blue)"));
  });

  it("keeps physical sides in either direction", () => {
    expect(drawn(boxWith("linear-gradient(to bottom left, red, blue)", "rtl"))).toBe(physical("linear-gradient(to bottom left, red, blue)"));
  });
});
