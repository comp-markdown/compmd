import { afterEach, describe, expect, it } from "vitest";
import { imageSources, type ImageSource } from "./images.ts";
import { createPage, type Page } from "./page.ts";
import type { ExpandedNode, ExpandedRoot } from "./tree.ts";

const text = (value: string): ExpandedNode => ({ type: "text", value });
const paragraph = (...children: ExpandedNode[]): ExpandedNode => ({ type: "paragraph", children });
const image = (url: string, alt: string, title?: string): ExpandedNode => ({ type: "image", url, alt, ...(title ? { title } : {}) });
const tree = (...children: ExpandedNode[]): ExpandedRoot => ({ type: "root", children });

/** An image source the test answers by hand: each `src` asked for, and a way to answer it. */
function asked() {
  const waiting = new Map<string, { resolve(shown: string | undefined): void; reject(error: unknown): void }>();
  const source: ImageSource = (src) =>
    new Promise((resolve, reject) => {
      waiting.set(src, { resolve, reject });
    });
  const answer = async (src: string, shown: string | undefined) => {
    waiting.get(src)?.resolve(shown);
    await Promise.resolve();
  };
  const fail = async (src: string) => {
    waiting.get(src)?.reject(new Error("unreachable"));
    await Promise.resolve();
  };
  return { source, asked: () => [...waiting.keys()], answer, fail };
}

let root = document.createElement("main");
function page(images: ImageSource): Page {
  root = document.body.appendChild(document.createElement("main"));
  return createPage(root, { images });
}
afterEach(() => {
  root.remove();
});

describe("imageSources", () => {
  it("lists Markdown images, link reference definitions and raw HTML images, in slots and fallbacks too, but no unsafe URL", () => {
    const nodes: ExpandedNode[] = [
      paragraph(image("./logo.png", "Logo"), image("javascript:alert(1)", "No")),
      { type: "definition", identifier: "photo", url: "https://example.com/photo.png" },
      { type: "html", value: '<img src="https://example.com/raw.png" alt="Raw">' },
      {
        type: "componentContainer",
        name: "card",
        params: [],
        children: [paragraph(image("in-slot.png", "In a slot"))],
        slots: [{ type: "componentSlot", name: "footer", children: [paragraph(image("footer.png", "Footer"))] }],
      },
      { type: "componentLeaf", name: "chart", params: [], fallback: [image("fallback.png", "Fallback")] },
    ];
    expect(imageSources(nodes)).toEqual(["./logo.png", "https://example.com/photo.png", "https://example.com/raw.png", "in-slot.png", "footer.png", "fallback.png"]);
  });
});

describe("images from a source", () => {
  it("shows the alt text until the source answers, then the image it gives, with its alt and title", async () => {
    const images = asked();
    page(images.source).render(tree(paragraph(text("See "), image("https://example.com/a.png", "A photo", "Taken in May"))));
    expect(images.asked()).toEqual(["https://example.com/a.png"]);
    const waiting = root.querySelector("p span.compmd-image-alt");
    expect(waiting?.textContent).toBe("A photo");
    expect(waiting?.getAttribute("title")).toBe("Taken in May");
    expect(root.querySelector("img")).toBeNull();
    await images.answer("https://example.com/a.png", "blob:shown");
    const shown = root.querySelector("p img");
    expect(shown?.getAttribute("src")).toBe("blob:shown");
    expect(shown?.getAttribute("alt")).toBe("A photo");
    expect(shown?.getAttribute("title")).toBe("Taken in May");
    expect(root.querySelector(".compmd-image-alt")).toBeNull();
  });

  it("keeps the alt text when the image can't be had, or its source fails", async () => {
    const images = asked();
    page(images.source).render(tree(paragraph(image("missing.png", "Missing"), image("broken.png", "Broken"))));
    await images.answer("missing.png", undefined);
    await images.fail("broken.png");
    expect([...root.querySelectorAll(".compmd-image-alt")].map((alt) => alt.textContent)).toEqual(["Missing", "Broken"]);
    expect(root.querySelector("img")).toBeNull();
  });

  it("asks for an image by reference with its definition's URL, and for one in raw HTML with its src", async () => {
    const images = asked();
    page(images.source).render(
      tree(
        paragraph({ type: "imageReference", identifier: "photo", alt: "By reference" }, { type: "html", value: '<img src="raw.png" alt="Raw">' }),
        { type: "definition", identifier: "photo", url: "photo.png" },
      ),
    );
    expect(images.asked()).toEqual(["photo.png", "raw.png"]);
    await images.answer("raw.png", "blob:raw");
    expect(root.querySelector("img")?.getAttribute("alt")).toBe("Raw");
    expect(root.querySelector(".compmd-image-alt")?.textContent).toBe("By reference");
  });

  it("shows the image of the src it has now, not an older one's answer", async () => {
    const images = asked();
    const p = page(images.source);
    p.render(tree(paragraph(image("old.png", "Picture"))));
    p.render(tree(paragraph(image("new.png", "Picture"))));
    await images.answer("old.png", "blob:old");
    expect(root.querySelector("img")).toBeNull();
    await images.answer("new.png", "blob:new");
    expect(root.querySelector("img")?.getAttribute("src")).toBe("blob:new");
  });

  it("updates a shown image's alt text in place", async () => {
    const images = asked();
    const p = page(images.source);
    p.render(tree(paragraph(image("a.png", "Before"))));
    await images.answer("a.png", "blob:a");
    const shown = root.querySelector("img");
    p.render(tree(paragraph(image("a.png", "After"))));
    expect(root.querySelector("img")).toBe(shown);
    expect(shown?.getAttribute("alt")).toBe("After");
  });
});
