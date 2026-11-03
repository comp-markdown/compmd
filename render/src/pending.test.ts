import { afterEach, describe, expect, it } from "vitest";
import type { Mount, SlotItem } from "./contract.ts";
import { createPage, type PageOptions } from "./page.ts";
import type { ExpandedNode, ExpandedRoot } from "./tree.ts";

const text = (value: string): ExpandedNode => ({ type: "text", value });
const paragraph = (...children: ExpandedNode[]): ExpandedNode => ({ type: "paragraph", children });
const pending = (name: string, ...children: ExpandedNode[]): ExpandedNode => ({ type: "componentPending", name, pack: "@kit/charts", children });
const tree = (...children: ExpandedNode[]): ExpandedRoot => ({ type: "root", children });

let root = document.createElement("main");
function page(options: PageOptions = {}) {
  root = document.body.appendChild(document.createElement("main"));
  return createPage(root, options);
}
afterEach(() => {
  root.remove();
});

describe("a use of a component on its way", () => {
  it("shows its fallback in a busy box, a block's in a div and an inline one's in a span", () => {
    page().render(tree(pending("chart", paragraph(text("Sales by month"))), paragraph(text("A "), pending("icon", text("star")), text("."))));
    const block = root.querySelector("div.compmd-pending");
    expect(block?.getAttribute("aria-busy")).toBe("true");
    expect(block?.textContent).toBe("Sales by month");
    const inline = root.querySelector("p > span.compmd-pending");
    expect(inline?.getAttribute("aria-busy")).toBe("true");
    expect(inline?.textContent).toBe("star");
  });

  it("is drawn by its component once the pack arrives, in place", () => {
    const p = page({ components: { chart: (target) => void (target.textContent = "drawn") } });
    p.render(tree(paragraph(text("Before.")), pending("chart", paragraph(text("Sales"))), paragraph(text("After."))));
    const before = root.querySelector("p");
    p.render(tree(paragraph(text("Before.")), { type: "componentLeaf", name: "chart", params: [], fallback: [text("Sales")] }, paragraph(text("After."))));
    expect(root.querySelector(".compmd-pending")).toBeNull();
    expect(root.textContent).toBe("Before.drawnAfter.");
    expect(root.querySelector("p")).toBe(before);
  });

  it("is an item of a slot with its name, pending, and no params", () => {
    const seen: SlotItem[] = [];
    const tabs: Mount = (target, { slots }) => {
      for (const item of (slots.default ?? []) as SlotItem[]) {
        seen.push(item);
        target.append(item.element);
      }
    };
    page({ components: { tabs } }).render(
      tree({ type: "componentContainer", name: "tabs", params: [], children: [pending("chart", paragraph(text("Sales")))], slots: [] }),
    );
    expect(seen.map((item) => item.component)).toEqual([{ name: "chart", params: {}, pending: true }]);
    expect(seen[0]?.element.textContent).toBe("Sales");
  });
});
