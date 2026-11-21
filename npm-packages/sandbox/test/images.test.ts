// A document's own images in both page modes. In host mode they load as any image of the host's page does; in frame
// mode the page's frame has no network, so the host's page fetches each image of the trees it sent and gives the frame
// its file, while the alt text shows until then and stays for an image it can't have. The remote origin here is
// configured, so nothing is asked (asking is `remote-images.test.ts`'s).

import type { HostComponent } from "@compmd/processor";
import type { ExpandedNode, ExpandedRoot } from "@compmd/render";
import { afterEach, describe, expect, it } from "vitest";
import { createSandbox, type View } from "../src/index.ts";
import { allowed, frames, hits, leaf, memoryAnswers, pause, probeCode, tagged } from "./harness.ts";

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

/** Asks about no origin: the other origin is configured, as a host that shows its images configures it (spec §15). */
const asked: string[] = [];
const origins = {
  configured: [new URL(allowed).origin],
  ask: (origin: string) => {
    asked.push(origin);
    return Promise.resolve("deny" as const);
  },
  answers: memoryAnswers(),
};

/** Where the test serves the packs' files: one pack's icon is there. */
const PACKS = "/test/fixtures/packs/";

function open(mode: "frame" | "host", components: HostComponent[]) {
  const area = document.body.appendChild(document.createElement("div"));
  area.style.height = "300px";
  areas.push(area);
  const view = createSandbox({ base: frames, packs: PACKS, page: mode, origins }).open(area, { components });
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
    expect(asked).toEqual([]);
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

  it("shows a pack's image in both modes from where the host serves the pack, and links to its file there", async () => {
    const pack = (node: ExpandedNode): ExpandedNode => ({ ...node, pack: true });
    const tree = (...more: ExpandedNode[]): ExpandedRoot => ({
      type: "root",
      children: [
        {
          type: "paragraph",
          children: [pack(image("@acme/kit@1.0.0/icons/star.svg", "Star")), pack({ type: "link", url: "@acme/kit@1.0.0/guide.comp.md", title: null, children: [{ type: "text", value: "Guide" }] })],
        },
        ...more,
      ],
      inputs: { result: { type: "string", value: "" } },
    });
    const linking: HostComponent = { ...looking, code: probeCode(`async () => { const shown = ${shown.toString()}; for (let tries = 0; tries < 150 && shown(document).images.length < 1; tries++) await new Promise((done) => setTimeout(done, 20)); return { ...shown(document), href: document.querySelector("a")?.getAttribute("href") }; }`) };
    const framed = open("frame", [linking]);
    framed.view.render(tree(leaf("probe")));
    await expect.poll(() => framed.values.length, { timeout: 5000 }).toBeGreaterThan(0);
    expect(framed.values[0]).toEqual({ images: [{ alt: "Star", blob: true }], alt: [], href: `${PACKS}@acme/kit@1.0.0/guide.comp.md` });
    const hosted = open("host", []);
    hosted.view.render(tree());
    await expect.poll(() => shown(hosted.area).images.length, { timeout: 5000 }).toBe(1);
    expect(hosted.area.querySelector("a")?.getAttribute("href")).toBe(`${PACKS}@acme/kit@1.0.0/guide.comp.md`);
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
