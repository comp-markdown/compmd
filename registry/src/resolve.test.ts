// `resolvePacks`, as a host uses it: one scenario a test, each a good use (what a host does and gets) or a bad one
// (what goes wrong, and what the host and the document are told). The packs are the local registry's
// (test/local-registry.ts, CATALOG, which says what each is).

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { initSync, OpenDocument, process, type Request } from "@compmd/processor";
import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { integrityOf, localRegistry, tarballOf, type LocalRegistry } from "../test/local-registry.ts";
import { memoryCache, type PackCache } from "./cache.ts";
import { registryProxy } from "./proxy.ts";
import { NpmRegistry, type Registry, type VersionInfo } from "./registry.ts";
import { DEFAULT_MAX_PACK_SIZE, resolvePacks, type ResolveOptions } from "./resolve.ts";

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

/**
 * A request for a document that imports each of `froms` with all its names:
 * `md.import: [{ from: "@kit/report@1", names: "*" }, …]`.
 */
function importing(...froms: string[]): Request {
  return pinned(...froms.map((from) => [from, undefined] as const));
}

/** The same, each import with the `integrity` it pins, if any. */
function pinned(...imports: (readonly [from: string, integrity: string | undefined])[]): Request {
  const entries = imports.map(([from, integrity]) => `    - from: "${from}"\n${integrity ? `      integrity: ${integrity}\n` : ""}      names: "*"\n`);
  return { path: "a.comp.md", files: { "a.comp.md": `---\nmd:\n  import:\n${entries.join("")}---\n` }, now: "2026-11-01T09:00:00Z" };
}

/** The local registry as the host's one registry, for every scope. */
const options = (more: Partial<ResolveOptions> = {}): ResolveOptions => ({ registries: { "*": new NpmRegistry({ url: local.url }) }, ...more });

/** The processor's diagnostics for `request`, as `code message`. */
const diagnostics = (request: Request) => process(request).diagnostics.map(({ code, message }) => `${code} ${message}`);

/** The tarballs downloaded since the last reset, as `pack@version`. */
const downloaded = () => local.hits.flatMap((hit) => /^\/tarballs\/(.+)\/([^/]+)$/.exec(hit)?.slice(1, 3).join("@") ?? []);

/** The packs whose versions lists were asked for since the last reset. */
const listed = () => local.hits.filter((hit) => !hit.startsWith("/tarballs/")).map((hit) => hit.slice(1));

/** Runs `use` while the local registry answers 500 to everything. */
async function whileFailing<T>(use: () => Promise<T>): Promise<T> {
  local.failing(true);
  try {
    return await use();
  } finally {
    local.failing(false);
  }
}

describe("a document's packs", () => {
  it("are added to the request: those it imports, then those they import, a level each round", async () => {
    const { request: ready, problems } = await resolvePacks(importing("@kit/report@1"), options());
    expect(problems).toEqual([]);
    expect(Object.keys(ready.packs ?? {})).toEqual(["@kit/report", "@kit/charts"]);
    // `@kit/report` imports `@kit/charts@1`: the newest 1.x is 1.1.0.
    expect(downloaded()).toEqual(["@kit/report@1.0.0", "@kit/charts@1.1.0"]);
    expect(diagnostics(ready)).toEqual([]);
  });

  it("give their built modules, checked, as code for the sandbox; nothing for a module not fetched", async () => {
    const { modules } = await resolvePacks(importing("@kit/charts@1"), options());
    const module = modules({ pack: "@kit/charts", version: "1.1.0", path: "dist/chart.js" });
    expect(module?.code.type).toBe("text/javascript");
    expect(await module?.code.text()).toContain("(1.1.0)");
    expect(modules({ pack: "@kit/charts", version: "1.0.0", path: "dist/chart.js" }), "a version not fetched").toBeUndefined();
  });

  it("leave the request given as it was", async () => {
    const given = importing("@kit/charts@1");
    await resolvePacks(given, options());
    expect(given.packs).toBeUndefined();
    expect(given.policy).toBeUndefined();
  });

  it("are downloaded once a version, however many imports resolve to it", async () => {
    const { request: ready } = await resolvePacks(importing("@kit/charts@1", "@kit/charts@^1.1"), options());
    expect(downloaded()).toEqual(["@kit/charts@1.1.0"]);
    expect(ready.packs?.["@kit/charts"]?.map(({ version }) => version)).toEqual(["1.1.0"]);
  });

  it("are each version the imports resolve to, when two ranges of a pack take two", async () => {
    const { request: ready } = await resolvePacks(importing("@kit/charts@1.0.0", "@kit/charts@^1.1"), options());
    expect(ready.packs?.["@kit/charts"]?.map(({ version }) => version)).toEqual(["1.0.0", "1.1.0"]);
  });

  it("keep the packs the request holds already: they aren't fetched", async () => {
    const first = await resolvePacks(importing("@kit/charts@1"), options());
    local.hits.length = 0;
    const { request: ready } = await resolvePacks({ ...importing("@kit/report@1"), packs: first.request.packs ?? {} }, options());
    expect(downloaded()).toEqual(["@kit/report@1.0.0"]);
    expect(diagnostics(ready)).toEqual([]);
  });
});

