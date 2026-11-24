// The caches, as the resolver uses them: one scenario a test. Cache Storage itself is in the browser tests
// (test/browser.test.ts); here, where Node has none, and where a page's refuses.

import { afterEach, describe, expect, it, vi } from "vitest";
import { defaultCache, memoryCache } from "./cache.ts";

const LIST = [{ version: "1.0.0", tarball: "https://npm.example/t" }];

/** A release of `@kit/charts`, by its integrity. */
const charts = (integrity: string) => ({ pack: "@kit/charts", version: "1.0.0", integrity });

/** Cache Storage in memory, as a page has it, kept across the caches made while it is stubbed. */
function stubStorage(): void {
  const stored = new Map<string, string>();
  const storage = {
    match: (url: string) => Promise.resolve(stored.has(url) ? new Response(stored.get(url)) : undefined),
    put: async (url: string, response: Response) => {
      stored.set(url, await response.text());
    },
  };
  vi.stubGlobal("caches", { open: () => Promise.resolve(storage) });
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("memoryCache", () => {
  it("keeps a release's files by its pack, version and integrity", async () => {
    const cache = memoryCache();
    expect(await cache.files(charts("sha512-a"))).toBeUndefined();
    await cache.keep(charts("sha512-a"), { "package.json": "{}" });
    expect(await cache.files(charts("sha512-a"))).toEqual({ "package.json": "{}" });
    expect(await cache.files(charts("sha512-b")), "another integrity").toBeUndefined();
    expect(await cache.files({ ...charts("sha512-a"), pack: "@evil/pack" }), "another pack, the same integrity").toBeUndefined();
    expect(await cache.files({ ...charts("sha512-a"), version: "2.0.0" }), "another version, the same integrity").toBeUndefined();
  });

  it("keeps a release's binary files by its pack, version and integrity, beside its text", async () => {
    const cache = memoryCache();
    expect(await cache.binaryFiles?.(charts("sha512-a"))).toBeUndefined();
    await cache.keepBinary?.(charts("sha512-a"), { "logo.png": new Uint8Array([0x89, 0x50]) });
    expect(await cache.binaryFiles?.(charts("sha512-a"))).toEqual({ "logo.png": new Uint8Array([0x89, 0x50]) });
    expect(await cache.binaryFiles?.({ ...charts("sha512-a"), pack: "@evil/pack" })).toBeUndefined();
  });

  it("keeps a versions list fresh for its max-age, then stale, by its key", async () => {
    let now = 0;
    const cache = memoryCache(() => now);
    expect(await cache.versions("https://npm.example/@kit/charts")).toBeUndefined();
    await cache.keepVersions("https://npm.example/@kit/charts", LIST, 300);
    expect(await cache.versions("https://npm.example/@kit/charts")).toEqual({ versions: LIST, fresh: true });
    now = 300_000;
    expect(await cache.versions("https://npm.example/@kit/charts")).toEqual({ versions: LIST, fresh: false });
    expect(await cache.versions("https://mirror.example/@kit/charts"), "another registry's").toBeUndefined();
  });

  it("keeps a list with no max-age (0) stale at once: used only when the registry can't be reached", async () => {
    const cache = memoryCache(() => 0);
    await cache.keepVersions("k", LIST, 0);
    expect(await cache.versions("k")).toEqual({ versions: LIST, fresh: false });
  });
});

describe("defaultCache", () => {
  it("is memory only where there is no Cache Storage, as in Node: each cache its own", async () => {
    let now = 0;
    const cache = defaultCache(() => now);
    await cache.keep(charts("sha512-a"), { "a.json": "[]" });
    expect(await cache.files(charts("sha512-a"))).toEqual({ "a.json": "[]" });
    expect(await defaultCache().files(charts("sha512-a")), "a second cache shares nothing").toBeUndefined();
    await cache.keepVersions("k", LIST, 1);
    now = 1000;
    expect(await cache.versions("k")).toEqual({ versions: LIST, fresh: false });
  });

  it("is memory only where Cache Storage refuses to open, as in a private window or an opaque origin", async () => {
    vi.stubGlobal("caches", { open: () => Promise.reject(new DOMException("refused", "SecurityError")) });
    const cache = defaultCache();
    await cache.keep(charts("sha512-a"), { "a.json": "[]" });
    expect(await cache.files(charts("sha512-a"))).toEqual({ "a.json": "[]" });
  });

  it("keeps a version's binary files in Cache Storage, so a cache made later, as on another visit, has them", async () => {
    stubStorage();
    const bytes = new Uint8Array([0x89, 0x50, 0x00, 0xff, 0xfe]);
    await defaultCache().keepBinary?.(charts("sha512-a"), { "icons/logo.png": bytes });
    expect(await defaultCache().binaryFiles?.(charts("sha512-a"))).toEqual({ "icons/logo.png": bytes });
    expect(await defaultCache().binaryFiles?.(charts("sha512-b"))).toBeUndefined();
  });

  it("keeps a release's files in Cache Storage by its pack, version and integrity, so a later visit finds only its own", async () => {
    stubStorage();
    await defaultCache().keep(charts("sha512-a"), { "a.json": "[]" });
    expect(await defaultCache().files(charts("sha512-a"))).toEqual({ "a.json": "[]" });
    expect(await defaultCache().files({ ...charts("sha512-a"), pack: "@evil/pack" })).toBeUndefined();
  });

  it("keeps in memory what Cache Storage fails to keep or read, as when it is full", async () => {
    const failing = { match: () => Promise.reject(new Error("broken")), put: () => Promise.reject(new DOMException("full", "QuotaExceededError")) };
    vi.stubGlobal("caches", { open: () => Promise.resolve(failing) });
    const cache = defaultCache();
    expect(await cache.files(charts("sha512-a")), "a read that fails is a miss").toBeUndefined();
    await cache.keep(charts("sha512-a"), { "a.json": "[]" });
    await cache.keepVersions("k", LIST, 300);
    expect(await cache.files(charts("sha512-a"))).toEqual({ "a.json": "[]" });
    expect(await cache.versions("k")).toEqual({ versions: LIST, fresh: true });
  });
});
