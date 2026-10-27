import { afterEach, describe, expect, it } from "vitest";
import type { RenderError } from "./contract.ts";
import { createPage } from "./page.ts";
import { createRegion } from "./region.ts";
import type { ExpandedComponent, ExpandedNode } from "./tree.ts";

const paragraph = (value: string): ExpandedNode => ({ type: "paragraph", children: [{ type: "text", value }] });
const grid = (...params: ExpandedComponent["params"]): ExpandedComponent => ({ type: "componentContainer", name: "box", params, children: [paragraph("cell")], slots: [] });
const responsive = grid({ key: "gap", breakpoint: "md", value: "l" });

let area: HTMLElement;
afterEach(() => {
  area.remove();
});

function region(container: boolean) {
  area = document.body.appendChild(document.createElement("div"));
  if (container) area.classList.add("compmd-container");
  const target = area.appendChild(document.createElement("div"));
  const errors: RenderError[] = [];
  return { target, errors, region: createRegion(target, { onError: (error) => errors.push(error) }) };
}

describe("a region's container", () => {
  it("is reported missing, once, when its blocks use breakpoints and nothing around it is a container", () => {
    const { region: r, errors } = region(false);
    r.render([responsive]);
    r.render([responsive, paragraph("more")]);
    expect(errors).toEqual([expect.objectContaining({ reason: "no-container" })]);
  });

  it("is the caller's: an element around the region with compmd-container", () => {
    const { region: r, errors } = region(true);
    r.render([responsive]);
    expect(errors).toEqual([]);
  });

  it("isn't needed when no block uses breakpoints", () => {
    const { region: r, errors } = region(false);
    r.render([grid({ key: "gap", breakpoint: null, value: "l" })]);
    expect(errors).toEqual([]);
  });

  it("is the page, for a region inside a page", () => {
    area = document.body.appendChild(document.createElement("main"));
    const page = createPage(area);
    page.render({ type: "root", children: [paragraph("before")] });
    const target = area.appendChild(document.createElement("div"));
    const errors: RenderError[] = [];
    createRegion(target, { stylesheet: page.stylesheet, onError: (error) => errors.push(error) }).render([responsive]);
    expect(errors).toEqual([]);
  });

  it("is checked once the target is attached, as an editor's widget is after it is rendered", async () => {
    area = document.body.appendChild(document.createElement("div"));
    area.classList.add("compmd-container");
    const target = document.createElement("div");
    const errors: RenderError[] = [];
    createRegion(target, { onError: (error) => errors.push(error) }).render([responsive]);
    area.append(target);
    await new Promise((resolve) => requestAnimationFrame(resolve));
    expect(errors).toEqual([]);
  });

  it("is reported missing after the frame, for a target attached where there is none", async () => {
    area = document.body.appendChild(document.createElement("div"));
    const target = document.createElement("div");
    const errors: RenderError[] = [];
    createRegion(target, { onError: (error) => errors.push(error) }).render([responsive]);
    expect(errors).toEqual([]);
    area.append(target);
    await new Promise((resolve) => requestAnimationFrame(resolve));
    expect(errors).toEqual([expect.objectContaining({ reason: "no-container" })]);
  });
});
