// Framing is invisible to layout (docs/sizing-components.md, decision 2): each use of the prototype's
// (spikes/wrap-npm-package/sizing/page.html) drawn by a trusted component in the page and by the same component in a
// strict frame, in host mode, and compared. Each component reports what it drew through `set`, since a test can't look
// into a frame; each box is found by its `id`, which stays with the page's box.
import type { HostComponent } from "@compmd/processor";
import type { ExpandedComponent, ExpandedNode, ExpandedRoot, Mount } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { createSandbox, type View } from "../src/index.ts";
import { frames } from "./harness.ts";

/** What a component drew, reported each time its size changes: its first element's size, what an SVG draws of it, its `fit` as SVG says it, and whether its box scrolls. */
const REPORT = `const report = (target, context) => {
  const send = () => {
    const element = target.firstElementChild;
    if (!element) return;
    const shown = element.getBoundingClientRect();
    let [w, h] = [shown.width, shown.height];
    const ratio = element.getAttribute("preserveAspectRatio");
    if (element.tagName === "svg" && ratio !== null && ratio !== "none") {
      const scale = (ratio.endsWith("slice") ? Math.max : Math.min)(shown.width / 8, shown.height / 8);
      [w, h] = [8 * scale, 8 * scale];
    }
    const round = (n) => Math.round(n * 10) / 10;
    context.set("result", JSON.stringify({ element: [round(shown.width), round(shown.height)], drawing: [round(w), round(h)], aspect: ratio, scrolls: target.scrollHeight > target.clientHeight }));
  };
  new ResizeObserver(send).observe(target);
  queueMicrotask(send);
};
const ALIGN = ["Min", "Mid", "Max"];
const aspect = (fit) => (fit.mode === "fill" ? "none" : "x" + ALIGN[fit.x * 2] + "Y" + ALIGN[fit.y * 2] + (fit.mode === "cover" ? " slice" : " meet"));
`;

/** The prototype's components: a QR code (an SVG with a ratio), a chart sized in script, text, a canvas, and a chart that doesn't contain its size. */
const SOURCES: Record<string, string> = {
  qr: `${REPORT}export default function (target, context) {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", "0 0 8 8");
  svg.setAttribute("preserveAspectRatio", aspect(context.fit));
  svg.style.cssText = "display: block; width: 100%; height: 100%";
  target.append(svg);
  report(target, context);
}`,
  chart: `${REPORT}export default function (target, context) {
  const box = document.createElement("div");
  box.style.cssText = "width: 100%; height: 100%; contain: size; contain-intrinsic-size: 300px 150px";
  target.append(box);
  report(target, context);
}`,
  flow: `${REPORT}export default function (target, context) {
  target.insertAdjacentHTML("beforeend", "<p style='margin: 0; font: 16px/1.5 sans-serif'>Text that flows: its height is its lines', at the width it is given, whatever the box's height.</p>");
  report(target, context);
}`,
  canvas: `${REPORT}export default function (target, context) {
  const canvas = document.createElement("canvas");
  canvas.style.cssText = "display: block; width: 100%; height: 100%; object-fit: " + context.fit.mode;
  target.append(canvas);
  report(target, context);
}`,
  growing: `${REPORT}export default function (target, context) {
  const box = document.createElement("div");
  box.style.cssText = "width: 100%; height: 150px";
  target.append(box);
  new ResizeObserver(() => { box.style.height = Math.max(150, target.clientHeight) + "px"; }).observe(target);
  report(target, context);
}`,
};
type Kind = keyof typeof SOURCES;

/** Each component trusted, as a module the test page loads, and strict, as `<kind>-framed`, the same code. */
async function components(): Promise<{ trusted: Record<string, Mount>; host: HostComponent[] }> {
  const trusted: Record<string, Mount> = {};
  const host: HostComponent[] = [];
  for (const [kind, code] of Object.entries(SOURCES)) {
    const url = URL.createObjectURL(new Blob([code], { type: "text/javascript" }));
    trusted[kind] = ((await import(/* @vite-ignore */ url)) as { default: Mount }).default;
    host.push({ name: kind, language: "js", code: "", trust: { level: "trusted" } }, { name: `${kind}-framed`, language: "js", code, trust: { level: "strict" } });
  }
  return { trusted, host };
}

interface Use {
  kind: Kind;
  params?: Record<string, string>;
  /** Made 5rem tall after the first comparison: the grid row must follow. */
  shrinks?: boolean;
}
interface Case {
  id: string;
  uses: Use[];
  wrap?: "grid" | "row" | "line";
}

