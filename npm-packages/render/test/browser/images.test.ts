// What only a browser's layout says about an image box (spec §9.2): its image takes the box's inside, so the box's
// height crops it, covering or contained as `object-fit` says and placed as `object-position` says; without a height,
// the image is as tall as it is at the box's width. A 2:1 drawing throughout.
import { afterEach, describe, expect, it, vi } from "vitest";
import { createPage } from "../../src/page.ts";
import type { ExpandedNode } from "../../src/tree.ts";

let root: HTMLElement;
afterEach(() => {
  root.remove();
});

const WIDE = `data:image/svg+xml,${encodeURIComponent('<svg xmlns="http://www.w3.org/2000/svg" width="400" height="200"><rect width="400" height="200" fill="teal"/></svg>')}`;

const param = (key: string, value: unknown) => ({ key, breakpoint: null, value });
const image: ExpandedNode = { type: "image", url: "harbour.svg", alt: "Boats at the quay", title: null };

/** A box `width` wide with `params`, holding `inlines` in one paragraph; its image once it has loaded. */
async function draw(width: string, params: { key: string; breakpoint: null; value: unknown }[], inlines: ExpandedNode[] = [image]) {
  root = document.body.appendChild(document.createElement("main"));
  root.style.cssText = "width: 600px";
  const box: ExpandedNode = { type: "componentContainer", name: "box", params: [param("width", width), ...params], children: [{ type: "paragraph", children: inlines }] };
  createPage(root, { images: () => Promise.resolve(WIDE) }).render({ type: "root", children: [box] });
  const img = await vi.waitFor(() => {
    const found = root.querySelector("img");
    if (!found?.complete) throw new Error("not loaded yet");
    return found;
  });
  return { box: root.querySelector<HTMLElement>(".compmd-box")!, img };
}

const size = (element: Element) => {
  const { width, height } = element.getBoundingClientRect();
  return [Math.round(width), Math.round(height)];
};

describe("an image box", () => {
  it("crops its image to a square, covering it", async () => {
    const { box, img } = await draw("200px", [param("aspect-ratio", "1"), param("object-fit", "cover")]);
    expect(box.classList.contains("compmd-image-box")).toBe(true);
    expect(size(img)).toEqual([200, 200]);
    expect(getComputedStyle(img).objectFit).toBe("cover");
  });

  it("crops its image to 16:9, inside the box's padding and border", async () => {
    const { img } = await draw("340px", [param("aspect-ratio", "16/9"), param("object-fit", "cover"), param("padding", "10px")]);
    const [width = 0, height = 0] = size(img);
    expect(width).toBe(320);
    expect(Math.abs(height - (340 * 9) / 16 + 20)).toBeLessThanOrEqual(1);
  });

  it("letterboxes its image with contain, the default", async () => {
    const { img } = await draw("200px", [param("aspect-ratio", "1")]);
    expect(size(img)).toEqual([200, 200]);
    expect(getComputedStyle(img).objectFit).toBe("contain");
  });

  it("places its image by object-position, the block position first", async () => {
    const { img } = await draw("200px", [param("aspect-ratio", "1"), param("object-fit", "cover"), param("object-position", "start end")]);
    expect(getComputedStyle(img).objectPosition).toBe("100% 0%");
  });

  it("is as tall as its image at its width without a height, nothing cropped", async () => {
    const { img } = await draw("300px", [param("object-fit", "cover")]);
    expect(size(img)).toEqual([300, 150]);
  });

  it("crops a linked image the same, the link around it", async () => {
    const { img } = await draw("200px", [param("aspect-ratio", "1"), param("object-fit", "cover")], [{ type: "link", url: "harbour.comp.md", title: null, children: [image] }]);
    expect(size(img)).toEqual([200, 200]);
    expect(img.closest("a")?.getAttribute("href")).toBe("harbour.comp.md");
  });

  it("isn't one with a caption beside its image, which keeps its own size", async () => {
    const { box, img } = await draw("200px", [param("aspect-ratio", "1"), param("object-fit", "cover")], [image, { type: "text", value: " Boats at the quay." }]);
    expect(box.classList.contains("compmd-image-box")).toBe(false);
    expect(getComputedStyle(img).objectFit).toBe("fill");
  });
});
