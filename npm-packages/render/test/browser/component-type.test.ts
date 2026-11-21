// What only a browser's styles say about a component's `type` (spec §7.4): its box's type as the box shows it, its own
// params and what it inherits, for a component that draws on a canvas; the new one, and an update, when it changes.
import { afterEach, describe, expect, it } from "vitest";
import type { Mount, TypeStyle } from "../../src/contract.ts";
import { createPage } from "../../src/page.ts";
import type { ExpandedNode } from "../../src/tree.ts";

let root: HTMLElement;
afterEach(() => {
  root.remove();
});

const param = (key: string, value: unknown) => ({ key, breakpoint: null, value });

/** A component that records the type it was given at each mount and update. */
function recording() {
  const types: TypeStyle[] = [];
  const mount: Mount = (_target, context) => {
    types.push({ ...context.type });
    return { update: () => types.push({ ...context.type }) };
  };
  return { types, mount };
}

const use = (...params: { key: string; breakpoint: null; value: unknown }[]): ExpandedNode => ({ type: "componentLeaf", name: "chart", params, fallback: [{ type: "text", value: "A chart" }] });

const settled = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("a component's type", () => {
  it("is its box's, its own params over what it inherits", async () => {
    root = document.body.appendChild(document.createElement("main"));
    root.style.cssText = "font: italic 20px/1.5 serif; color: rgb(0, 0, 128)";
    const { types, mount } = recording();
    createPage(root, { components: { chart: mount } }).render({ type: "root", children: [use(param("font-size", "s"), param("font-weight", "bold"))] });
    await settled();
    expect(types[0]).toMatchObject({ size: 17.5, weight: "700", style: "italic", color: "rgb(0, 0, 128)" });
    expect(types[0]?.family).toContain("serif");
  });

  it("is the new one after an update that changes it, with an update", async () => {
    root = document.body.appendChild(document.createElement("main"));
    root.style.fontSize = "16px";
    const { types, mount } = recording();
    const page = createPage(root, { components: { chart: mount } });
    page.render({ type: "root", children: [use(param("font-size", "s"))] });
    await settled();
    page.render({ type: "root", children: [use(param("font-size", "xl"), param("color", "#a40e26"))] });
    await settled();
    expect(types.at(-1)).toMatchObject({ size: 24, color: "rgb(164, 14, 38)" });
  });

  it("follows what changes around it when the host says so, and updates only then", async () => {
    root = document.body.appendChild(document.createElement("main"));
    root.style.fontSize = "16px";
    const { types, mount } = recording();
    const page = createPage(root, { components: { chart: mount } });
    page.render({ type: "root", children: [use()] });
    await settled();
    const before = types.length;
    page.restyle();
    expect(types).toHaveLength(before);
    root.style.fontSize = "20px";
    page.restyle();
    expect(types.at(-1)?.size).toBe(20);
  });
});