const CASES: Case[] = [
  { id: "qr-none", uses: [{ kind: "qr" }] },
  { id: "qr-width", uses: [{ kind: "qr", params: { width: "10rem" } }] },
  { id: "qr-height", uses: [{ kind: "qr", params: { height: "6rem" } }] },
  { id: "qr-wide", uses: [{ kind: "qr", params: { width: "16rem", height: "6rem" } }] },
  { id: "qr-tall", uses: [{ kind: "qr", params: { width: "6rem", height: "12rem" } }] },
  { id: "qr-ratio", uses: [{ kind: "qr", params: { width: "12rem", "aspect-ratio": "2/1" } }] },
  { id: "qr-start", uses: [{ kind: "qr", params: { height: "6rem", "object-position": "start" } }] },
  { id: "qr-end", uses: [{ kind: "qr", params: { width: "6rem", height: "12rem", "object-position": "end" } }] },
  { id: "qr-cover", uses: [{ kind: "qr", params: { width: "16rem", height: "6rem", "object-fit": "cover" } }] },
  { id: "qr-fill", uses: [{ kind: "qr", params: { width: "16rem", height: "6rem", "object-fit": "fill" } }] },
  { id: "chart-none", uses: [{ kind: "chart" }] },
  { id: "chart-ratio", uses: [{ kind: "chart", params: { "aspect-ratio": "3/1" } }] },
  { id: "chart-height", uses: [{ kind: "chart", params: { height: "8rem" } }] },
  { id: "chart-box", uses: [{ kind: "chart", params: { width: "16rem", height: "6rem" } }] },
  { id: "flow-none", uses: [{ kind: "flow" }] },
  { id: "flow-height", uses: [{ kind: "flow", params: { height: "2rem" } }] },
  { id: "grid-charts", wrap: "grid", uses: [{ kind: "chart", params: { height: "20rem" }, shrinks: true }, { kind: "chart" }] },
  { id: "grid-text", wrap: "grid", uses: [{ kind: "chart", params: { height: "20rem" }, shrinks: true }, { kind: "flow" }] },
  { id: "grid-qr", wrap: "grid", uses: [{ kind: "chart", params: { height: "20rem" }, shrinks: true }, { kind: "qr" }] },
  { id: "row", wrap: "row", uses: [{ kind: "qr" }, { kind: "qr", params: { width: "5rem" } }, { kind: "chart" }] },
  { id: "inline", wrap: "line", uses: [{ kind: "qr", params: { width: "3rem" } }, { kind: "chart", params: { width: "6rem", height: "2rem" } }, { kind: "qr" }] },
];

const param = (key: string, value: unknown) => ({ key, breakpoint: null, value });

/** A use, in the page or framed, its box bordered as the prototype's and found by `id`; its report in the input `id`. */
function useNode(use: Use, id: string, framed: boolean, inline: boolean, short: boolean): ExpandedComponent {
  const params = { border: "2px solid #e5007d", ...use.params, ...(short && use.shrinks ? { height: "5rem" } : {}) };
  return {
    type: inline ? "componentInline" : "componentLeaf",
    name: framed ? `${use.kind}-framed` : use.kind,
    params: [...Object.entries(params).map(([key, value]) => param(key, value)), param("id", id), { key: "result", breakpoint: null, value: "", input: id }],
    fallback: [{ type: "text", value: use.kind }],
  };
}

const layout = (wrap: Case["wrap"]) =>
  wrap === "grid"
    ? [param("display", "grid"), param("grid-template-columns", "minmax(0, 1fr) minmax(0, 1fr)"), param("gap", "s")]
    : wrap === "row"
      ? [param("display", "flex"), param("flex-wrap", "wrap"), param("gap", "s")]
      : [];

/** Each case twice, in the page and framed, side by side in a column 468 pixels wide each, as the prototype's. */
function treeOf(cases: readonly Case[], short = false): { tree: ExpandedRoot; ids: string[] } {
  const ids: string[] = [];
  const children: ExpandedNode[] = cases.flatMap((example) =>
    [false, true].map((framed): ExpandedNode => {
      const uses = example.uses.map((use, index) => {
        const id = `${example.id}-${String(index)}-${framed ? "framed" : "page"}`;
        ids.push(id);
        return useNode(use, id, framed, example.wrap === "line", short);
      });
      if (example.wrap === "line") return { type: "paragraph", children: [{ type: "text", value: "A line with " }, ...uses] };
      return { type: "componentContainer", name: "box", params: [param("width", "468px"), ...layout(example.wrap)], children: uses };
    }),
  );
  return { tree: { type: "root", children, inputs: Object.fromEntries(ids.map((id) => [id, { type: "string", value: "" }])) }, ids };
}

