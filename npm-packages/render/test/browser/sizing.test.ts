// A component in its box, in the page (docs/sizing-components.md, decisions 1 to 3): the prototype's cases
// (spikes/wrap-npm-package/sizing/page.html), drawn by the renderer, each box with a 2px border as there. The sizes are
// those the prototype measured in Chrome and WebKit alike; text, whose height is its fonts', is measured against itself.
import { afterEach, describe, expect, it } from "vitest";
import type { Fit, Mount } from "../../src/contract.ts";
import { createPage } from "../../src/page.ts";
import type { ExpandedComponent, ExpandedNode } from "../../src/tree.ts";

const NS = "http://www.w3.org/2000/svg";
const ALIGN = ["Min", "Mid", "Max"];

/** `fit` as SVG's own attribute: contain is `meet`, cover `slice`, fill `none`. */
const aspect = (fit: Fit) => (fit.mode === "fill" ? "none" : `x${String(ALIGN[fit.x * 2])}Y${String(ALIGN[fit.y * 2])} ${fit.mode === "cover" ? "slice" : "meet"}`);

/** A drawing with an aspect ratio of its own, as a QR code: an SVG with a `viewBox`, the box's size, fitted by `fit`. */
const qr: Mount = (target, { fit }) => {
  const svg = document.createElementNS(NS, "svg");
  svg.setAttribute("viewBox", "0 0 8 8");
  svg.setAttribute("preserveAspectRatio", aspect(fit));
  svg.style.cssText = "display: block; width: 100%; height: 100%";
  target.append(svg);
};

/** A drawing with no aspect ratio, sized in script from its box, as a chart: it contains its size, CSS's 300 by 150. */
const chart: Mount = (target) => {
  const box = document.createElement("div");
  box.style.cssText = "width: 100%; height: 100%; contain: size; contain-intrinsic-size: 300px 150px";
  target.append(box);
};

/** Content that flows, as a card's text. */
const flow: Mount = (target) => {
  target.insertAdjacentHTML("beforeend", "<p style='margin: 0'>Text that flows: its height is its lines', at the width it is given, whatever the box's height.</p>");
};

type Kind = "qr" | "chart" | "flow";
type Size = [width: number, height: number];

/** A use of `kind` with the box params `params`, its border the prototype's. */
const use = (kind: Kind, params: Record<string, string> = {}, inline = false): ExpandedComponent => ({
  type: inline ? "componentInline" : "componentLeaf",
  name: kind,
  params: Object.entries({ border: "2px solid #e5007d", ...params }).map(([key, value]) => ({ key, breakpoint: null, value })),
  fallback: [],
});

/** The uses in a two-column grid, as a `box` lays them out. */
const grid = (...uses: ExpandedComponent[]): ExpandedNode => ({
  type: "componentContainer",
  name: "box",
  params: [
    { key: "display", breakpoint: null, value: "grid" },
    { key: "grid-template-columns", breakpoint: null, value: "minmax(0, 1fr) minmax(0, 1fr)" },
    { key: "gap", breakpoint: null, value: "s" },
  ],
  children: uses,
});

/** The uses in a row that wraps. */
const row = (...uses: ExpandedComponent[]): ExpandedNode => ({
  type: "componentContainer",
  name: "box",
  params: [
    { key: "display", breakpoint: null, value: "flex" },
    { key: "flex-wrap", breakpoint: null, value: "wrap" },
    { key: "gap", breakpoint: null, value: "s" },
  ],
  children: uses,
});

/** The uses in a line of text. */
const line = (...uses: ExpandedComponent[]): ExpandedNode => ({ type: "paragraph", children: [{ type: "text", value: "A line with " }, ...uses] });

let root: HTMLElement;
afterEach(() => {
  root.remove();
});

/** Draws `block` in a column 468 pixels wide, the prototype's: each use's box. */
function draw(block: ExpandedNode): HTMLElement[] {
  root = document.body.appendChild(document.createElement("main"));
  root.style.cssText = "width: 468px; font: 16px/1.5 system-ui, sans-serif";
  createPage(root, { components: { qr, chart, flow } }).render({ type: "root", children: [block] });
  return [...root.querySelectorAll<HTMLElement>(".compmd-component")];
}

const round = (n: number) => Math.round(n * 10) / 10;
const sizeOf = (rect: DOMRect): Size => [round(rect.width), round(rect.height)];

/** What a box holds: its first element's size, and for a scaled SVG what is drawn of it. */
function drawn(box: HTMLElement): { box: Size; element: Size; drawing: Size } {
  const element = box.firstElementChild!;
  const shown = element.getBoundingClientRect();
  let drawing: Size = [shown.width, shown.height];
  const ratio = element.getAttribute("preserveAspectRatio");
  if (element instanceof SVGSVGElement && ratio !== null && ratio !== "none") {
    const scale = (ratio.endsWith("slice") ? Math.max : Math.min)(shown.width / 8, shown.height / 8);
    drawing = [8 * scale, 8 * scale];
  }
  return { box: sizeOf(box.getBoundingClientRect()), element: sizeOf(shown), drawing: drawing.map(round) as Size };
}

const square = (side: number, inside = side - 4): ReturnType<typeof drawn> => ({ box: [side, side], element: [inside, inside], drawing: [inside, inside] });

