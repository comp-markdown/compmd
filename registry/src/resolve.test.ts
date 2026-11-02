import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { initSync, OpenDocument, process, type Request } from "@compmd/processor";
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { integrityOf, localRegistry, tarballOf, type LocalRegistry } from "../test/local-registry.ts";
import { memoryCache } from "./cache.ts";
import { registryProxy } from "./proxy.ts";
import { NpmRegistry, type Registry } from "./registry.ts";
import { resolvePacks, type ResolveOptions } from "./resolve.ts";

let local: LocalRegistry;
beforeAll(async () => {
  initSync({ module: readFileSync(fileURLToPath(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
  local = await localRegistry();
});
afterAll(() => local.close());
afterEach(() => {
  local.hits.length = 0;
  vi.unstubAllGlobals();
});

/** A request for a document that imports each of `imports` (`from`, and an `integrity` to pin). */
function request(imports: [from: string, integrity?: string][], body = ""): Request {
  const entries = imports.map(([from, integrity]) => `    - from: "${from}"\n${integrity ? `      integrity: ${integrity}\n` : ""}      names: "*"\n`);
  return { path: "a.comp.md", files: { "a.comp.md": `---\nmd:\n  import:\n${entries.join("")}---\n${body}` }, now: "2026-11-01T09:00:00Z" };
}

const options = (more: Partial<ResolveOptions> = {}): ResolveOptions => ({ registries: { "*": new NpmRegistry({ url: local.url }) }, ...more });

/** The codes and messages of the processor's diagnostics for `request`. */
const diagnostics = (request: Request) => process(request).diagnostics.map(({ code, message }) => `${code} ${message}`);

const tarballHits = () => local.hits.filter((hit) => hit.startsWith("/tarballs/"));

describe("resolvePacks", () => {
  it("resolves the document's packs, then those they import, one level a round", async () => {
    const body = '\n:: section[S] {title="Q3"}\n\n:: chart[C] {title="Installs"}\n';
    const { request: ready, modules, problems } = await resolvePacks(request([["@kit/report@1"]], body), options());
    expect(problems).toEqual([]);
    expect(Object.keys(ready.packs ?? {})).toEqual(["@kit/report", "@kit/charts"]);
    expect(ready.packs?.["@kit/charts"]?.map(({ version }) => version)).toEqual(["1.1.0"]);
    expect(local.hits).toEqual(["/@kit/report", "/tarballs/@kit/report/1.0.0", "/@kit/charts", "/tarballs/@kit/charts/1.1.0"]);
    expect(diagnostics(ready)).toEqual([]);
    const module = modules({ pack: "@kit/charts", version: "1.1.0", path: "dist/chart.js" });
    expect(await module?.code.text()).toContain("(1.1.0)");
    expect(module?.code.type).toBe("text/javascript");
    expect(modules({ pack: "@kit/charts", version: "1.0.0", path: "dist/chart.js" })).toBeUndefined();
  });

  it("fetches nothing again from a cache: tarballs by integrity, versions lists while fresh", async () => {
    const cache = memoryCache();
    await resolvePacks(request([["@kit/report@1"]]), options({ cache }));
    local.hits.length = 0;
    const { request: ready } = await resolvePacks(request([["@kit/report@1"]]), options({ cache }));
    expect(local.hits).toEqual([]);
    expect(ready.packs?.["@kit/charts"]?.[0]?.files["dist/chart.js"]).toContain("(1.1.0)");
  });

  it("serves a pinned import from the cache without asking the registry", async () => {
    const pin = integrityOf(tarballOf("kit-charts-1.0.0.tgz"));
    const cache = memoryCache();
    const first = await resolvePacks(request([["@kit/charts@1.0.0", pin]]), options({ cache }));
    expect(diagnostics(first.request)).toEqual([]);
    local.hits.length = 0;
    const again = await resolvePacks(request([["@kit/charts@1.0.0", pin]]), options({ cache: cache }));
    expect(local.hits).toEqual([]);
    expect(diagnostics(again.request)).toEqual([]);
  });

  it("applies the policy before downloading: a denied version, the size cap, no stated size", async () => {
    const denied = { ...request([["@kit/charts@1"]]), policy: { deny: [{ pack: "@kit/charts", version: "1.1.0", reason: "a flaw" }] } };
    const deny = await resolvePacks(denied, options());
    expect(diagnostics(deny.request)).toEqual([expect.stringMatching(/^F026 `@kit\/charts@1\.1\.0` is denied here: a flaw/)]);
    const capped = await resolvePacks(request([["@kit/huge@1"], ["@kit/unsized@1"]]), options());
    expect(capped.request.policy?.refused).toEqual({
      "@kit/huge": "`@kit/huge@1.0.0` is 10.0 MB unpacked, more than this host's 5 MB",
      "@kit/unsized": "its registry doesn't say how large `@kit/unsized@1.0.0` is, and this host takes packs up to 5 MB",
    });
    expect(diagnostics(capped.request)).toEqual([
      expect.stringMatching(/^F026 `@kit\/huge` isn't available here: `@kit\/huge@1\.0\.0` is 10\.0 MB/),
      expect.stringMatching(/^F026 `@kit\/unsized` isn't available here/),
    ]);
    const larger = await resolvePacks(request([["@kit/huge@1"]]), options({ maxPackSize: 20_000_000 }));
    expect(larger.request.packs?.["@kit/huge"]?.[0]?.files["package.json"]).toBeDefined();
    expect(tarballHits()).toEqual(["/tarballs/@kit/huge/1.0.0"]);
  });

  it("gives F035 for a tarball that doesn't match its integrity, and for a pin that doesn't match the registry's", async () => {
    const { request: broken } = await resolvePacks(request([["@kit/broken@1"]]), options());
    expect(diagnostics(broken)).toEqual([expect.stringMatching(/^F035 `@kit\/broken@1\.0\.0`: its tarball doesn't match the integrity its registry publishes/)]);
    local.hits.length = 0;
    const { request: pinned } = await resolvePacks(request([["@kit/charts@1.0.0", "sha512-other"]]), options());
    expect(diagnostics(pinned)).toEqual([expect.stringMatching(/^F035 `@kit\/charts@1\.0\.0` doesn't match this `integrity`/)]);
    expect(tarballHits()).toEqual([]);
  });

  it("leaves what it can't get to the processor, and says why for the host's logs", async () => {
    const unknown = await resolvePacks(request([["@kit/nothing@1"], ["@kit/charts@^9"]]), options());
    expect(unknown.problems).toEqual([]);
    expect(diagnostics(unknown.request)).toEqual([expect.stringMatching(/^F026 no release of `@kit\/nothing`/), expect.stringMatching(/^F026 no release of `@kit\/charts`/)]);
    const unscoped = await resolvePacks(request([["@other/pack@1"]]), { registries: { "@kit": new NpmRegistry({ url: local.url }) } });
    expect(unscoped.problems).toEqual([{ pack: "@other/pack", message: "no registry is configured for `@other/pack`'s scope" }]);
    local.failing(true);
    const down = await resolvePacks(request([["@kit/charts@1"]]), options());
    local.failing(false);
    expect(down.problems).toEqual([{ pack: "@kit/charts", message: expect.stringContaining("answered 500") as string }]);
  });

  it("uses a stale versions list when the registry can't be reached", async () => {
    let now = 0;
    const cache = memoryCache(() => now);
    await resolvePacks(request([["@kit/charts@1"]]), options({ cache }));
    now = 600_000;
    local.failing(true);
    const offline = await resolvePacks(request([["@kit/charts@1"]]), options({ cache }));
    local.failing(false);
    expect(offline.problems).toEqual([]);
    expect(diagnostics(offline.request)).toEqual([]);
  });

  it("with fetch: host, refuses a registry or a tarball on another origin than the page's", async () => {
    vi.stubGlobal("location", { href: "https://app.example/notes/" });
    await expect(resolvePacks(request([["@kit/charts@1"]]), options())).rejects.toThrow('only with `fetch: "direct"`');
    const elsewhere: Registry = {
      url: "https://app.example/compmd/registry/",
      versions: () => Promise.resolve({ versions: [{ version: "1.0.0", tarball: "https://cdn.example/t.tgz", integrity: "sha512-x", unpackedSize: 10 }], maxAge: 0 }),
      tarball: () => Promise.reject(new Error("not fetched")),
    };
    const { problems } = await resolvePacks(request([["@kit/charts@1"]]), { registries: { "*": elsewhere } });
    expect(problems).toEqual([{ pack: "@kit/charts", message: expect.stringContaining("`https://cdn.example/t.tgz` isn't on this page's origin") as string }]);
    const direct = await resolvePacks(request([["@kit/charts@1"]]), options({ fetch: "direct" }));
    expect(direct.problems).toEqual([]);
  });

  it("resolves through the host's proxy as through the registry", async () => {
    const proxy = registryProxy({ registries: { "*": new NpmRegistry({ url: local.url }) }, base: "/compmd/registry/" });
    const throughProxy: typeof fetch = (url, init) => proxy(new Request(url, init));
    const registries = { "*": new NpmRegistry({ url: "https://notes.example/compmd/registry/", fetch: throughProxy }) };
    const { request: ready, problems } = await resolvePacks(request([["@kit/report@1"]]), { registries });
    expect(problems).toEqual([]);
    expect(Object.keys(ready.packs ?? {})).toEqual(["@kit/report", "@kit/charts"]);
    expect(tarballHits()).toEqual(["/tarballs/@kit/report/1.0.0", "/tarballs/@kit/charts/1.1.0"]);
  });

  describe("library packs", () => {
    /** Each host component's libraries, `name: pack@version/path`. */
    const libraries = (request: Request) =>
      Object.fromEntries(new OpenDocument(request).components.map(({ name, libraries }) => [name, Object.entries(libraries ?? {}).map(([as, { pack, version, path }]) => `${as}: ${pack}@${version}/${path}`)]));

    it("fetches the version every range asked in a round matches, once", async () => {
      const { request: ready, modules, problems } = await resolvePacks(request([["@kit/bars@1"], ["@kit/lines@1"]]), options());
      expect(problems).toEqual([]);
      expect(tarballHits()).toEqual(["/tarballs/@kit/bars/1.0.0", "/tarballs/@kit/lines/1.0.0", "/tarballs/@kit/d3/7.9.5"]);
      expect(diagnostics(ready)).toEqual([]);
      expect(libraries(ready)).toEqual({ bars: ["@kit/d3: @kit/d3@7.9.5/dist/d3.js"], lines: ["@kit/d3: @kit/d3@7.9.5/dist/d3.js"] });
      expect(await modules({ pack: "@kit/d3", version: "7.9.5", path: "dist/d3.js" })?.code.text()).toContain('"7.9.5"');
    });

    it("fetches again for a range asked in a later round, the version every range matches", async () => {
      const { request: ready } = await resolvePacks(request([["@kit/bars@1"], ["@kit/dashboard@1"]]), options());
      expect(tarballHits()).toEqual(["/tarballs/@kit/bars/1.0.0", "/tarballs/@kit/dashboard/1.0.0", "/tarballs/@kit/lines/1.0.0", "/tarballs/@kit/d3/7.12.0", "/tarballs/@kit/d3/7.9.5"]);
      expect(diagnostics(ready)).toEqual([]);
      expect(libraries(ready)).toEqual({ bars: ["@kit/d3: @kit/d3@7.9.5/dist/d3.js"], lines: ["@kit/d3: @kit/d3@7.9.5/dist/d3.js"] });
    });

    it("leaves a library no version of which every range matches to the processor", async () => {
      const policy = { deny: [{ pack: "@kit/d3", version: "7.9.5", reason: "a flaw" }] };
      const { request: ready } = await resolvePacks({ ...request([["@kit/bars@1"], ["@kit/lines@1"]]), policy }, options());
      expect(diagnostics(ready)).toEqual([expect.stringMatching(/^W121 `@kit\/bars@1\.0\.0` needs `@kit\/d3` `\^7\.9`, but `@kit\/d3@7\.9\.5` is denied here: a flaw/), expect.stringMatching(/^W121 `@kit\/lines@1\.0\.0` needs `@kit\/d3` `~7\.9\.0`, but/)]);
    });
  });

  it("keeps the request's own packs and refusals", async () => {
    const given = { ...request([["@kit/report@1"]]), policy: { refused: { "@kit/charts": "kept out" } } };
    const { request: ready } = await resolvePacks(given, options());
    expect(ready.policy?.refused).toEqual({ "@kit/charts": "kept out" });
    expect(Object.keys(ready.packs ?? {})).toEqual(["@kit/report"]);
    expect(given.packs).toBeUndefined();
  });
});