const views: View[] = [];
const areas: HTMLElement[] = [];
afterEach(() => {
  for (const view of views.splice(0)) view.destroy();
  for (const area of areas.splice(0)) area.remove();
});

async function open(area: HTMLElement) {
  areas.push(area);
  const { trusted, host } = await components();
  const view = createSandbox({ base: frames, page: "host", trusted }).open(area, { components: host });
  views.push(view);
  const reports = new Map<string, unknown>();
  view.on("set", ({ input, value }) => reports.set(input, JSON.parse(value as string)));
  return { view, reports };
}

const round = (n: number) => Math.round(n * 10) / 10;
const boxOf = (id: string) => {
  const rect = document.getElementById(id)?.getBoundingClientRect();
  return rect ? [round(rect.width), round(rect.height)] : undefined;
};

/** Each use's box and report in the page, beside the same use framed: the same, once the frames have reported. */
async function expectFramedAsInPage(ids: readonly string[], reports: Map<string, unknown>, compareReports = true) {
  const pairs = () =>
    ids
      .filter((id) => id.endsWith("-page"))
      .map((id) => {
        const framed = id.replace(/-page$/, "-framed");
        return { id, page: { box: boxOf(id), drawn: compareReports ? reports.get(id) : null }, framed: { box: boxOf(framed), drawn: compareReports ? reports.get(framed) : null } };
      });
  await expect.poll(() => pairs().filter(({ page, framed }) => page.box === undefined || JSON.stringify(page) !== JSON.stringify(framed)), { timeout: 30_000, interval: 100 }).toEqual([]);
}

/** Every one of the prototype's uses framed: 28 frames, each loading its runtime, while the other test files run. */
const MANY_FRAMES = 60_000;

describe("a framed use, laid out as in the page", () => {
  // One case at a time, on screen: a browser pauses the observers of a frame off screen, so what its component
  // reports of its drawing would be stale there, though its box isn't (the test below).
  it.each(CASES.map((example) => [example.id, example] as const))("%s: the same box, and the same drawing", async (_id, example) => {
    const area = document.body.appendChild(document.createElement("div"));
    const { view, reports } = await open(area);
    const { tree, ids } = treeOf([example]);
    view.render(tree);
    await expectFramedAsInPage(ids, reports);
  }, MANY_FRAMES);

  it("follows a grid row that shrinks", async () => {
    const rows = CASES.filter(({ wrap }) => wrap === "grid");
    const area = document.body.appendChild(document.createElement("div"));
    const { view, reports } = await open(area);
    const { tree, ids } = treeOf(rows);
    view.render(tree);
    await expectFramedAsInPage(ids, reports, false);
    view.render(treeOf(rows, true).tree);
    await expectFramedAsInPage(ids, reports, false);
    expect(boxOf("grid-text-1-framed")).toEqual([230, 80]);
  }, MANY_FRAMES);

  it("has its box's size while it is off screen, which the page's messages give it", async () => {
    const area = document.body.appendChild(document.createElement("div"));
    area.style.marginTop = "4000px";
    const { view, reports } = await open(area);
    const { tree, ids } = treeOf(CASES);
    view.render(tree);
    await expectFramedAsInPage(ids, reports, false);
  }, MANY_FRAMES);

  it.each<[string, Case, (drawn: unknown) => void]>([
    ["places object-position=start on the right, right to left", { id: "rtl", uses: [{ kind: "qr", params: { height: "6rem", "object-position": "center start", dir: "rtl" } }] }, (drawn) => { expect(drawn).toMatchObject({ aspect: "xMaxYMid meet" }); }],
    ["scrolls only with scroll", { id: "scroll", uses: [{ kind: "flow", params: { height: "2rem", scroll: "true" } }] }, (drawn) => { expect(drawn).toMatchObject({ scrolls: true }); }],
    ["sizes a canvas as an image of 300 by 150", { id: "canvas", uses: [{ kind: "canvas" }] }, (drawn) => { expect(drawn).toMatchObject({ element: [464, 232] }); }],
    ["keeps a drawing that doesn't contain its size as in the page, grown", { id: "growing", wrap: "grid", uses: [{ kind: "chart", params: { height: "20rem" } }, { kind: "growing" }] }, () => undefined],
  ])("%s", async (_title, example, check) => {
    const area = document.body.appendChild(document.createElement("div"));
    const { view, reports } = await open(area);
    const { tree, ids } = treeOf([example]);
    view.render(tree);
    await expectFramedAsInPage(ids, reports);
    check(reports.get(ids.find((id) => id.endsWith("framed")) ?? ""));
  });
});
