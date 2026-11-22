// Fonts of a document's own (spec §9.4), as a browser loads them: a box in a pack's font shows its fallback after it,
// the font loads, a file that isn't there leaves the fallback, an example keeps a font of its own of the same name,
// and the page waits for the fonts' rules before it has drawn.
import { afterEach, describe, expect, it, vi } from "vitest";
import { createPage, type Page } from "../../src/page.ts";
import type { ExpandedFont, ExpandedNode, ExpandedRoot } from "../../src/tree.ts";

let root: HTMLElement;
let page: Page;
afterEach(() => {
  page.destroy();
  root.remove();
});

const param = (key: string, value: unknown) => ({ key, breakpoint: null, value });

/** A box in the font `family`, holding `words`. */
const inFont = (family: string, words: string): ExpandedNode => ({
  type: "componentContainer",
  name: "box",
  params: [param("font-family", family)],
  children: [{ type: "paragraph", children: [{ type: "text", value: words }] }],
});

/** A font `family` whose one file is the pack's `file`. */
const packFont = (family: string, file: string): ExpandedFont => ({
  family,
  fallback: "heading",
  files: [{ url: `@acme/look@1.0.0/fonts/${file}`, pack: true, weight: "normal", style: "normal" }],
});

/** `tree` drawn, the packs' files served from the fixtures. */
async function draw(tree: ExpandedRoot): Promise<HTMLElement> {
  root = document.body.appendChild(document.createElement("main"));
  page = createPage(root, { packFile: (place) => `/test/fixtures/fonts/${place.split("/").at(-1) ?? ""}` });
  page.render(tree);
  await page.drawn();
  return root;
}

/** The document's font faces whose family ends with `family`. */
const faces = (family: string): FontFace[] => [...document.fonts].filter((face) => face.family.replace(/"/g, "").endsWith(`-${family}`));

describe("a font of the document's own", () => {
  it("shows a box in it, its fallback after it, once it has loaded", async () => {
    const page = await draw({ type: "root", children: [inFont("serif", "The harbour at dusk.")], fonts: [packFont("serif", "main.woff2")] });
    const box = page.querySelector<HTMLElement>(".compmd-box")!;
    const family = getComputedStyle(box).fontFamily;
    expect(family).toMatch(/^"?compmd-font-\d+-serif"?, system-ui, sans-serif$/);
    const [face] = faces("serif");
    expect(face?.display).toBe("swap");
    await vi.waitFor(() => {
      expect(face?.status).toBe("loaded");
    });
  });

  it("leaves the fallback where its file isn't there", async () => {
    const page = await draw({ type: "root", children: [inFont("gone", "Still readable.")], fonts: [packFont("gone", "missing.woff2")] });
    const [face] = faces("gone");
    await vi.waitFor(() => {
      expect(face?.status).toBe("error");
    });
    expect(getComputedStyle(page.querySelector(".compmd-box")!).fontFamily).toContain("sans-serif");
  });

  it("keeps an example's font of the same name its own", async () => {
    const example: ExpandedNode = {
      type: "code",
      lang: "compmd",
      meta: null,
      value: "",
      render: "output",
      document: { type: "root", example: 1, children: [inFont("serif", "The example's.")], fonts: [{ ...packFont("serif", "main.woff2"), fallback: "mono" }] },
    };
    const page = await draw({ type: "root", children: [inFont("serif", "The page's."), example], fonts: [packFont("serif", "main.woff2")] });
    const [outer, inner] = [...page.querySelectorAll<HTMLElement>(".compmd-box")].map((box) => getComputedStyle(box).fontFamily);
    expect(outer).toContain("sans-serif");
    expect(inner).toContain("monospace");
    expect(outer?.split(",")[0]).not.toBe(inner?.split(",")[0]);
  });

  it("takes its rules away with the page", async () => {
    await draw({ type: "root", children: [inFont("serif", "Gone soon.")], fonts: [packFont("serif", "main.woff2")] });
    expect(faces("serif").length).toBeGreaterThan(0);
    const before = document.adoptedStyleSheets.length;
    page.destroy();
    expect(document.adoptedStyleSheets.length).toBeLessThan(before);
    page = createPage(root);
  });
});
