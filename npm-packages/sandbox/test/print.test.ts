// Printing a view (docs/printing.md), in both page modes, as Chrome and WebKit print: the page light, a component
// with `print: fallback` showing its fallback instead of itself, and `view.print()` waiting for the view to settle.

import type { HostComponent } from "@compmd/processor";
import type { ExpandedComponent, ExpandedNode, ExpandedRoot } from "@compmd/render";
import { afterEach, describe, expect, it, vi } from "vitest";
import { commands } from "vitest/browser";
import { createSandbox, type SandboxOptions, type View } from "../src/index.ts";
import { frames, probeCode } from "./harness.ts";

declare module "vitest/browser" {
  interface BrowserCommands {
    /** The page as it prints, or as it shows (`vitest.config.ts`). */
    emulateMedia: (media: "print" | "screen") => Promise<void>;
  }
}

const views: View[] = [];
const areas: HTMLElement[] = [];
afterEach(async () => {
  for (const view of views.splice(0)) view.destroy();
  for (const area of areas.splice(0)) area.remove();
  vi.unstubAllGlobals();
  await commands.emulateMedia("screen");
});

/** A map that draws itself, and prints its fallback. */
const map: HostComponent = { name: "map", language: "js", code: "export default function mount(target) { target.textContent = 'the map'; }", trust: { level: "strict" }, print: "fallback" };

/** A use of `map`, its fallback `Lisbon, on a map`. */
const mapUse: ExpandedComponent = { type: "componentLeaf", name: "map", params: [], fallback: [{ type: "text", value: "Lisbon, on a map" }] };

/** A use of `probe`, its result bound to the input `result`. */
const probeUse: ExpandedComponent = { type: "componentLeaf", name: "probe", params: [{ key: "result", breakpoint: null, value: "", input: "result" }], fallback: [{ type: "text", value: "probe" }] };

const RESULT = { result: { type: "string" as const, value: "" } };

const page = (...children: ExpandedNode[]): ExpandedRoot => ({ type: "root", children, inputs: RESULT });

/** A view of `components` in `mode`, with the values its components set. */
function open(mode: "frame" | "host", components: HostComponent[], options: Partial<SandboxOptions> = {}) {
  const area = document.body.appendChild(document.createElement("div"));
  area.style.height = "300px";
  areas.push(area);
  const view = createSandbox({ base: frames, page: mode, ...options }).open(area, { components });
  views.push(view);
  const values: unknown[] = [];
  view.on("set", ({ value }) => values.push(JSON.parse(value as string)));
  return { area, view, values };
}

/** How the map's box, its fallback for paper and the page, `area`, show, as computed. */
const shown = (area: HTMLElement) => {
  const style = (selector: string) => {
    const element = area.querySelector(selector);
    return element ? getComputedStyle(element) : undefined;
  };
  return {
    box: style(".compmd-print-wrap > .compmd-component")?.display,
    paper: style(".compmd-print-paper")?.display,
    paperText: area.querySelector(".compmd-print-paper")?.textContent,
    scheme: getComputedStyle(area).colorScheme,
  };
};

/** A strict component in the page's frame that reports `shown` of its page there, once the map has drawn. */
const probe: HostComponent = {
  name: "probe",
  language: "js",
  code: probeCode(`async () => {
    for (let i = 0; i < 50 && !document.querySelector(".compmd-print-wrap .compmd-component")?.textContent; i++) await new Promise((resolve) => setTimeout(resolve, 20));
    const style = (selector) => { const element = document.querySelector(selector); return element ? getComputedStyle(element) : undefined; };
    return { box: style(".compmd-print-wrap > .compmd-component")?.display, paper: style(".compmd-print-paper")?.display, paperText: document.querySelector(".compmd-print-paper")?.textContent, scheme: style(".compmd-page")?.colorScheme };
  }`),
  trust: { level: "strict" },
};

describe("on paper", () => {
  it("a component with print: fallback shows its fallback in the page's frame, which is light (frame mode)", async () => {
    await commands.emulateMedia("print");
    const { view, values } = open("frame", [map, probe]);
    view.render(page(mapUse, probeUse));
    await vi.waitFor(() => {
      expect(values).toHaveLength(1);
    }, 10_000);
    expect(values[0]).toEqual({ box: "none", paper: "block", paperText: "Lisbon, on a map", scheme: "light" });
  });

  it("a component with print: fallback shows its fallback in the host's page (host mode)", async () => {
    const { area, view } = open("host", [map]);
    view.render(page(mapUse));
    await view.settled();
    await commands.emulateMedia("print");
    expect(shown(area)).toEqual({ box: "none", paper: "block", paperText: "Lisbon, on a map", scheme: "light" });
  });

  it("on screen, the component shows itself and its fallback for paper is hidden", async () => {
    const { area, view } = open("host", [map]);
    view.render(page(mapUse));
    await view.settled();
    const { box, paper } = shown(area);
    expect(box).not.toBe("none");
    expect(paper).toBe("none");
  });
});

describe("view.print()", () => {
  it("prints the view light, once it has settled, and puts its scheme back", async () => {
    const { area, view } = open("host", [map]);
    area.style.colorScheme = "dark";
    view.render(page(mapUse));
    const printed: { scheme: string; settled: boolean }[] = [];
    vi.stubGlobal("print", () => printed.push({ scheme: area.style.getPropertyValue("color-scheme"), settled: "compmdSettled" in area.dataset }));
    await view.print();
    expect(printed).toEqual([{ scheme: "light", settled: true }]);
    expect(area.style.colorScheme).toBe("dark");
  });
});
