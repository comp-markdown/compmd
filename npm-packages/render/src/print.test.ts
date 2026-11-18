import { afterEach, describe, expect, it, vi } from "vitest";
import type { Mount, SlotItem } from "./contract.ts";
import { createPage, type PageOptions } from "./page.ts";
import type { ExpandedComponent, ExpandedNode, ExpandedRoot } from "./tree.ts";

const text = (value: string): ExpandedNode => ({ type: "text", value });
const paragraph = (...children: ExpandedNode[]): ExpandedNode => ({ type: "paragraph", children });
const tree = (...children: ExpandedNode[]): ExpandedRoot => ({ type: "root", children });
const leaf = (name: string, fallback: ExpandedNode[]): ExpandedComponent => ({ type: "componentLeaf", name, params: [], fallback });
const inline = (name: string, fallback: ExpandedNode[]): ExpandedComponent => ({ type: "componentInline", name, params: [], fallback });

/** A component that draws `drawn` in its box, and places its default slot's items after it. */
const drawing =
  (drawn: string): Mount =>
  (target, { slots }) => {
    target.append(drawn);
    for (const item of (slots.default ?? []) as SlotItem[]) target.append(item.element);
  };

let root: HTMLElement;
function page(options: PageOptions = {}) {
  root = document.body.appendChild(document.createElement("main"));
  return createPage(root, options);
}
afterEach(() => {
  root.remove();
  vi.unstubAllGlobals();
});

describe("a component that prints its fallback", () => {
  it("has its leaf's fallback beside its box, for paper, and the box alone on screen", () => {
    page({ components: { map: drawing("the map") }, printsFallback: (name) => name === "map" }).render(tree(leaf("map", [text("Lisbon, on a map")])));
    const wrap = root.querySelector(".compmd-print-wrap");
    expect(wrap?.tagName).toBe("DIV");
    expect(wrap?.firstElementChild?.classList.contains("compmd-component")).toBe(true);
    expect(wrap?.firstElementChild?.textContent).toBe("the map");
    expect(wrap?.querySelector(".compmd-print-paper")?.textContent).toBe("Lisbon, on a map");
  });

  it("has a container's slots as Markdown for paper, and an inline component's fallback in its text", () => {
    const tabs: ExpandedComponent = { type: "componentContainer", name: "tabs", params: [], children: [paragraph(text("First tab"))], slots: [{ type: "componentSlot", name: "more", children: [paragraph(text("Second tab"))] }] };
    page({ components: { tabs: drawing("tabs"), clock: drawing("12:00") }, printsFallback: (name) => name === "tabs" || name === "clock" }).render(
      tree(tabs, paragraph(text("Now "), inline("clock", [text("noon")]))),
    );
    const [tabsWrap, clockWrap] = root.querySelectorAll(".compmd-print-wrap");
    expect([...(tabsWrap?.querySelectorAll(".compmd-print-paper p") ?? [])].map((p) => p.textContent)).toEqual(["First tab", "Second tab"]);
    expect(clockWrap?.tagName).toBe("SPAN");
    expect(clockWrap?.querySelector(".compmd-print-paper")?.textContent).toBe("noon");
  });

  it("follows its fallback as the document changes, and leaves other components unwrapped", () => {
    const shown = page({ components: { map: drawing("the map"), chart: drawing("the chart") }, printsFallback: (name) => name === "map" });
    shown.render(tree(leaf("map", [text("Lisbon")]), leaf("chart", [text("Sales")])));
    shown.render(tree(leaf("map", [text("Porto")]), leaf("chart", [text("Sales")])));
    expect(root.querySelector(".compmd-print-paper")?.textContent).toBe("Porto");
    expect(root.querySelectorAll(".compmd-print-wrap")).toHaveLength(1);
  });

  it("is styled to show its fallback only on paper", () => {
    const sheet = page({}).stylesheet;
    sheet.flush();
    const rules = [...document.adoptedStyleSheets.flatMap((s) => [...s.cssRules])].map((rule) => rule.cssText).join("\n");
    expect(rules).toMatch(/\.compmd-print-paper \{ display: none; \}/);
    expect(rules).toMatch(/@media print/);
  });
});

describe("page.print()", () => {
  it("prints in the light scheme once its components have drawn, then puts the scheme back", async () => {
    let resolveDrawing: () => void = () => undefined;
    const drawingLate: Mount = (_target, { busy }) => {
      busy(new Promise<void>((resolve) => (resolveDrawing = resolve)));
    };
    const shown = page({ components: { chart: drawingLate } });
    root.style.colorScheme = "dark";
    shown.render(tree(leaf("chart", [text("Sales")])));
    const printed: string[] = [];
    // happy-dom has no `print`: the window's is the test's.
    vi.stubGlobal("print", () => printed.push(root.style.getPropertyValue("color-scheme")));
    const printing = shown.print();
    await Promise.resolve();
    expect(printed).toEqual([]);
    resolveDrawing();
    await printing;
    expect(printed).toEqual(["light"]);
    expect(root.style.colorScheme).toBe("dark");
  });
});
