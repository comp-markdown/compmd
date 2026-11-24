// A document's own fonts (spec §9.4) in both page modes, in Chrome and WebKit. In frame mode the page's frame has no
// network, so the host's page gives it each font's file, which it loads from a `blob:` URL its policy allows; in host
// mode a component's frame is sent its box's font's files with the box's type, and declares them. Either way, what the
// component draws is in the box's font.

import type { HostComponent } from "@compmd/processor";
import type { ExpandedNode, ExpandedRoot } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { createSandbox, type View } from "../src/index.ts";
import { frames, leaf, probeCode } from "./harness.ts";

const views: View[] = [];
const areas: HTMLElement[] = [];
afterEach(() => {
  for (const view of views.splice(0)) view.destroy();
  for (const area of areas.splice(0)) area.remove();
});

/** Where the test serves the packs' files: the look pack's font is there. */
const PACKS = "/test/fixtures/packs/";

/** What a document shows of its font: whether a face of the font has loaded, and whether `element` is set in it. */
const inFont = (element: Element) => {
  const faces = [...document.fonts].filter((face) => face.family.replace(/"/g, "").endsWith("-serif"));
  return { loaded: faces.some((face) => face.status === "loaded"), set: getComputedStyle(element).fontFamily.replace(/"/g, "").startsWith("compmd-font-") };
};

/** A strict component that reports what its document shows of the font, once a face has loaded. */
const probe: HostComponent = {
  name: "probe",
  language: "js",
  code: probeCode(`async (target) => {
    const inFont = ${inFont.toString()};
    target.textContent = "Drawn in the box's font.";
    for (let tries = 0; tries < 150 && !inFont(target).loaded; tries++) await new Promise((done) => setTimeout(done, 20));
    return inFont(target);
  }`),
  trust: { level: "strict" },
};

/** A box in the pack's font holding `children`. */
const inSerif = (...children: ExpandedNode[]): ExpandedNode => ({ type: "componentContainer", name: "box", params: [{ key: "font-family", breakpoint: null, value: "serif" }], children, slots: [] });

/** The look pack's font, served under `PACKS`. */
const SERVED = "@acme/look@1.0.0/fonts/main.woff2";
/** A registry pack's font, which nothing serves under `PACKS`: the host holds its file (`packFiles`). */
const HELD = "@kit/look@2.0.0/fonts/main.woff2";

const tree = (font = SERVED, ...children: ExpandedNode[]): ExpandedRoot => ({
  type: "root",
  children,
  inputs: { result: { type: "string", value: "" } },
  fonts: [{ family: "serif", fallback: "heading", files: [{ url: font, pack: true, weight: "normal", style: "normal" }] }],
});

/** The files the host holds of registry packs: `HELD`, the bytes of the served font. */
async function heldFiles(): Promise<(place: string) => Blob | undefined> {
  const bytes = await (await fetch(`${PACKS}${SERVED}`)).blob();
  const font = new Blob([bytes], { type: "font/woff2" });
  return (place) => (place === HELD ? font : undefined);
}

function open(mode: "frame" | "host", packFiles?: (place: string) => Blob | undefined) {
  const area = document.body.appendChild(document.createElement("div"));
  area.style.height = "300px";
  areas.push(area);
  const view = createSandbox({ base: frames, packs: PACKS, page: mode, ...(packFiles ? { packFiles } : {}) }).open(area, { components: [probe] });
  views.push(view);
  const values: unknown[] = [];
  view.on("set", ({ value }) => values.push(JSON.parse(value as string)));
  return { area, view, values };
}

describe("a document's own font", () => {
  it("loads in the page's frame from the file its host gives it, and a component there draws in it (frame mode)", async () => {
    const { view, values } = open("frame");
    view.render(tree(SERVED, inSerif({ type: "paragraph", children: [{ type: "text", value: "The harbour at dusk." }] }, leaf("probe"))));
    await expect.poll(() => values.length, { timeout: 10_000 }).toBeGreaterThan(0);
    expect(values[0]).toEqual({ loaded: true, set: true });
  });

  it("reaches a component's frame with its box's type, which declares it, and the component draws in it (host mode)", async () => {
    const { area, view, values } = open("host");
    view.render(tree(SERVED, inSerif({ type: "paragraph", children: [{ type: "text", value: "The harbour at dusk." }] }, leaf("probe"))));
    await expect.poll(() => values.length, { timeout: 10_000 }).toBeGreaterThan(0);
    expect(values[0]).toEqual({ loaded: true, set: true });
    // The page's own text is in it too.
    await expect.poll(() => inFont(area.querySelector("p")!).loaded, { timeout: 10_000 }).toBe(true);
  });
});

describe("a font of a pack fetched from a registry", () => {
  for (const mode of ["frame", "host"] as const) {
    it(`loads from the file the host holds, which nothing serves under \`packs\`, and a component draws in it (${mode} mode)`, async () => {
      const { view, values } = open(mode, await heldFiles());
      view.render(tree(HELD, inSerif({ type: "paragraph", children: [{ type: "text", value: "The harbour at dusk." }] }, leaf("probe"))));
      await expect.poll(() => values.length, { timeout: 10_000 }).toBeGreaterThan(0);
      expect(values[0]).toEqual({ loaded: true, set: true });
    });
  }
});
