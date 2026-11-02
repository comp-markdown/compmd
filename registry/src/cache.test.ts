import { describe, expect, it } from "vitest";
import { defaultCache, memoryCache } from "./cache.ts";

const LIST = [{ version: "1.0.0", tarball: "https://npm.example/t" }];

describe("the cache", () => {
  it("keeps files by integrity, and versions lists fresh for their max-age, then stale", async () => {
    let now = 0;
    const cache = memoryCache(() => now);
    expect(await cache.files("sha512-a")).toBeUndefined();
    await cache.keep("sha512-a", { "package.json": "{}" });
    expect(await cache.files("sha512-a")).toEqual({ "package.json": "{}" });
    expect(await cache.versions("k")).toBeUndefined();
    await cache.keepVersions("k", LIST, 300);
    expect(await cache.versions("k")).toEqual({ versions: LIST, fresh: true });
    now = 300_000;
    expect(await cache.versions("k")).toEqual({ versions: LIST, fresh: false });
  });

  it("is memory only where there is no Cache Storage, as in Node", async () => {
    let now = 0;
    const cache = defaultCache(() => now);
    await cache.keep("sha512-a", { "a.json": "[]" });
    expect(await cache.files("sha512-a")).toEqual({ "a.json": "[]" });
    expect(await defaultCache().files("sha512-a")).toBeUndefined();
    await cache.keepVersions("k", LIST, 1);
    now = 1000;
    expect(await cache.versions("k")).toEqual({ versions: LIST, fresh: false });
    expect(await cache.versions("other")).toBeUndefined();
  });
});
