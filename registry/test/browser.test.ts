// Resolving in a browser, as a static site with `fetch: "direct"` does: the tarballs gunzipped by the browser's
// DecompressionStream, checked with crypto.subtle, and kept in Cache Storage across visits.

import { init, process, type Request } from "@compmd/processor";
import { beforeAll, describe, expect, inject, it } from "vitest";
import { defaultCache, NpmRegistry, resolvePacks } from "../src/index.ts";

const registry = inject("registry");

beforeAll(async () => {
  await init();
  await caches.delete("compmd-registry-v1");
});

const request: Request = {
  path: "a.comp.md",
  files: { "a.comp.md": '---\nmd:\n  import:\n    - from: "@kit/report@1"\n      names: "*"\n---\n\n:: chart[C] {title="Installs"}\n' },
  now: "2026-11-01T09:00:00Z",
};

describe("resolvePacks in a browser", () => {
  it("fetches, checks and unpacks, and a later visit's cache asks the registry nothing", async () => {
    const asked: string[] = [];
    const counting: typeof fetch = (url, init) => {
      asked.push(url instanceof Request ? url.url : url.toString());
      return fetch(url, init);
    };
    const registries = { "*": new NpmRegistry({ url: registry, fetch: counting }) };
    const { request: ready, modules } = await resolvePacks(request, { registries, fetch: "direct", cache: defaultCache() });
    expect(process(ready).diagnostics).toEqual([]);
    expect(await modules({ pack: "@kit/charts", version: "1.1.0", path: "dist/chart.js" })?.code.text()).toContain("(1.1.0)");
    expect(asked).toHaveLength(4);
    // A new cache, as on a later visit: what Cache Storage kept, memory doesn't have.
    const again = await resolvePacks(request, { registries, fetch: "direct", cache: defaultCache() });
    expect(process(again.request).diagnostics).toEqual([]);
    expect(asked).toHaveLength(4);
  });

  it("keeps versions lists in Cache Storage: a later visit reads one back, stale once its max-age has passed", async () => {
    let now = 0;
    const first = defaultCache(() => now);
    await first.keepVersions("https://npm.example/@kit/charts", [{ version: "1.0.0", tarball: "https://npm.example/t" }], 300);
    now = 301_000;
    expect(await defaultCache(() => now).versions("https://npm.example/@kit/charts")).toEqual({ versions: [{ version: "1.0.0", tarball: "https://npm.example/t" }], fresh: false });
  });

  it("with fetch: host, refuses the registry on another origin", async () => {
    await expect(resolvePacks(request, { registries: { "*": new NpmRegistry({ url: registry }) } })).rejects.toThrow('`fetch: "direct"`');
  });
});
