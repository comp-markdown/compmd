import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { MountContext } from "./contract.ts";
import { rendersSlots, type NodesContext } from "./nodes.ts";
import { createPage } from "./page.ts";
import type { ExpandedComponent, ExpandedInput, ExpandedNode } from "./tree.ts";

const paragraph = (text: string): ExpandedNode => ({ type: "paragraph", children: [{ type: "text", value: text }] });

/** `::: panel` around a paragraph and a chart, its title bound to the input `title`. */
const panel = (text: string): ExpandedComponent => ({
  type: "componentContainer",
  name: "panel",
  params: [{ key: "title", breakpoint: null, value: "Sales", input: "title" }],
  children: [paragraph(text), { type: "componentLeaf", name: "chart", params: [] }],
});

const inputs: Record<string, ExpandedInput> = { title: { type: "string", value: "Sales" } };

let area: HTMLElement;
beforeEach(() => {
  area = document.body.appendChild(document.createElement("main"));
});
afterEach(() => {
  area.remove();
});

describe("a component that renders its slots itself", () => {
  it("gets the use's node and the inputs, and the renderer renders none of its slots", () => {
    const mounted: NodesContext[] = [];
    const updated: [ExpandedComponent, unknown][] = [];
    const destroyed: string[] = [];
    let chartMounts = 0;
    const page = createPage(area, {
      components: {
        panel: rendersSlots((target, context) => {
          mounted.push(context);
          target.textContent = "framed";
          return { update: (node, next) => updated.push([node, next]), destroy: () => destroyed.push("panel") };
        }),
        chart: () => {
          chartMounts += 1;
        },
      },
    });
    page.render({ type: "root", children: [panel("Before")], inputs });
    expect(area.querySelector(".compmd-component")?.textContent).toBe("framed");
    expect(area.querySelector(".compmd-item")).toBeNull();
    expect(chartMounts).toBe(0);
    expect(mounted[0]?.node).toEqual(panel("Before"));
    expect(mounted[0]?.inputs).toEqual(inputs);
    expect(mounted[0]?.slots).toEqual({});
    expect(mounted[0]?.params).toEqual({ title: "Sales" });

    page.render({ type: "root", children: [panel("After")], inputs });
    expect(updated).toEqual([[panel("After"), inputs]]);
    expect(mounted).toHaveLength(1);
    page.render({ type: "root", children: [] });
    expect(destroyed).toEqual(["panel"]);
  });

  it("renders a code block as its renderer's leaf, with the block's text as `code`", () => {
    const nodes: ExpandedComponent[] = [];
    const page = createPage(area, {
      components: {
        mermaid: rendersSlots((_target, { node }) => {
          nodes.push(node);
          return { update: (next) => nodes.push(next) };
        }),
      },
    });
    const block = (value: string): ExpandedNode => ({
      type: "code",
      lang: "mermaid",
      meta: null,
      value,
      component: { name: "mermaid", params: [{ key: "code", breakpoint: null, value }] },
    });
    page.render({ type: "root", children: [block("A --> B")] });
    page.render({ type: "root", children: [block("A --> C")] });
    expect(nodes.map((node) => [node.type, node.name, node.params[0]?.value])).toEqual([
      ["componentLeaf", "mermaid", "A --> B"],
      ["componentLeaf", "mermaid", "A --> C"],
    ]);
  });

  it("shows the use's fallback when it throws, and is never called as a plain mount", () => {
    const page = createPage(area, {
      components: {
        panel: rendersSlots(() => {
          throw new Error("no frame");
        }),
      },
    });
    page.render({ type: "root", children: [panel("Kept")] });
    expect(area.textContent).toContain("Kept");
    const plain = rendersSlots(() => undefined) as (target: Element, context: MountContext) => unknown;
    expect(() => plain(area, {} as MountContext)).toThrow("runs only through the renderer");
  });
});
