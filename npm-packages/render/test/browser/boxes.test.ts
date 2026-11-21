// What only a browser's layout says about two boxes (spec §7.2, §9.2): a box with `stretch-link`, whose first link
// takes a click anywhere on it while its other links stay links, and an inline box in a line of text.
import { afterEach, describe, expect, it } from "vitest";
import { createPage } from "../../src/page.ts";
import { stretchFirstLink } from "../../src/stretch-link.ts";
import type { ExpandedNode } from "../../src/tree.ts";

let root: HTMLElement;
afterEach(() => {
  root.remove();
});

function draw(...children: ExpandedNode[]): HTMLElement {
  root = document.body.appendChild(document.createElement("main"));
  root.style.cssText = "width: 468px; font: 16px/1.5 sans-serif";
  createPage(root, {}).render({ type: "root", children });
  return root;
}

const param = (key: string, value: unknown) => ({ key, breakpoint: null, value });
const text = (value: string): ExpandedNode => ({ type: "text", value });
const link = (url: string, words: string): ExpandedNode => ({ type: "link", url, title: null, children: [text(words)] });

/** What a click at the middle of `element` reaches. */
function hit(element: Element): Element | null {
  const { left, top, width, height } = element.getBoundingClientRect();
  return document.elementFromPoint(left + width / 2, top + height / 2);
}

describe("stretch-link", () => {
  const card: ExpandedNode = {
    type: "componentContainer",
    name: "box",
    params: [param("stretch-link", true), param("padding", "m")],
    children: [
      { type: "heading", depth: 3, children: [link("harbour.comp.md", "The harbour at dusk")] },
      { type: "paragraph", children: [text("A walk along the old quay, "), link("tides.comp.md", "with its tides"), text(", an hour before the boats come in.")] },
    ],
  };

  it("makes a click anywhere on the box its first link's, and leaves its other links theirs", () => {
    const page = draw(card);
    const [first, other] = [...page.querySelectorAll("a")];
    const description = page.querySelector("p")!;
    // The paragraph's middle, away from the link in it: a part of the card that is only text.
    const { left, top, height } = description.getBoundingClientRect();
    expect(document.elementFromPoint(left + 4, top + height / 2)?.closest("a")).toBe(first);
    expect(hit(other!)?.closest("a")).toBe(other);
    expect(first?.classList.contains("compmd-stretched")).toBe(true);
  });

  it("follows the box's first link when an update moves it", () => {
    root = document.body.appendChild(document.createElement("main"));
    const page = createPage(root, {});
    page.render({ type: "root", children: [card] });
    const moved = { ...card, children: [{ type: "paragraph", children: [link("quay.comp.md", "The quay")] }, ...((card as { children?: ExpandedNode[] }).children ?? [])] } as ExpandedNode;
    page.render({ type: "root", children: [moved] });
    const marked = [...root.querySelectorAll(".compmd-stretched")];
    expect(marked.map((element) => element.getAttribute("href"))).toEqual(["quay.comp.md"]);
  });

  it("leaves a link marked by the box around it when a box inside, without the flag, updates after it, as a link card's card", () => {
    root = document.body.appendChild(document.createElement("main"));
    const outer = root.appendChild(document.createElement("div"));
    const inner = outer.appendChild(document.createElement("div"));
    const first = inner.appendChild(document.createElement("a"));
    first.href = "harbour.comp.md";
    stretchFirstLink(outer, true);
    stretchFirstLink(inner, false);
    expect(first.classList.contains("compmd-stretched")).toBe(true);
    stretchFirstLink(outer, false);
    expect(first.classList.contains("compmd-stretched")).toBe(false);
  });
});

describe("an inline box", () => {
  it("sits in its line, its padding, border and background applied as a block's", () => {
    const page = draw({
      type: "paragraph",
      children: [text("The new "), { type: "componentInline", name: "box", params: [param("padding-inline", "s"), param("border", "1px solid teal"), param("background", "subtle")], fallback: [text("beta")] }, text(" search.")],
    });
    const badge = page.querySelector<HTMLElement>("span.compmd-box")!;
    const style = getComputedStyle(badge);
    expect([style.display, style.paddingLeft, style.borderTopStyle]).toEqual(["inline-block", "8px", "solid"]);
    expect(badge.textContent).toBe("beta");
    expect(page.querySelector("p")?.textContent).toBe("The new beta search.");
    const line = page.querySelector("p")!.getBoundingClientRect();
    expect(badge.getBoundingClientRect().top).toBeGreaterThanOrEqual(line.top);
    expect(badge.getBoundingClientRect().bottom).toBeLessThanOrEqual(line.bottom);
  });
});
