import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { integrityOf, localRegistry, tarballOf, type LocalRegistry } from "../test/local-registry.ts";
import { isPackName, maxAgeOf, NpmRegistry } from "./registry.ts";

let local: LocalRegistry;
beforeAll(async () => {
  local = await localRegistry();
});
afterAll(() => local.close());

describe("NpmRegistry", () => {
  it("lists a pack's versions from npm's short metadata, with how long to keep them", async () => {
    const npm = new NpmRegistry({ url: local.url });
    const { versions, maxAge } = await npm.versions("@kit/charts");
    expect(maxAge).toBe(300);
    expect(versions.map(({ version }) => version)).toEqual(["1.0.0", "1.1.0"]);
    expect(versions[0]).toEqual({
      version: "1.0.0",
      tarball: `${local.url}tarballs/@kit/charts/1.0.0`,
      integrity: integrityOf(tarballOf("kit-charts-1.0.0.tgz")),
      unpackedSize: 2000,
    });
    expect(local.hits.at(-1)).toBe("/@kit/charts");
  });

  it("gives no versions for a pack the registry doesn't have, and fails on anything else", async () => {
    const npm = new NpmRegistry({ url: local.url.slice(0, -1) });
    expect(npm.url).toBe(local.url);
    expect(await npm.versions("@kit/nothing")).toEqual({ versions: [], maxAge: 0 });
    await expect(npm.versions("chartkit")).rejects.toThrow("`chartkit` isn't a pack's name");
    local.failing(true);
    await expect(npm.versions("@kit/charts")).rejects.toThrow("answered 500");
    await expect(npm.tarball({ version: "1.0.0", tarball: `${local.url}tarballs/@kit/charts/1.0.0` })).rejects.toThrow("answered 500");
    local.failing(false);
  });

  it("gets a tarball's bytes as they are", async () => {
    const npm = new NpmRegistry({ url: local.url });
    const bytes = await npm.tarball({ version: "1.0.0", tarball: `${local.url}tarballs/@kit/charts/1.0.0` });
    expect(Buffer.from(bytes)).toEqual(tarballOf("kit-charts-1.0.0.tgz"));
  });

  it("sends no credentials, only its own headers", async () => {
    const sent: RequestInit[] = [];
    const fetcher = (_url: string | URL | Request, init?: RequestInit) => {
      sent.push(init ?? {});
      return Promise.resolve(new Response("{}", { status: 200 }));
    };
    const npm = new NpmRegistry({ url: "https://npm.example/", fetch: fetcher, headers: { authorization: "Bearer t" } });
    await npm.versions("@kit/charts");
    expect(sent[0]).toEqual({ credentials: "omit", headers: { authorization: "Bearer t", accept: "application/vnd.npm.install-v1+json" } });
  });

  it("needs a page for a relative URL", () => {
    expect(() => new NpmRegistry({ url: "/compmd/registry/" })).toThrow();
  });
});

describe("names and headers", () => {
  it("takes only scoped npm names", () => {
    expect(isPackName("@kit/charts")).toBe(true);
    expect(isPackName("@kit/charts/../x")).toBe(false);
    expect(isPackName("@Kit/charts")).toBe(false);
    expect(isPackName("charts")).toBe(false);
  });

  it("reads max-age from Cache-Control", () => {
    expect(maxAgeOf("public, max-age=300")).toBe(300);
    expect(maxAgeOf("no-store")).toBe(0);
    expect(maxAgeOf(null)).toBe(0);
  });
});