describe("a component in its box, in the page", () => {
  it.each<[string, ExpandedComponent, ReturnType<typeof drawn>]>([
    ["QR code, no size: the column's width, square", use("qr"), square(468)],
    ["QR code, width=10rem", use("qr", { width: "10rem" }), square(160)],
    ["QR code, height=6rem: the column's width, the drawing the height's", use("qr", { height: "6rem" }), { box: [468, 96], element: [464, 92], drawing: [92, 92] }],
    ["QR code, width=16rem height=6rem", use("qr", { width: "16rem", height: "6rem" }), { box: [256, 96], element: [252, 92], drawing: [92, 92] }],
    ["QR code, width=6rem height=12rem", use("qr", { width: "6rem", height: "12rem" }), { box: [96, 192], element: [92, 188], drawing: [92, 92] }],
    ["QR code, width=12rem aspect-ratio=2/1: the border counted in", use("qr", { width: "12rem", "aspect-ratio": "2/1" }), { box: [192, 96], element: [188, 92], drawing: [92, 92] }],
    ["QR code, width=16rem height=6rem object-fit=cover", use("qr", { width: "16rem", height: "6rem", "object-fit": "cover" }), { box: [256, 96], element: [252, 92], drawing: [252, 252] }],
    ["QR code, width=16rem height=6rem object-fit=fill", use("qr", { width: "16rem", height: "6rem", "object-fit": "fill" }), { box: [256, 96], element: [252, 92], drawing: [252, 92] }],
    ["Chart, no size: 150 tall", use("chart"), { box: [468, 154], element: [464, 150], drawing: [464, 150] }],
    ["Chart, aspect-ratio=3/1: the border counted in", use("chart", { "aspect-ratio": "3/1" }), { box: [468, 156], element: [464, 152], drawing: [464, 152] }],
    ["Chart, height=8rem", use("chart", { height: "8rem" }), { box: [468, 128], element: [464, 124], drawing: [464, 124] }],
    ["Chart, width=16rem height=6rem", use("chart", { width: "16rem", height: "6rem" }), { box: [256, 96], element: [252, 92], drawing: [252, 92] }],
    ["Text, height=2rem, shorter than its text", use("flow", { height: "2rem" }), { box: [468, 32], element: [464, 28], drawing: [464, 28] }],
  ])("%s", (_title, node, expected) => {
    const [box] = draw(node);
    expect(drawn(box!)).toEqual(expected);
  });

  it("gives text with no size its lines' height", () => {
    const [box] = draw(use("flow"));
    const { box: size, element } = drawn(box!);
    expect(element[0]).toBe(464);
    expect(size).toEqual([468, element[1] + 4]);
  });

  it("places the drawing by object-position: start and end", () => {
    const [start] = draw(use("qr", { height: "6rem", "object-position": "start" }));
    expect(start?.querySelector("svg")?.getAttribute("preserveAspectRatio")).toBe("xMinYMin meet");
    root.remove();
    const [end] = draw(use("qr", { width: "6rem", height: "12rem", "object-position": "end" }));
    expect(end?.querySelector("svg")?.getAttribute("preserveAspectRatio")).toBe("xMaxYMax meet");
  });

  it("places the inline start on the right in a right-to-left document", () => {
    draw(use("qr", { height: "6rem", "object-position": "center start", dir: "rtl" }));
    expect(root.querySelector("svg")?.getAttribute("preserveAspectRatio")).toBe("xMaxYMid meet");
  });

  it.each<[string, Kind]>([
    ["a chart with no size", "chart"],
    ["text", "flow"],
  ])("stretches %s to a grid row a chart of height=20rem makes", (_title, kind) => {
    const boxes = draw(grid(use("chart", { height: "20rem" }), use(kind)));
    expect(boxes.map(drawn)).toEqual([
      { box: [230, 320], element: [226, 316], drawing: [226, 316] },
      { box: [230, 320], element: [226, 316], drawing: [226, 316] },
    ]);
  });

  it("fits a QR code in a grid row a chart of height=20rem makes", () => {
    const [, qrBox] = draw(grid(use("chart", { height: "20rem" }), use("qr")));
    expect(drawn(qrBox!)).toEqual({ box: [230, 320], element: [226, 316], drawing: [226, 226] });
  });

  it("gives a QR code and a chart with no size CSS's 300 pixels in a row", () => {
    const boxes = draw(row(use("qr"), use("qr", { width: "5rem" }), use("chart")));
    expect(boxes.map(drawn)).toEqual([
      square(304),
      { box: [80, 304], element: [76, 300], drawing: [76, 76] },
      { box: [304, 154], element: [300, 150], drawing: [300, 150] },
    ]);
  });

  it("sizes an inline component as a block one, in a line of text", () => {
    const boxes = draw(line(use("qr", { width: "3rem" }, true), use("chart", { width: "6rem", height: "2rem" }, true), use("qr", {}, true)));
    expect(boxes.map((box) => getComputedStyle(box).display)).toEqual(["inline-grid", "inline-grid", "inline-grid"]);
    expect(boxes.map(drawn)).toEqual([square(48), { box: [96, 32], element: [92, 28], drawing: [92, 28] }, square(304)]);
  });
});
