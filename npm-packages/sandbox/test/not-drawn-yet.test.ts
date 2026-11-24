// A component not drawn yet (spec §7.4): until its mount returns, its use shows its fallback, inline and as a block, in
// both page modes, then its drawing in its place, the text around it moving once. The server holds back the
// component's built module (`setup.ts`, `packs/held.js`), as a frame's code is on a cold load.

import type { HostComponent } from "@compmd/processor";
import type { ExpandedNode, ExpandedRoot } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { createSandbox, type View } from "../src/index.ts";
import { frames, probeCode } from "./harness.ts";

const views: View[] = [];
const areas: HTMLElement[] = [];
afterEach(() => {
  for (const view of views.splice(0)) view.destroy();
  for (const area of areas.splice(0)) area.remove();
});

/** The held module's place, which the host gives by its URL. */
const HELD = { pack: "@kit/held", version: "1.0.0", path: "dist/held.js" };

/** The held module, as `setup.ts` writes it: fetched here, it would come from the cache to the frame at once. */
const HELD_MODULE = 'export default function mount(target) { target.textContent = "held"; }\n';

/** `text`'s SHA-384, as an integrity hash. */
async function integrityOf(text: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-384", new TextEncoder().encode(text));
  return `sha384-${btoa(String.fromCharCode(...new Uint8Array(digest)))}`;
}

/** The component whose module is held back; it draws `held`. */
const held: HostComponent = { name: "rating", language: "js", code: "", trust: { level: "strict" }, module: HELD };

/** A strict component in the page's frame that reports whether the page showed the fallbacks before it drew `held`. */
const watcher: HostComponent = {
  name: "watcher",
  language: "js",
  code: probeCode(`async () => {
    let sawFallbacks = false;
    for (let tries = 0; tries < 400; tries++) {
      const text = document.body.textContent ?? "";
      if (text.includes("two of five") && text.includes("A block's fallback")) sawFallbacks = true;
      if (text.includes("The cheese was held at best.") && !text.includes("two of five")) return { sawFallbacks, drawn: true };
      await new Promise((done) => setTimeout(done, 20));
    }
    return { sawFallbacks, drawn: false };
  }`),
  trust: { level: "strict" },
};

const text = (value: string): ExpandedNode => ({ type: "text", value });

/** "The cheese was `rating` at best.", then a block `rating`, and a watcher bound to the input `result`. */
const TREE: ExpandedRoot = {
  type: "root",
  children: [
    { type: "paragraph", children: [text("The cheese was "), { type: "componentInline", name: "rating", params: [], fallback: [text("two of five")] }, text(" at best.")] },
    { type: "componentLeaf", name: "rating", params: [], fallback: [text("A block's fallback")] },
    { type: "componentLeaf", name: "watcher", params: [{ key: "result", breakpoint: null, value: "", input: "result" }], fallback: [text("")] },
  ],
  inputs: { result: { type: "string", value: "" } },
};

async function open(page: "frame" | "host") {
  // A URL of its own for each page, so no frame gets it from the cache.
  const url = `${frames}packs/held.js?for=${page}-${String(Date.now())}`;
  const integrity = await integrityOf(HELD_MODULE);
  const area = document.body.appendChild(document.createElement("div"));
  areas.push(area);
  const view = createSandbox({ base: frames, page, modules: (place) => (place.path === HELD.path ? { url, integrity } : undefined) }).open(area, { components: [held, watcher] });
  views.push(view);
  const values: unknown[] = [];
  view.on("set", ({ value }) => values.push(JSON.parse(value as string)));
  return { area, view, values };
}

/** Where the text after the inline use starts, in the host's page. */
function afterInline(area: HTMLElement): number {
  const paragraph = area.querySelector("p");
  const last = paragraph?.lastChild;
  if (!last) return Number.NaN;
  const range = document.createRange();
  range.selectNodeContents(last);
  return Math.round(range.getBoundingClientRect().left);
}

describe("a component not drawn yet", () => {
  it("shows its fallback in the host's page until its frame has mounted it, then its drawing, the text moving once (host mode)", async () => {
    const { area, view } = await open("host");
    view.render(TREE);
    // Before the module comes: the fallbacks, in the uses' places.
    await expect.poll(() => area.querySelector("p")?.textContent, { timeout: 1000 }).toBe("The cheese was two of five at best.");
    expect(area.textContent).toContain("A block's fallback");
    const places = new Set<number>();
    let sampling = true;
    const sample = () => {
      places.add(afterInline(area));
      if (sampling) requestAnimationFrame(sample);
    };
    sample();
    // Then the drawings, the fallbacks gone.
    await expect.poll(() => area.textContent.includes("two of five"), { timeout: 10_000 }).toBe(false);
    expect(area.textContent).not.toContain("A block's fallback");
    await view.settled();
    sampling = false;
    expect(places.size, "where the text after the inline use started, at each frame").toBeLessThanOrEqual(2);
  });

  it("shows its fallback in the page's frame until its code has loaded, then its drawing (frame mode)", async () => {
    const { view, values } = await open("frame");
    view.render(TREE);
    await expect.poll(() => values.length, { timeout: 15_000 }).toBeGreaterThan(0);
    expect(values[0]).toEqual({ sawFallbacks: true, drawn: true });
  });
});
