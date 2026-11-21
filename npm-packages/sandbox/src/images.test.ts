import type { ExpandedNode, Refused, Splice } from "@compmd/render";
import { describe, expect, it } from "vitest";
import { ImageGate } from "./image-gate.ts";
import { hostImages, PageImages } from "./images.ts";
import { localAnswers } from "./origins.ts";

const image = (url: string): ExpandedNode => ({ type: "paragraph", children: [{ type: "image", url, alt: "" }] });
const splice = (...nodes: ExpandedNode[]): Splice => ({ from: 0, to: 0, nodes, lines: 0, offset: 0 });

/** A file's type, when it is a file. */
const typeOf = (file: Blob | Refused | undefined): string | undefined => (file instanceof Blob ? file.type : undefined);

/** A gate that lets the CDN's images through, as a host that configured it does, and asks about no other. */
const cdnGate = () => new ImageGate({ configured: ["https://cdn.example.com"], ask: () => Promise.resolve("deny"), answers: localAnswers(undefined) });

/** Page images over a fetch that answers every URL with `type`, recording each request. */
function images(type = "image/png", status = 200) {
  const requests: { url: string; init: RequestInit | undefined }[] = [];
  const fetcher = (input: RequestInfo | URL, init?: RequestInit) => {
    requests.push({ url: input instanceof Request ? input.url : input.toString(), init });
    return Promise.resolve(new Response(new Blob(["x"], { type }), { status }));
  };
  return { images: new PageImages(fetcher, "https://docs.example.com/notes/", { schemes: [] }, cdnGate()), requests };
}

