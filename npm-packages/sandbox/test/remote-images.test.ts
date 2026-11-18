// A document's remote images, asked about (spec §15, docs/remote-images.md), in both page modes: an image of an origin
// the reader hasn't decided on asks the first time it is drawn; refused, it shows its alt text with a button; the
// button asks again, from the host's page, and once the origin is allowed the image shows.

import type { HostComponent } from "@compmd/processor";
import type { ExpandedNode, ExpandedRoot } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { createSandbox, type OriginAnswer, type OriginKind, type View } from "../src/index.ts";
import { allowed, frames, leaf, memoryAnswers, probeCode, tagged } from "./harness.ts";

const views: View[] = [];
const areas: HTMLElement[] = [];
afterEach(() => {
  for (const view of views.splice(0)) view.destroy();
  for (const area of areas.splice(0)) area.remove();
});

const ORIGIN = new URL(allowed).origin;

/** A document with one image of another origin, and more nodes after it. */
const withImage = (tag: string, ...more: ExpandedNode[]): ExpandedRoot => ({
  type: "root",
  children: [{ type: "paragraph", children: [{ type: "image", url: `${allowed}${tag}/square.svg`, alt: "A square" }] }, ...more],
  inputs: { result: { type: "string", value: "" } },
});

/** A view whose reader gives `answers` in order, recording each question. */
function open(mode: "frame" | "host", components: HostComponent[], ...answers: OriginAnswer[]) {
  const area = document.body.appendChild(document.createElement("div"));
  area.style.height = "300px";
  areas.push(area);
  const asked: { origin: string; kinds: readonly OriginKind[] }[] = [];
  const ask = (origin: string, _document: string, kinds: readonly OriginKind[]) => {
    asked.push({ origin, kinds });
    return Promise.resolve(answers.shift() ?? "deny");
  };
  const view = createSandbox({ base: frames, page: mode, origins: { ask, answers: memoryAnswers() } }).open(area, { components });
  views.push(view);
  const values: unknown[] = [];
  view.on("set", ({ value }) => values.push(JSON.parse(value as string)));
  return { area, view, values, asked };
}

/** A strict component in the page's frame: waits for the refused image, reports it, presses its button, then reports the image that shows. */
const pressing: HostComponent = {
  name: "probe",
  language: "js",
  code: probeCode(`async () => {
    const wait = async (found) => { for (let tries = 0; tries < 150 && !found(); tries++) await new Promise((done) => setTimeout(done, 20)); return found(); };
    const refused = await wait(() => document.querySelector(".compmd-image-refused"));
    const before = { alt: refused?.firstElementChild?.textContent, button: refused?.querySelector("button")?.textContent };
    refused?.querySelector("button")?.click();
    const image = await wait(() => [...document.querySelectorAll("img")].find((img) => img.complete && img.naturalWidth > 0));
    return { before, after: image ? { alt: image.alt, blob: image.src.startsWith("blob:") } : null };
  }`),
  trust: { level: "strict" },
};

describe("a remote image, asked about", () => {
  it("asks the first time it is drawn, refused shows its alt text with a button, and the button asks again (host mode)", async () => {
    const { area, view, asked } = open("host", [], "deny", "allow");
    view.render(withImage(tagged()));
    await expect.poll(() => area.querySelector(".compmd-image-refused")).not.toBeNull();
    const refused = area.querySelector(".compmd-image-refused");
    expect(refused?.firstElementChild?.textContent).toBe("A square");
    expect(refused?.querySelector("button")?.textContent).toBe(`Show images from ${new URL(allowed).host}`);
    expect(asked).toEqual([{ origin: ORIGIN, kinds: ["images"] }]);
    refused?.querySelector("button")?.click();
    await expect.poll(() => [...area.querySelectorAll("img")].filter((img) => img.complete && img.naturalWidth > 0).length, { timeout: 5000 }).toBe(1);
    expect(area.querySelector("img")?.alt).toBe("A square");
    expect(asked).toHaveLength(2);
  });

  it("is asked about by the host's page for the page's frame, whose button asks its parent (frame mode)", async () => {
    const { view, values, asked } = open("frame", [pressing], "deny", "allow");
    view.render(withImage(tagged(), leaf("probe")));
    await expect.poll(() => values.length, { timeout: 8000 }).toBeGreaterThan(0);
    expect(values[0]).toEqual({ before: { alt: "A square", button: `Show images from ${new URL(allowed).host}` }, after: { alt: "A square", blob: true } });
    expect(asked).toEqual([
      { origin: ORIGIN, kinds: ["images"] },
      { origin: ORIGIN, kinds: ["images"] },
    ]);
  });
});
