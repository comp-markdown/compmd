import { describe, expect, it } from "vitest";
import { fitImage, IMAGE_BOX, isImageBox } from "./image-box.ts";
import type { ExpandedNode } from "./tree.ts";

const text = (value: string): ExpandedNode => ({ type: "text", value });
const image: ExpandedNode = { type: "image", url: "harbour.jpg", alt: "Boats", title: null };
const link = (...children: ExpandedNode[]): ExpandedNode => ({ type: "link", url: "harbour.comp.md", title: null, children });
const paragraph = (...children: ExpandedNode[]): ExpandedNode => ({ type: "paragraph", children });
const param = (key: string, value: unknown) => ({ key, breakpoint: null, value });

describe("isImageBox", () => {
  it("is one image in one paragraph, alone or as a link's only content, white space aside", () => {
    expect(isImageBox([paragraph(image)])).toBe(true);
    expect(isImageBox([paragraph(text(" "), image, text("\n"))])).toBe(true);
    expect(isImageBox([paragraph(link(image))])).toBe(true);
    expect(isImageBox([paragraph({ type: "imageReference", identifier: "boats", alt: "Boats" })])).toBe(true);
  });

  it("isn't anything beside it: two images, text, a second paragraph, nothing", () => {
    expect(isImageBox([])).toBe(false);
    expect(isImageBox([paragraph(image, image)])).toBe(false);
    expect(isImageBox([paragraph(image, text(" Boats"))])).toBe(false);
    expect(isImageBox([paragraph(link(image, text("Boats")))])).toBe(false);
    expect(isImageBox([paragraph(image), paragraph(text("A caption"))])).toBe(false);
    expect(isImageBox([{ type: "heading", depth: 2, children: [image] }])).toBe(false);
  });
});

describe("fitImage", () => {
  it("marks an image box with its fit, the inline start the right in a right-to-left box, and unmarks it", () => {
    const box = document.createElement("div");
    box.dir = "rtl";
    fitImage(box, [param("object-fit", "cover"), param("object-position", "end start")], true);
    expect(box.classList.contains(IMAGE_BOX)).toBe(true);
    expect(box.style.getPropertyValue("--compmd-image-fit")).toBe("cover");
    expect(box.style.getPropertyValue("--compmd-image-position")).toBe("100% 100%");
    fitImage(box, [], false);
    expect(box.classList.contains(IMAGE_BOX)).toBe(false);
    expect(box.style.getPropertyValue("--compmd-image-fit")).toBe("");
  });
});
