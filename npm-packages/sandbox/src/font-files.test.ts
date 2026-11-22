// The files of a document's own fonts (spec §9.4), as the host's page gives them to its page frame and loads them in
// host mode: only those of the trees and splices it sent, never from another origin, a registry pack's from the file
// the host holds.
import type { ExpandedFont, Splice } from "@compmd/render";
import { describe, expect, it } from "vitest";
import { ImageGate } from "./image-gate.ts";
import { hostFonts, PageImages } from "./images.ts";
import { localAnswers } from "./origins.ts";

const font = (url: string, pack = false): ExpandedFont => ({ family: "serif", fallback: "body", files: [{ url, ...(pack ? { pack } : {}), weight: "normal", style: "normal" }] });
const splice = (fonts: ExpandedFont[]): Splice => ({ from: 0, to: 0, nodes: [], lines: 0, offset: 0, fonts });

/** Page images over a fetch that answers every URL with a font, recording each request; `held` the registry packs' files. */
function pageImages(held: Record<string, Blob> = {}) {
  const requests: string[] = [];
  const fetcher = (input: RequestInfo | URL) => {
    requests.push(input instanceof Request ? input.url : input.toString());
    return Promise.resolve(new Response(new Blob(["wOF2"], { type: "font/woff2" })));
  };
  const gate = new ImageGate({ configured: [], ask: () => Promise.resolve("deny"), answers: localAnswers(undefined) });
  const page = new PageImages(fetcher, "https://docs.example.com/notes/", { schemes: [] }, gate, { at: "/packs/", images: (place) => held[place] });
  return { page, requests };
}

describe("a page frame's font files", () => {
  it("are fetched from the host's page's origin for the fonts of the tree it was given, a pack's by its URL", async () => {
    const { page, requests } = pageImages();
    page.rendered({ type: "root", children: [], fonts: [font("./fonts/serif.woff2"), font("@acme/look@1.0.0/sans.woff2", true)] });
    expect(await page.file("./fonts/serif.woff2")).toBeInstanceOf(Blob);
    expect(await page.file("/packs/@acme/look@1.0.0/sans.woff2")).toBeInstanceOf(Blob);
    expect(requests).toEqual(["https://docs.example.com/notes/fonts/serif.woff2", "https://docs.example.com/packs/@acme/look@1.0.0/sans.woff2"]);
  });

  it("are given for splices' fonts too, as objects or as JSON text, and none the trees don't have", async () => {
    const { page, requests } = pageImages();
    page.rendered({ type: "root", children: [] });
    page.spliced([splice([font("./a.woff2")]), JSON.stringify(splice([font("./b.woff")]))]);
    expect(await page.file("./a.woff2")).toBeInstanceOf(Blob);
    expect(await page.file("./b.woff")).toBeInstanceOf(Blob);
    expect(await page.file("./c.woff2")).toBeUndefined();
    expect(requests).toHaveLength(2);
  });

  it("are never fetched from another origin, and a registry pack's comes from the file the host holds", async () => {
    const held = new Blob(["held"], { type: "font/woff2" });
    const { page, requests } = pageImages({ "@kit/look@2.0.0/serif.woff2": held });
    page.rendered({ type: "root", children: [], fonts: [font("https://fonts.example.com/x.woff2"), font("@kit/look@2.0.0/serif.woff2", true)] });
    expect(await page.file("https://fonts.example.com/x.woff2")).toBeUndefined();
    expect(await page.file("/packs/@kit/look@2.0.0/serif.woff2")).toBe(held);
    expect(requests).toEqual([]);
  });
});

describe("hostFonts", () => {
  it("loads a font from its URL, and a registry pack's the host holds from a blob: URL of it, made once", async () => {
    const fonts = hostFonts({ at: "/packs/", images: (place) => (place === "@kit/look@2.0.0/serif.woff2" ? new Blob(["held"]) : undefined) });
    expect(await fonts("./fonts/serif.woff2")).toBe("./fonts/serif.woff2");
    const url = await fonts("/packs/@kit/look@2.0.0/serif.woff2");
    expect(url).toMatch(/^blob:/);
    expect(await fonts("/packs/@kit/look@2.0.0/serif.woff2")).toBe(url);
    expect(await hostFonts(undefined)("/packs/x.woff2")).toBe("/packs/x.woff2");
  });
});
