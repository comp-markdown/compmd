// @vitest-environment happy-dom
import { beforeAll, describe, expect, it } from "vitest";
import mount, { parse } from "./minimermaid.ts";

beforeAll(() => {
  // happy-dom has no layout: an arrow's length is what the drawing asks it.
  Object.defineProperty(window.SVGElement.prototype, "getTotalLength", { value: () => 100, configurable: true });
  Object.defineProperty(window.Element.prototype, "animate", { value: () => ({}), configurable: true });
});

describe("parse", () => {
  it("reads chains of steps, their labels, and the direction", () => {
    expect(parse("graph LR; Found --> Reading --> Read")).toEqual({
      across: true,
      labels: new Map([
        ["Found", "Found"],
        ["Reading", "Reading"],
        ["Read", "Read"],
      ]),
      edges: [
        ["Found", "Reading"],
        ["Reading", "Read"],
      ],
    });
    const chart = parse("graph TD\n  a[A draft] --> b\n  a --> c[Done]");
    expect(chart.across).toBe(false);
    expect([...chart.labels]).toEqual([
      ["a", "A draft"],
      ["b", "b"],
      ["c", "Done"],
    ]);
    expect(chart.edges).toEqual([
      ["a", "b"],
      ["a", "c"],
    ]);
  });

  it("leaves out what it doesn't understand", () => {
    expect(parse("sequenceDiagram; Alice->>Bob: hi").labels.size).toBe(0);
    expect(parse("").edges).toEqual([]);
  });
});

describe("minimermaid", () => {
  it("draws a box per step and an arrow per link, with its text alternative", () => {
    const target = document.createElement("div");
    const fallback = { text: "Found, then reading, then read", element: document.createElement("pre") };
    const mounted = mount(target, { params: { code: "graph LR; Found --> Reading --> Read", theme: "default" }, fallback });
    const svg = target.querySelector("svg[role=img]");
    expect(svg?.getAttribute("aria-label")).toBe("Found, then reading, then read");
    expect([...target.querySelectorAll("text")].map((text) => text.textContent)).toEqual(["Found", "Reading", "Read"]);
    expect(target.querySelectorAll("g[stroke] path")).toHaveLength(4);
    mounted.update({ code: "graph TD; A --> B", theme: "dark" });
    expect([...target.querySelectorAll("text")].map((text) => text.textContent)).toEqual(["A", "B"]);
    expect(target.querySelector("text")?.getAttribute("fill")).toBe("#fff");
    mounted.destroy();
    expect(target.children).toHaveLength(0);
  });

  it("draws the same wobble at every render", () => {
    const draw = () => {
      const target = document.createElement("div");
      mount(target, { params: { code: "graph LR; A --> B", theme: "neutral" }, fallback: { text: "", element: document.createElement("pre") } });
      return target.innerHTML;
    };
    expect(draw()).toBe(draw());
  });
});
