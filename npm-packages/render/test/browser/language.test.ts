// What only a browser's layout says about a document's language and direction (spec §2.1, §12.3): an Arabic
// document's columns run right to left, an Arabic example in an English page does too, and a document that writes
// neither keeps where it is shown's.
import { afterEach, describe, expect, it } from "vitest";
import { createPage } from "../../src/page.ts";
import type { ExpandedNode, ExpandedRoot } from "../../src/tree.ts";

let root: HTMLElement;
afterEach(() => {
  root.remove();
});

const param = (key: string, value: unknown) => ({ key, breakpoint: null, value });
const paragraph = (value: string): ExpandedNode => ({ type: "paragraph", children: [{ type: "text", value }] });

/** Two boxes side by side, their texts `first` and `first`, then "second". */
const row = (first: string): ExpandedNode => ({
  type: "componentContainer",
  name: "box",
  params: [param("display", "grid"), param("grid-template-columns", "1fr 1fr")],
  children: [{ type: "componentContainer", name: "box", params: [], children: [paragraph(first)] }, { type: "componentContainer", name: "box", params: [], children: [paragraph(`${first} second`)] }],
});

function draw(tree: ExpandedRoot) {
  root = document.body.appendChild(document.createElement("main"));
  root.style.cssText = "width: 600px";
  const page = createPage(root, {});
  page.render(tree);
  return page;
}

/** Whether the row's first box, holding `text`, is to the right of its second. */
const firstOnTheRight = (text: string) => {
  const box = (words: string) => [...root.querySelectorAll("p")].find((p) => p.textContent === words)!.getBoundingClientRect();
  return box(text).left > box(`${text} second`).left;
};

describe("a document's language and direction", () => {
  it("lays an Arabic document out right to left", () => {
    draw({ type: "root", lang: "ar", dir: "rtl", children: [row("first")] });
    expect([root.lang, root.dir]).toEqual(["ar", "rtl"]);
    expect(firstOnTheRight("first")).toBe(true);
  });

  it("lays an Arabic example out right to left in an English page", () => {
    const example: ExpandedNode = { type: "code", lang: "compmd", meta: "{render=output}", value: "", render: "output", document: { type: "root", lang: "ar", dir: "rtl", example: 1, children: [row("example")] } };
    draw({ type: "root", lang: "en", dir: "ltr", children: [row("page"), example] });
    expect(root.querySelector(".compmd-example-document")?.getAttribute("dir")).toBe("rtl");
    expect(firstOnTheRight("example")).toBe(true);
    expect(firstOnTheRight("page")).toBe(false);
  });

  it("keeps where it is shown's when it writes neither, and takes back what it set", () => {
    const page = draw({ type: "root", lang: "ar", dir: "rtl", children: [row("first")] });
    page.render({ type: "root", children: [row("first")] });
    expect([root.hasAttribute("lang"), root.hasAttribute("dir")]).toEqual([false, false]);
    root.dir = "rtl";
    page.render({ type: "root", children: [row("first")] });
    expect(root.dir).toBe("rtl");
    expect(firstOnTheRight("first")).toBe(true);
  });
});
