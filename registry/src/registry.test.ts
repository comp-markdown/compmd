// `NpmRegistry`, as a host or the proxy uses it: one scenario a test, each a good use or a bad one. The packs are the
// local registry's (test/local-registry.ts).

import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { integrityOf, localRegistry, tarballOf, type LocalRegistry } from "../test/local-registry.ts";
import { isPackName, maxAgeOf, NpmRegistry } from "./registry.ts";

let local: LocalRegistry;
beforeAll(async () => {
  local = await localRegistry();
});
afterAll(() => local.close());
afterEach(() => {
  vi.unstubAllGlobals();
});

/** Runs `use` while the local registry answers 500 to everything. */
async function whileFailing<T>(use: () => Promise<T>): Promise<T> {
  local.failing(true);
  try {
    return await use();
  } finally {
    local.failing(false);
  }
}

/** A registry whose every answer is `packument`, with `cache-control: max-age=60`. */
const answering = (packument: unknown) =>
  new NpmRegistry({ url: "https://npm.example/", fetch: () => Promise.resolve(new Response(JSON.stringify(packument), { headers: { "cache-control": "max-age=60" } })) });

describe("NpmRegistry: a pack's versions", () => {
  it("lists every version with its tarball, integrity and unpacked size, and how long the list may be kept", async () => {
    const { versions, maxAge } = await new NpmRegistry({ url: local.url }).versions("@kit/charts");
    expect(maxAge).toBe(300);
    expect(versions.map(({ version }) => version)).toEqual(["1.0.0", "1.1.0"]);
    expect(versions[0]).toEqual({
      version: "1.0.0",
      tarball: `${local.url}tarballs/@kit/charts/1.0.0`,
      integrity: integrityOf(tarballOf("kit-charts-1.0.0.tgz")),
      unpackedSize: 2000,
    });
  });

  it("is none, not an error, for a pack the registry doesn't have", async () => {
    expect(await new NpmRegistry({ url: local.url }).versions("@kit/nothing")).toEqual({ versions: [], maxAge: 0 });
  });

  it("refuses a name that isn't a pack's before asking: only scoped npm names reach a URL", async () => {
    await expect(new NpmRegistry({ url: local.url }).versions("chartkit")).rejects.toThrow("`chartkit` isn't a pack's name");
    expect(isPackName("@kit/charts")).toBe(true);
    expect(isPackName("@kit/charts/../x"), "a path").toBe(false);
    expect(isPackName("@Kit/charts"), "upper case").toBe(false);
    expect(isPackName("charts"), "unscoped").toBe(false);
  });

  it("fails when the registry fails, naming it and the pack", async () => {
    await expect(whileFailing(() => new NpmRegistry({ url: local.url }).versions("@kit/charts"))).rejects.toThrow(`${local.url} answered 500 for \`@kit/charts\``);
  });

  it("reads the metadata as a registry may write it: relative tarball URLs, and fields it leaves out", async () => {
    const registry = answering({
      versions: {
        "1.0.0": { dist: { tarball: "-/charts-1.0.0.tgz", integrity: "sha512-a", unpackedSize: 10 } },
        "1.1.0": { dist: { tarball: "https://cdn.example/charts-1.1.0.tgz" } },
        "2.0.0": { dist: {} },
        "3.0.0": {},
      },
    });
    const { versions, maxAge } = await registry.versions("@kit/charts");
    expect(maxAge).toBe(60);
    expect(versions, "a version without a tarball is left out").toEqual([
      { version: "1.0.0", tarball: "https://npm.example/-/charts-1.0.0.tgz", integrity: "sha512-a", unpackedSize: 10 },
      { version: "1.1.0", tarball: "https://cdn.example/charts-1.1.0.tgz" },
    ]);
    expect(await answering({}).versions("@kit/charts"), "no versions at all").toEqual({ versions: [], maxAge: 60 });
  });
});

describe("NpmRegistry: metadata, as the proxy passes it on", () => {
  it("is the registry's short metadata as it answered, with its max-age", async () => {
    const metadata = await new NpmRegistry({ url: local.url }).metadata("@kit/charts");
    expect(metadata?.maxAge).toBe(300);
    expect(Object.keys((JSON.parse(metadata?.body ?? "{}") as { versions: object }).versions)).toEqual(["1.0.0", "1.1.0"]);
  });

  it("is undefined for a pack the registry doesn't have", async () => {
    expect(await new NpmRegistry({ url: local.url }).metadata("@kit/nothing")).toBeUndefined();
  });
});

describe("NpmRegistry: a version's tarball", () => {
  it("is its bytes as the registry sent them, unchecked: the resolver checks them", async () => {
    const bytes = await new NpmRegistry({ url: local.url }).tarball({ version: "1.0.0", tarball: `${local.url}tarballs/@kit/charts/1.0.0` });
    expect(Buffer.from(bytes)).toEqual(tarballOf("kit-charts-1.0.0.tgz"));
  });

  it("fails when the registry fails, naming the tarball", async () => {
    const tarball = `${local.url}tarballs/@kit/charts/1.0.0`;
    await expect(whileFailing(() => new NpmRegistry({ url: local.url }).tarball({ version: "1.0.0", tarball }))).rejects.toThrow(`${tarball} answered 500`);
  });
});

describe("NpmRegistry: requests", () => {
  it("send no credentials, only the headers the host gives, such as a private registry's token", async () => {
    const sent: RequestInit[] = [];
    const fetcher = (_url: string | URL | Request, init?: RequestInit) => {
      sent.push(init ?? {});
      return Promise.resolve(new Response("{}", { status: 200 }));
    };
    await new NpmRegistry({ url: "https://npm.example/", fetch: fetcher, headers: { authorization: "Bearer t" } }).versions("@kit/charts");
    expect(sent).toEqual([{ credentials: "omit", headers: { authorization: "Bearer t", accept: "application/vnd.npm.install-v1+json" } }]);
  });

  it("go to the registry's URL, which ends in `/` whether it was given with one or not", () => {
    expect(new NpmRegistry({ url: "https://npm.example/registry" }).url).toBe("https://npm.example/registry/");
    expect(new NpmRegistry({ url: "https://npm.example/registry/" }).url).toBe("https://npm.example/registry/");
  });

  it("may go to a URL relative to the page, such as the host's proxy", () => {
    vi.stubGlobal("location", { href: "https://notes.example/docs/a" });
    expect(new NpmRegistry({ url: "/compmd/registry/" }).url).toBe("https://notes.example/compmd/registry/");
  });

  it("can't go to a relative URL where there is no page to resolve it against, as on a server", () => {
    expect(() => new NpmRegistry({ url: "/compmd/registry/" })).toThrow(TypeError);
  });
});

describe("maxAgeOf", () => {
  it("reads max-age from a Cache-Control header, and is 0 without one", () => {
    expect(maxAgeOf("public, max-age=300")).toBe(300);
    expect(maxAgeOf("no-store")).toBe(0);
    expect(maxAgeOf(null)).toBe(0);
  });
});