describe("PageImages", () => {
  it("fetches an image of the tree it was given, a relative one against its base, without credentials or a Referer", async () => {
    const { images: page, requests } = images();
    page.rendered({ type: "root", children: [image("./logo.png"), image("https://cdn.example.com/a.png")] });
    expect(typeOf(await page.file("./logo.png"))).toBe("image/png");
    expect(typeOf(await page.file("https://cdn.example.com/a.png"))).toBe("image/png");
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

  it("fetches a pack's image by the URL the frame asks for, where the host serves the pack, and none by its place", async () => {
    const requests: string[] = [];
    const fetcher = (input: RequestInfo | URL) => {
      requests.push(input instanceof Request ? input.url : input.toString());
      return Promise.resolve(new Response(new Blob(["x"], { type: "image/svg+xml" })));
    };
    const place = "@acme/kit@1.4.2/icons/star.svg";
    const star: ExpandedNode = { type: "paragraph", children: [{ type: "image", url: place, alt: "", pack: true }] };
    const page = new PageImages(fetcher, "https://docs.example.com/notes/", { schemes: [] }, cdnGate(), { at: "/compmd/packs/" });
    page.rendered({ type: "root", children: [star] });
    expect(typeOf(await page.file(`/compmd/packs/${place}`))).toBe("image/svg+xml");
    expect(await page.file(place)).toBeUndefined();
    expect(requests).toEqual([`https://docs.example.com/compmd/packs/${place}`]);
    const unmapped = new PageImages(fetcher, "https://docs.example.com/", { schemes: [] }, cdnGate());
    unmapped.rendered({ type: "root", children: [star] });
    expect(await unmapped.file(place)).toBeUndefined();
  });

  it("gives a registry pack's image the host holds, fetching nothing, and fetches a bundled pack's", async () => {
    const requests: string[] = [];
    const fetcher = (input: RequestInfo | URL) => {
      requests.push(input instanceof Request ? input.url : input.toString());
      return Promise.resolve(new Response(new Blob(["x"], { type: "image/png" })));
    };
    const held = new Blob(["<svg/>"], { type: "image/svg+xml" });
    const fetched = "@acme/kit@1.4.2/icons/star.svg";
    const registry = "@acme/maps@2.0.0/pin.svg";
    const images = (place: string) => (place === registry ? held : undefined);
    const tree = { type: "root" as const, children: [registry, fetched].map((url): ExpandedNode => ({ type: "paragraph", children: [{ type: "image", url, alt: "", pack: true }] })) };
    const page = new PageImages(fetcher, "https://docs.example.com/", { schemes: [] }, cdnGate(), { at: "/packs/", images });
    page.rendered(tree);
    expect(await page.file(`/packs/${registry}`)).toBe(held);
    expect(typeOf(await page.file(`/packs/${fetched}`))).toBe("image/png");
    expect(requests).toEqual([`https://docs.example.com/packs/${fetched}`]);
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
    expect(typeOf(await page.file("app:file/3"))).toBe("image/png");
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
    const images = hostImages(
      ["app"],
      (src) => {
        asked.push(src);
        return Promise.resolve(new Blob(["x"], { type: "image/png" }));
      },
      cdnGate(),
    );
    const first = await images("app:file/3");
    expect(typeof first === "string" && first.startsWith("blob:")).toBe(true);
    expect(await images("app:file/3")).toBe(first);
    expect(await images("./logo.png")).toBe("./logo.png");
    expect(asked).toEqual(["app:file/3"]);
  });

  it("shows a registry pack's image the host holds from a blob: URL of it, made once, and a bundled pack's from its src", async () => {
    const registry = "@acme/maps@2.0.0/pin.svg";
    const notAnImage = "@acme/maps@2.0.0/guide.html";
    const held: Record<string, Blob> = { [registry]: new Blob(["<svg/>"], { type: "image/svg+xml" }), [notAnImage]: new Blob(["<p>"], { type: "text/html" }) };
    const images = hostImages([], undefined, cdnGate(), { at: "/packs/", images: (place) => held[place] });
    const first = await images(`/packs/${registry}`);
    expect(typeof first === "string" && first.startsWith("blob:")).toBe(true);
    expect(await images(`/packs/${registry}`)).toBe(first);
    expect(await images("/packs/@acme/kit@1.4.2/star.svg")).toBe("/packs/@acme/kit@1.4.2/star.svg");
    // Only an image is given from a blob: URL, which would give a page the host's page's origin.
    expect(await images(`/packs/${notAnImage}`)).toBe(`/packs/${notAnImage}`);
  });

  it("asks again for an image the host had no file for, or failed to give", async () => {
    let calls = 0;
    const images = hostImages(
      ["app"],
      () => {
        calls += 1;
        return calls === 1 ? Promise.resolve(undefined) : Promise.reject(new Error("gone"));
      },
      cdnGate(),
    );
    expect(await images("app:file/3")).toBeUndefined();
    expect(await images("app:file/3")).toBeUndefined();
    await images("app:file/3");
    expect(calls).toBe(3);
  });

  it("shows a remote image once its origin is configured or allowed, and refuses it otherwise", async () => {
    const images = hostImages([], undefined, cdnGate());
    expect(await images("https://cdn.example.com/a.png")).toBe("https://cdn.example.com/a.png");
    expect(await images("https://photos.example.com/b.png")).toEqual({ refused: "https://photos.example.com" });
    expect(await images("app:file/3")).toBe("app:file/3");
  });

  it("shows every remote image, asking about none, when the host allows every origin (spec §15)", async () => {
    let asked = 0;
    const gate = new ImageGate({
      configured: "any",
      ask: () => {
        asked += 1;
        return Promise.resolve("deny");
      },
      answers: localAnswers(undefined),
    });
    const images = hostImages([], undefined, gate);
    expect(await images("https://photos.example.com/b.png")).toBe("https://photos.example.com/b.png");
    expect(await gate.allow("https://photos.example.com")).toBe(true);
    expect(asked).toBe(0);
  });
});

describe("remote images in a page frame (spec §15)", () => {
  const PHOTOS = "https://photos.example.com";

  /** Page images whose gate answers with `answers`, in order, recording what it asked about. */
  function asking(...answers: ("allow" | "deny")[]) {
    const asked: { origin: string; kinds: readonly string[] }[] = [];
    const gate = new ImageGate({
      ask: (origin, _document, kinds) => {
        asked.push({ origin, kinds });
        return Promise.resolve(answers.shift() ?? "deny");
      },
      answers: localAnswers(undefined),
    });
    const fetcher = () => Promise.resolve(new Response(new Blob(["x"], { type: "image/png" })));
    const page = new PageImages(fetcher, "https://docs.example.com/", { schemes: [] }, gate);
    page.rendered({ type: "root", children: [image(`${PHOTOS}/a.png`), image(`${PHOTOS}/b.png`)] });
    return { page, gate, asked };
  }

  it("asks about an image's origin the first time, once, and refuses it when the reader does", async () => {
    const { page, asked } = asking("deny");
    expect(await page.file(`${PHOTOS}/a.png`)).toEqual({ refused: PHOTOS });
    expect(await page.file(`${PHOTOS}/b.png`)).toEqual({ refused: PHOTOS });
    expect(asked).toEqual([{ origin: PHOTOS, kinds: ["images"] }]);
  });

  it("fetches it once the reader allows its origin, from the button, and tells who listens", async () => {
    const { page, gate, asked } = asking("deny", "allow");
    expect(await page.file(`${PHOTOS}/a.png`)).toEqual({ refused: PHOTOS });
    const allowed: string[] = [];
    gate.onAllowed((origin) => allowed.push(origin));
    await page.allowOrigin(PHOTOS);
    expect(allowed).toEqual([PHOTOS]);
    expect(typeOf(await page.file(`${PHOTOS}/a.png`))).toBe("image/png");
    expect(asked).toHaveLength(2);
  });

  it("asks only about an origin of the frame's images, and not again within a minute of a refusal", async () => {
    const { page, asked } = asking("deny", "deny", "allow");
    await page.allowOrigin("https://tracker.example.com");
    expect(asked).toEqual([]);
    let now = 1000;
    await page.allowOrigin(PHOTOS, () => now);
    await page.allowOrigin(PHOTOS, () => now + 30_000);
    expect(asked).toHaveLength(1);
    now += 61_000;
    await page.allowOrigin(PHOTOS, () => now);
    expect(asked).toHaveLength(2);
  });
});
