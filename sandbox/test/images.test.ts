// A document's own images in both page modes. In host mode they load as any image of the host's page does; in frame
// mode the page's frame has no network, so the host's page fetches each image of the trees it sent and gives the frame
// its file, while the alt text shows until then and stays for an image it can't have.

import type { HostComponent } from "@compmd/processor";
import type { ExpandedNode, ExpandedRoot } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { createSandbox, type View } from "../src/index.ts";
import { allowed, frames, hits, leaf, pause, probeCode, tagged } from "./harness.ts";

const views: View[] = [];
const areas: HTMLElement[] = [];
afterEach(() => {
  for (const view of views.splice(0)) view.destroy();
  for (const area of areas.splice(0)) area.remove();
});

/** What a page shows of its images: each loaded image's alt text and whether it is a `blob:` URL, and the alt texts left. */
const shown = (root: ParentNode) => ({
  images: [...root.querySelectorAll("img")].filter((image) => image.complete && image.naturalWidth > 0).map((image) => ({ alt: image.alt, blob: image.src.startsWith("blob:") })),
  alt: [...root.querySelectorAll(".compmd-image-alt")].map((alt) => alt.textContent),
});

const image = (url: string, alt: string): ExpandedNode => ({ type: "image", url, alt });

/** A document with an image of the host's page's origin, one from another origin, and one that isn't an image. */
function withImages(tag: string, ...more: ExpandedNode[]): ExpandedRoot {
  return {
    type: "root",
    children: [
      {
        type: "paragraph",
        children: [image("/test/fixtures/square.svg", "Here"), image(`${allowed}${tag}/square.svg`, "Elsewhere"), image(`${allowed}${tag}/page.txt`, "Not an image")],
      },
      ...more,
    ],
    inputs: { result: { type: "string", value: "" } },
  };
}

/** A strict component in the page's frame that reports what the frame's page shows once two images have loaded. */
const looking: HostComponent = {
  name: "probe",
  language: "js",
  code: probeCode(`async () => {
    const shown = ${shown.toString()};
    for (let tries = 0; tries < 150 && shown(document).images.length < 2; tries++) await new Promise((done) => setTimeout(done, 20));
    return shown(document);
  }`),
  trust: { level: "strict" },
};

function open(mode: "frame" | "host", components: HostComponent[]) {
  const area = document.body.appendChild(document.createElement("div"));
  area.style.height = "300px";
  areas.push(area);
  const view = createSandbox({ base: frames, page: mode }).open(area, { components });
  views.push(view);
  const values: unknown[] = [];
  view.on("set", ({ value }) => values.push(JSON.parse(value as string)));
  return { area, view, values };
}

describe("a document's own images", () => {
  it("shows them in frame mode from files the host's page fetched, and the alt text of one that isn't an image", async () => {
    const tag = tagged();
    const { view, values } = open("frame", [looking]);
    view.render(withImages(tag, leaf("probe")));
    await expect.poll(() => values.length, { timeout: 5000 }).toBeGreaterThan(0);
    expect(values[0]).toEqual({
      images: [
        { alt: "Here", blob: true },
        { alt: "Elsewhere", blob: true },
      ],
      alt: ["Not an image"],
    });
    expect(await hits(allowed, tag)).toEqual([`/${tag}/square.svg`, `/${tag}/page.txt`]);
  });

  it("loads them in host mode as the host's page loads any image", async () => {
    const tag = tagged();
    const { area, view } = open("host", []);
    view.render(withImages(tag));
    await expect.poll(() => shown(area).images.length, { timeout: 5000 }).toBe(2);
    expect(shown(area)).toEqual({
      images: [
        { alt: "Here", blob: false },
        { alt: "Elsewhere", blob: false },
      ],
      alt: [],
    });
  });

  it("keeps the page frame's policy in frame mode: code in it still loads no image itself", async () => {
    const tag = tagged();
    const making: HostComponent = {
      name: "probe",
      language: "js",
      code: probeCode(`async (target) => {
        const made = Object.assign(document.createElement("img"), { src: ${JSON.stringify(`${allowed}${tag}/made.svg`)} });
        target.append(made);
        await new Promise((done) => { made.onload = done; made.onerror = done; });
        return made.naturalWidth;
      }`),
      trust: { level: "strict" },
    };
    const { view, values } = open("frame", [making]);
    view.render({ type: "root", children: [leaf("probe")], inputs: { result: { type: "string", value: "" } } });
    await expect.poll(() => values.length, { timeout: 5000 }).toBeGreaterThan(0);
    expect(values[0]).toBe(0);
    await pause(200);
    expect(await hits(allowed, tag)).toEqual([]);
  });
});
