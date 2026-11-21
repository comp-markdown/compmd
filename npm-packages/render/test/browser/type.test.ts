// What only a browser's layout says about type in a box (spec §9.2, §9.4): its sizes are in `em`, so a badge in a
// heading is small for the heading; it reaches the boxes inside; a theme changes a token; and a transform shows the
// text without changing it.
import { afterEach, describe, expect, it } from "vitest";
import { createPage } from "../../src/page.ts";
import type { ExpandedNode } from "../../src/tree.ts";

let root: HTMLElement;
afterEach(() => {
  root.remove();
});

const param = (key: string, value: unknown) => ({ key, breakpoint: null, value });
const text = (value: string): ExpandedNode => ({ type: "text", value });

function draw(...children: ExpandedNode[]): HTMLElement {
  root = document.body.appendChild(document.createElement("main"));
  root.style.cssText = "width: 600px; font: 16px/1.5 sans-serif";
  createPage(root, {}).render({ type: "root", children });
  return root;
}

const badge = (...params: { key: string; breakpoint: null; value: unknown }[]): ExpandedNode => ({ type: "componentInline", name: "box", params, fallback: [text("new")] });
const box = (params: { key: string; breakpoint: null; value: unknown }[], children: ExpandedNode[]): ExpandedNode => ({ type: "componentContainer", name: "box", params, children });
const style = (element: Element | null) => getComputedStyle(element as Element);

describe("type in a box", () => {
  it("sizes a badge in em, so it is small for a heading as for a line of text", () => {
    const page = draw({ type: "heading", depth: 1, children: [text("Saved searches "), badge(param("font-size", "s"))] }, { type: "paragraph", children: [text("Search "), badge(param("font-size", "s"))] });
    const [inHeading, inText] = [...page.querySelectorAll("span.compmd-box")];
    const heading = parseFloat(style(page.querySelector("h1")).fontSize);
    expect(parseFloat(style(inHeading ?? null).fontSize)).toBeCloseTo(heading * 0.875, 1);
    expect(parseFloat(style(inText ?? null).fontSize)).toBeCloseTo(16 * 0.875, 1);
  });

  it("reaches the boxes inside, until one sets its own", () => {
    const page = draw(box([param("font-family", "mono"), param("color", "#a40e26"), param("text-align", "center")], [box([], [{ type: "paragraph", children: [text("inner")] }]), box([param("text-align", "start")], [{ type: "paragraph", children: [text("own")] }])]));
    const [inner, own] = [...page.querySelectorAll("p")];
    expect(style(inner ?? null).fontFamily).toContain("monospace");
    expect(style(inner ?? null).color).toBe("rgb(164, 14, 38)");
    expect(style(inner ?? null).textAlign).toBe("center");
    expect(style(own ?? null).textAlign).toBe("start");
  });

  it("takes a token's value from the theme", () => {
    const page = draw(box([param("color", "accent"), param("line-height", "loose")], [{ type: "paragraph", children: [text("themed")] }]));
    page.style.setProperty("--compmd-text-accent", "rgb(0, 128, 0)");
    const paragraph = page.querySelector("p");
    expect(style(paragraph).color).toBe("rgb(0, 128, 0)");
    expect(parseFloat(style(paragraph).lineHeight)).toBeCloseTo(16 * 1.8, 0);
  });

  it("shows a transformed text in capitals, its text as written", () => {
    const page = draw({ type: "paragraph", children: [badge(param("text-transform", "uppercase"), param("letter-spacing", "wide"))] });
    const shown = page.querySelector("span.compmd-box");
    expect(style(shown).textTransform).toBe("uppercase");
    expect(parseFloat(style(shown).letterSpacing)).toBeCloseTo(16 * 0.05, 1);
    expect(shown?.textContent).toBe("new");
  });
});
