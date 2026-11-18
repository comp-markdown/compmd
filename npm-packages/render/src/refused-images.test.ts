import { afterEach, describe, expect, it } from "vitest";
import type { ImageSource } from "./images.ts";
import { createPage } from "./page.ts";
import type { ExpandedNode, ExpandedRoot } from "./tree.ts";

const image = (url: string, alt: string): ExpandedNode => ({ type: "image", url, alt });
const tree = (...children: ExpandedNode[]): ExpandedRoot => ({ type: "root", children: [{ type: "paragraph", children }] });

const PHOTOS = "https://photos.example.com";

let root = document.createElement("main");
afterEach(() => {
  root.remove();
});

/** A page whose image source refuses `PHOTOS` until `allow()`, and records the origins the reader asks for. */
function page() {
  root = document.body.appendChild(document.createElement("main"));
  let allowed = false;
  const asked: string[] = [];
  const images: ImageSource = (src) => Promise.resolve(allowed ? `blob:${src}` : { refused: new URL(src).origin });
  const shown = createPage(root, { images, allowOrigin: (origin) => asked.push(origin) });
  const settle = () => new Promise((resolve) => setTimeout(resolve, 0));
  return {
    shown,
    asked,
    settle,
    allow: () => {
      allowed = true;
      shown.imagesAllowed(PHOTOS);
    },
  };
}

describe("an image whose origin the reader hasn't allowed", () => {
  it("shows its alt text with a button that asks for its origin", async () => {
    const { shown, asked, settle } = page();
    shown.render(tree(image(`${PHOTOS}/beds.jpg`, "The beds in May")));
    await settle();
    const refused = root.querySelector(".compmd-image-refused");
    expect(refused?.firstElementChild?.textContent).toBe("The beds in May");
    const button = refused?.querySelector("button");
    expect(button?.textContent).toBe("Show images from photos.example.com");
    button?.click();
    expect(asked).toEqual([PHOTOS]);
    expect(root.querySelector("img")).toBeNull();
  });

  it("shows once its origin is allowed, and keeps its alt text", async () => {
    const { shown, settle, allow } = page();
    shown.render(tree(image(`${PHOTOS}/beds.jpg`, "The beds in May"), image(`${PHOTOS}/shed.jpg`, "The shed")));
    await settle();
    allow();
    await settle();
    expect([...root.querySelectorAll("img")].map((img) => [img.getAttribute("src"), img.getAttribute("alt")])).toEqual([
      [`blob:${PHOTOS}/beds.jpg`, "The beds in May"],
      [`blob:${PHOTOS}/shed.jpg`, "The shed"],
    ]);
    expect(root.querySelector(".compmd-image-refused")).toBeNull();
  });

  it("follows a change of its alt text while refused", async () => {
    const { shown, settle } = page();
    shown.render(tree(image(`${PHOTOS}/beds.jpg`, "Before")));
    await settle();
    shown.render(tree(image(`${PHOTOS}/beds.jpg`, "After")));
    expect(root.querySelector(".compmd-image-refused")?.firstElementChild?.textContent).toBe("After");
    expect(root.querySelector(".compmd-image-refused button")).not.toBeNull();
  });
});