describe("the host's policy, applied before any download", () => {
  it("leaves out a pack the host refused: it isn't asked for, and the processor says why (F026)", async () => {
    const given = { ...importing("@kit/report@1"), policy: { refused: { "@kit/charts": "kept out" } } };
    const { request: ready } = await resolvePacks(given, options());
    expect(listed()).toEqual(["@kit/report"]);
    expect(ready.policy?.refused).toEqual({ "@kit/charts": "kept out" });
    expect(diagnostics(ready)).toEqual([expect.stringMatching(/^F026 `@kit\/charts` isn't available here: kept out/)]);
  });

  it("leaves out a pack the document may not import, under `imports: listed` (F026)", async () => {
    const { request: ready } = await resolvePacks({ ...importing("@kit/charts@1"), policy: { imports: "listed" } }, options());
    expect(local.hits).toEqual([]);
    expect(diagnostics(ready)).toEqual([expect.stringMatching(/^F026 this host allows only the packs its configuration lists/)]);
  });

  it("doesn't download a denied version; the processor gives the host's reason (F026)", async () => {
    const given = { ...importing("@kit/charts@1"), policy: { deny: [{ pack: "@kit/charts", version: "1.1.0", reason: "a flaw" }] } };
    const { request: ready } = await resolvePacks(given, options());
    expect(downloaded()).toEqual([]);
    expect(diagnostics(ready)).toEqual([expect.stringMatching(/^F026 `@kit\/charts@1\.1\.0` is denied here: a flaw/)]);
  });

  it("doesn't download a pack over the size cap, 5 MB when the host sets none, and says so in `policy.refused` (F026)", async () => {
    expect(DEFAULT_MAX_PACK_SIZE).toBe(5_000_000);
    const { request: ready } = await resolvePacks(importing("@kit/huge@1"), options());
    expect(downloaded()).toEqual([]);
    expect(ready.policy?.refused).toEqual({ "@kit/huge": "`@kit/huge@1.0.0` is 10.0 MB unpacked, more than this host's 5 MB" });
    expect(diagnostics(ready)).toEqual([expect.stringMatching(/^F026 `@kit\/huge` isn't available here: `@kit\/huge@1\.0\.0` is 10\.0 MB/)]);
  });

  it("takes a larger pack under a larger cap, and refuses a small one under a smaller cap", async () => {
    const larger = await resolvePacks(importing("@kit/huge@1"), options({ maxPackSize: 20_000_000 }));
    expect(larger.request.packs?.["@kit/huge"]?.[0]?.files["package.json"]).toBeDefined();
    const smaller = await resolvePacks(importing("@kit/charts@1"), options({ maxPackSize: 1_000 }));
    expect(smaller.request.policy?.refused).toEqual({ "@kit/charts": "`@kit/charts@1.1.0` is 0.0 MB unpacked, more than this host's 0.001 MB" });
  });

  it("doesn't download a pack whose registry doesn't say how large it is", async () => {
    const { request: ready } = await resolvePacks(importing("@kit/unsized@1"), options());
    expect(downloaded()).toEqual([]);
    expect(ready.policy?.refused).toEqual({ "@kit/unsized": "its registry doesn't say how large `@kit/unsized@1.0.0` is, and this host takes packs up to 5 MB" });
  });

  it("doesn't download a pack its registry publishes no SHA-512 integrity for: it couldn't be checked", async () => {
    const { request: ready } = await resolvePacks(importing("@kit/unchecked@1", "@kit/sha1@1"), options());
    expect(downloaded()).toEqual([]);
    expect(ready.policy?.refused).toEqual({
      "@kit/unchecked": "its registry publishes no SHA-512 integrity for `@kit/unchecked@1.0.0`, so it can't be checked",
      "@kit/sha1": "its registry publishes no SHA-512 integrity for `@kit/sha1@1.0.0`, so it can't be checked",
    });
  });
});

describe("checking what is downloaded", () => {
  it("serves an import pinned to its tarball's integrity, then from the cache without asking the registry", async () => {
    const pin = integrityOf(tarballOf("kit-charts-1.0.0.tgz"));
    const cache = memoryCache();
    const first = await resolvePacks(pinned(["@kit/charts@1.0.0", pin]), options({ cache }));
    expect(diagnostics(first.request)).toEqual([]);
    local.hits.length = 0;
    const again = await resolvePacks(pinned(["@kit/charts@1.0.0", pin]), options({ cache }));
    expect(local.hits).toEqual([]);
    expect(diagnostics(again.request)).toEqual([]);
  });

  it("gives F035 for a tarball whose bytes don't match the integrity its registry publishes", async () => {
    const { request: ready } = await resolvePacks(importing("@kit/broken@1"), options());
    expect(diagnostics(ready)).toEqual([expect.stringMatching(/^F035 `@kit\/broken@1\.0\.0`: its tarball doesn't match the integrity its registry publishes/)]);
  });

  it("gives F035 for a pinned integrity that isn't the registry's, without downloading", async () => {
    const { request: ready } = await resolvePacks(pinned(["@kit/charts@1.0.0", "sha512-other"]), options());
    expect(downloaded()).toEqual([]);
    expect(diagnostics(ready)).toEqual([expect.stringMatching(/^F035 `@kit\/charts@1\.0\.0` doesn't match this `integrity`/)]);
  });
});

describe("registries", () => {
  /** A registry on the local one that records the packs it is asked about. */
  const recording = (asked: string[]): Registry => {
    const npm = new NpmRegistry({ url: local.url });
    return {
      url: npm.url,
      versions: (name) => {
        asked.push(name);
        return npm.versions(name);
      },
      tarball: (version) => npm.tarball(version),
    };
  };

  it("are chosen by a pack's scope, with `*` for every other scope", async () => {
    const kit: string[] = [];
    const others: string[] = [];
    await resolvePacks(importing("@kit/charts@1", "@other/pack@1"), { registries: { "@kit": recording(kit), "*": recording(others) } });
    expect(kit).toEqual(["@kit/charts"]);
    expect(others).toEqual(["@other/pack"]);
  });

  it("give a problem for a pack whose scope has none, and no `*`; the processor says F026", async () => {
    const { request: ready, problems } = await resolvePacks(importing("@other/pack@1"), { registries: { "@kit": new NpmRegistry({ url: local.url }) } });
    expect(problems).toEqual([{ pack: "@other/pack", message: "no registry is configured for `@other/pack`'s scope" }]);
    expect(diagnostics(ready)).toEqual([expect.stringMatching(/^F026 no release of `@other\/pack`/)]);
  });

  it("give a problem when one fails, for the host's logs; the document gets the processor's F026", async () => {
    const { request: ready, problems } = await whileFailing(() => resolvePacks(importing("@kit/charts@1"), options()));
    expect(problems).toEqual([{ pack: "@kit/charts", message: expect.stringContaining("answered 500 for `@kit/charts`") as string }]);
    expect(diagnostics(ready)).toEqual([expect.stringMatching(/^F026 no release of `@kit\/charts`/)]);
  });

  it("leave a pack they don't have, or a range none of its versions match, to the processor (F026), with no problem", async () => {
    const { request: ready, problems } = await resolvePacks(importing("@kit/nothing@1", "@kit/charts@^9"), options());
    expect(problems).toEqual([]);
    expect(diagnostics(ready)).toEqual([expect.stringMatching(/^F026 no release of `@kit\/nothing`/), expect.stringMatching(/^F026 no release of `@kit\/charts` matches `@kit\/charts@\^9`/)]);
  });

  it("may be the host's own: anything with a URL, `versions` and `tarball`", async () => {
    // An in-memory registry, as a host with packs of its own might write.
    const charts: VersionInfo = { version: "1.0.0", tarball: "memory:charts", integrity: integrityOf(tarballOf("kit-charts-1.0.0.tgz")), unpackedSize: 2000 };
    const own: Registry = {
      url: "memory:/",
      versions: (name) => Promise.resolve({ versions: name === "@kit/charts" ? [charts] : [], maxAge: 60 }),
      tarball: () => Promise.resolve(new Uint8Array(tarballOf("kit-charts-1.0.0.tgz"))),
    };
    const { request: ready, problems } = await resolvePacks(importing("@kit/charts@1"), { registries: { "*": own } });
    expect(problems).toEqual([]);
    expect(ready.packs?.["@kit/charts"]?.[0]?.version).toBe("1.0.0");
    expect(diagnostics(ready)).toEqual([]);
  });
});

describe("who fetches (fetch: host or direct)", () => {
  it("on a server, any registry is fine: there is no reader to protect", async () => {
    const { problems } = await resolvePacks(importing("@kit/charts@1"), options({ fetch: "host" }));
    expect(problems).toEqual([]);
  });

  it("with `fetch: host` in a page, fetches through the host's proxy on the page's own origin", async () => {
    vi.stubGlobal("location", { href: "https://notes.example/docs/" });
    const proxy = registryProxy({ registries: { "*": new NpmRegistry({ url: local.url }) }, base: "/compmd/registry/" });
    const throughProxy: typeof fetch = (url, init) => proxy(new Request(url, init));
    const registries = { "*": new NpmRegistry({ url: "/compmd/registry/", fetch: throughProxy }) };
    const { request: ready, problems } = await resolvePacks(importing("@kit/charts@1"), { registries });
    expect(problems).toEqual([]);
    expect(diagnostics(ready)).toEqual([]);
  });

  it("with `fetch: host` in a page, refuses a registry on another origin, naming `fetch: \"direct\"`", async () => {
    vi.stubGlobal("location", { href: "https://notes.example/docs/" });
    await expect(resolvePacks(importing("@kit/charts@1"), options())).rejects.toThrow('isn\'t on this page\'s origin, and `fetch` is `host`: the reader\'s browser fetches from a registry only with `fetch: "direct"`');
  });

  it("with `fetch: host` in a page, doesn't download a tarball its registry puts on another origin", async () => {
    vi.stubGlobal("location", { href: "https://notes.example/docs/" });
    const elsewhere: Registry = {
      url: "https://notes.example/compmd/registry/",
      versions: () => Promise.resolve({ versions: [{ version: "1.0.0", tarball: "https://cdn.example/t.tgz", integrity: "sha512-x", unpackedSize: 10 }], maxAge: 0 }),
      tarball: () => Promise.reject(new Error("not fetched")),
    };
    const { problems } = await resolvePacks(importing("@kit/charts@1"), { registries: { "*": elsewhere } });
    expect(problems).toEqual([{ pack: "@kit/charts", message: expect.stringContaining("`https://cdn.example/t.tgz` isn't on this page's origin") as string }]);
  });

  it("with `fetch: direct`, a page fetches from the registries the host names, wherever they are", async () => {
    vi.stubGlobal("location", { href: "https://notes.example/docs/" });
    const { problems } = await resolvePacks(importing("@kit/charts@1"), options({ fetch: "direct" }));
    expect(problems).toEqual([]);
  });
});

describe("the cache", () => {
  it("serves a second resolve with nothing fetched: tarballs by integrity, versions lists while fresh", async () => {
    const cache = memoryCache();
    await resolvePacks(importing("@kit/report@1"), options({ cache }));
    local.hits.length = 0;
    const { request: ready } = await resolvePacks(importing("@kit/report@1"), options({ cache }));
    expect(local.hits).toEqual([]);
    expect(ready.packs?.["@kit/charts"]?.[0]?.files["dist/chart.js"]).toContain("(1.1.0)");
  });

  it("asks for a versions list again once its registry's max-age (300 s here) has passed, and downloads nothing kept", async () => {
    let now = 0;
    const cache = memoryCache(() => now);
    await resolvePacks(importing("@kit/charts@1"), options({ cache }));
    local.hits.length = 0;
    now = 301_000;
    await resolvePacks(importing("@kit/charts@1"), options({ cache }));
    expect(listed()).toEqual(["@kit/charts"]);
    expect(downloaded()).toEqual([]);
  });

  it("uses a stale versions list when the registry can't be reached, rather than none", async () => {
    let now = 0;
    const cache = memoryCache(() => now);
    await resolvePacks(importing("@kit/charts@1"), options({ cache }));
    now = 600_000;
    const offline = await whileFailing(() => resolvePacks(importing("@kit/charts@1"), options({ cache })));
    expect(offline.problems).toEqual([]);
    expect(diagnostics(offline.request)).toEqual([]);
  });

  it("may be the host's own `PackCache`, kept by integrity and by the registry's URL and the pack", async () => {
    const calls: string[] = [];
    const memory = memoryCache();
    const own: PackCache = {
      files: (integrity) => (calls.push(`files ${integrity.slice(0, 7)}`), memory.files(integrity)),
      keep: (integrity, files) => (calls.push(`keep ${integrity.slice(0, 7)}`), memory.keep(integrity, files)),
      versions: (key) => (calls.push(`versions ${key}`), memory.versions(key)),
      keepVersions: (key, versions, maxAge) => (calls.push(`keepVersions ${key} ${String(maxAge)}`), memory.keepVersions(key, versions, maxAge)),
    };
    await resolvePacks(importing("@kit/charts@1"), options({ cache: own }));
    expect(calls).toEqual([`versions ${local.url}@kit/charts`, `keepVersions ${local.url}@kit/charts 300`, "files sha512-", "keep sha512-"]);
  });
});

describe("library packs (docs/sharing-library-packs.md, decision 5)", () => {
  /** Each component's library packs, as `name: pack@version/path`. */
  const libraries = (request: Request) =>
    Object.fromEntries(new OpenDocument(request).components.map(({ name, libraries }) => [name, Object.entries(libraries ?? {}).map(([as, { pack, version, path }]) => `${as}: ${pack}@${version}/${path}`)]));

  it("are fetched at the version every range asked in a round matches, once", async () => {
    // `@kit/bars` asks `@kit/d3` at `^7.9`, `@kit/lines` at `~7.9.0`: 7.12.0 matches the first only, 7.9.5 both.
    const { request: ready, modules, problems } = await resolvePacks(importing("@kit/bars@1", "@kit/lines@1"), options());
    expect(problems).toEqual([]);
    expect(downloaded()).toEqual(["@kit/bars@1.0.0", "@kit/lines@1.0.0", "@kit/d3@7.9.5"]);
    expect(diagnostics(ready)).toEqual([]);
    expect(libraries(ready)).toEqual({ bars: ["@kit/d3: @kit/d3@7.9.5/dist/d3.js"], lines: ["@kit/d3: @kit/d3@7.9.5/dist/d3.js"] });
    expect(await modules({ pack: "@kit/d3", version: "7.9.5", path: "dist/d3.js" })?.code.text()).toContain('"7.9.5"');
  });

  it("are fetched again for a range asked in a later round, at the version every range asked matches", async () => {
    // Round 2 asks `^7.9` (from `@kit/bars`): 7.12.0. Round 3 asks `~7.9.0` (from `@kit/lines`, which `@kit/dashboard`
    // imports): 7.12.0 doesn't match it, so the version both match, 7.9.5, is fetched, and the processor takes it.
    const { request: ready } = await resolvePacks(importing("@kit/bars@1", "@kit/dashboard@1"), options());
    expect(downloaded()).toEqual(["@kit/bars@1.0.0", "@kit/dashboard@1.0.0", "@kit/lines@1.0.0", "@kit/d3@7.12.0", "@kit/d3@7.9.5"]);
    expect(diagnostics(ready)).toEqual([]);
    expect(libraries(ready)).toEqual({ bars: ["@kit/d3: @kit/d3@7.9.5/dist/d3.js"], lines: ["@kit/d3: @kit/d3@7.9.5/dist/d3.js"] });
  });

  it("follow the policy as packs do: a denied version is W121 on the imports that name it", async () => {
    const policy = { deny: [{ pack: "@kit/d3", version: "7.9.5", reason: "a flaw" }] };
    const { request: ready } = await resolvePacks({ ...importing("@kit/bars@1", "@kit/lines@1"), policy }, options());
    expect(diagnostics(ready)).toEqual([
      expect.stringMatching(/^W121 `@kit\/bars@1\.0\.0` needs `@kit\/d3` `\^7\.9`, but `@kit\/d3@7\.9\.5` is denied here: a flaw/),
      expect.stringMatching(/^W121 `@kit\/lines@1\.0\.0` needs `@kit\/d3` `~7\.9\.0`, but `@kit\/d3@7\.9\.5` is denied here: a flaw/),
    ]);
  });
});

describe("through the host's proxy", () => {
  it("resolves as through the registry itself", async () => {
    const proxy = registryProxy({ registries: { "*": new NpmRegistry({ url: local.url }) }, base: "/compmd/registry/" });
    const throughProxy: typeof fetch = (url, init) => proxy(new Request(url, init));
    const registries = { "*": new NpmRegistry({ url: "https://notes.example/compmd/registry/", fetch: throughProxy }) };
    const { request: ready, problems } = await resolvePacks(importing("@kit/report@1"), { registries });
    expect(problems).toEqual([]);
    expect(Object.keys(ready.packs ?? {})).toEqual(["@kit/report", "@kit/charts"]);
    expect(downloaded()).toEqual(["@kit/report@1.0.0", "@kit/charts@1.1.0"]);
  });
});
