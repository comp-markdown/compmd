import type { ExpandedNode, Splice } from "@compmd/render";
import { describe, expect, it } from "vitest";
import { PageImages } from "./images.ts";

const image = (url: string): ExpandedNode => ({ type: "paragraph", children: [{ type: "image", url, alt: "" }] });
const splice = (...nodes: ExpandedNode[]): Splice => ({ from: 0, to: 0, nodes, lines: 0, offset: 0 });

/** Page images over a fetch that answers every URL with `type`, recording each request. */
function images(type = "image/png", status = 200) {
  const requests: { url: string; init: RequestInit | undefined }[] = [];
  const fetcher = (input: RequestInfo | URL, init?: RequestInit) => {
    requests.push({ url: input instanceof Request ? input.url : input.toString(), init });
    return Promise.resolve(new Response(new Blob(["x"], { type }), { status }));
  };
  return { images: new PageImages(fetcher, "https://docs.example.com/notes/"), requests };
}

describe("PageImages", () => {
  it("fetches an image of the tree it was given, a relative one against its base, without credentials or a Referer", async () => {
    const { images: page, requests } = images();
    page.rendered({ type: "root", children: [image("./logo.png"), image("https://cdn.example.com/a.png")] });
    expect((await page.file("./logo.png"))?.type).toBe("image/png");
    expect((await page.file("https://cdn.example.com/a.png"))?.type).toBe("image/png");
    expect(requests).toEqual([
      { url: "https://docs.example.com/notes/logo.png", init: { credentials: "omit", referrerPolicy: "no-referrer" } },
      { url: "https://cdn.example.com/a.png", init: { credentials: "omit", referrerPolicy: "no-referrer" } },
    ]);
  });

  it("gives no image the trees it was given don't have, and fetches nothing for it", async () => {
    const { images: page, requests } = images();
    page.rendered({ type: "root", children: [image("./logo.png")] });
    expect(await page.file("https://tracker.example.com/?secret=1")).toBeUndefined();
    expect(requests).toEqual([]);
  });

  it("forgets the images of the last tree when it is given a new one", async () => {
    const { images: page } = images();
    page.rendered({ type: "root", children: [image("./old.png")] });
    page.rendered({ type: "root", children: [image("./new.png")] });
    expect(await page.file("./old.png")).toBeUndefined();
    expect(await page.file("./new.png")).toBeDefined();
  });

  it("adds the images of splices, as objects or as JSON text", async () => {
    const { images: page } = images();
    page.rendered({ type: "root", children: [] });
    page.spliced([splice(image("./object.png")), JSON.stringify(splice(image("./text.png")))]);
    expect(await page.file("./object.png")).toBeDefined();
    expect(await page.file("./text.png")).toBeDefined();
  });

  it("gives nothing for a file that isn't an image, a failed answer, or a fetch that throws", async () => {
    const tree = { type: "root" as const, children: [image("./a.png")] };
    const text = images("text/html");
    text.images.rendered(tree);
    expect(await text.images.file("./a.png")).toBeUndefined();
    const missing = images("image/png", 404);
    missing.images.rendered(tree);
    expect(await missing.images.file("./a.png")).toBeUndefined();
    const throwing = new PageImages(() => Promise.reject(new TypeError("offline")), "https://docs.example.com/");
    throwing.rendered(tree);
    expect(await throwing.file("./a.png")).toBeUndefined();
  });
});
