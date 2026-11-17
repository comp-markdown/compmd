import type { ExpandedNode, Splice } from "@compmd/render";
import { describe, expect, it } from "vitest";
import { hostImages, PageImages } from "./images.ts";

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

  it("gives an image of the host's own schemes from the host's files, fetching nothing", async () => {
    const asked: string[] = [];
    const file = (src: string) => {
      asked.push(src);
      return Promise.resolve(src === "app:file/3" ? new Blob(["x"], { type: "image/png" }) : undefined);
    };
    const requests: string[] = [];
    const fetcher = (input: RequestInfo | URL) => {
      requests.push(input instanceof Request ? input.url : input.toString());
      return Promise.resolve(new Response(new Blob(["x"], { type: "image/png" })));
    };
    const page = new PageImages(fetcher, "https://docs.example.com/", { schemes: ["app"], file });
    page.rendered({ type: "root", children: [image("app:file/3"), image("app:file/4"), image("other:x")] });
    expect((await page.file("app:file/3"))?.type).toBe("image/png");
    expect(await page.file("app:file/4")).toBeUndefined();
    expect(await page.file("other:x")).toBeUndefined();
    expect(asked).toEqual(["app:file/3", "app:file/4"]);
    expect(requests).toEqual([]);
  });

  it("gives no image of the host's own schemes without its files, or when it doesn't name them", async () => {
    const tree = { type: "root" as const, children: [image("app:file/3")] };
    const unloaded = new PageImages(fetch, "https://docs.example.com/", { schemes: ["app"] });
    unloaded.rendered(tree);
    expect(await unloaded.file("app:file/3")).toBeUndefined();
    const unnamed = new PageImages(fetch, "https://docs.example.com/", { schemes: [], file: () => Promise.resolve(new Blob(["x"], { type: "image/png" })) });
    unnamed.rendered(tree);
    expect(await unnamed.file("app:file/3")).toBeUndefined();
  });
});

describe("hostImages", () => {
  it("shows an image of the host's own schemes from a blob: URL of its file, asked for once, and any other from its src", async () => {
    const asked: string[] = [];
    const images = hostImages(["app"], (src) => {
      asked.push(src);
      return Promise.resolve(new Blob(["x"], { type: "image/png" }));
    });
    const first = await images("app:file/3");
    expect(first?.startsWith("blob:")).toBe(true);
    expect(await images("app:file/3")).toBe(first);
    expect(await images("./logo.png")).toBe("./logo.png");
    expect(asked).toEqual(["app:file/3"]);
  });

  it("asks again for an image the host had no file for, or failed to give", async () => {
    let calls = 0;
    const images = hostImages(["app"], () => {
      calls += 1;
      return calls === 1 ? Promise.resolve(undefined) : Promise.reject(new Error("gone"));
    });
    expect(await images("app:file/3")).toBeUndefined();
    expect(await images("app:file/3")).toBeUndefined();
    await images("app:file/3");
    expect(calls).toBe(3);
  });
});
