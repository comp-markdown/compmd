// A tarball's intake, bounded from the download to the files (audit S10 to S12): each way a tarball can be too large or
// malformed is refused with a message, before it is held whole, and a normal pack still comes. One scenario a test.

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";
import { initSync, process, type Request as ProcessRequest } from "@compmd/processor";
import { beforeAll, describe, expect, it } from "vitest";
import { integrityOf, tarballOf } from "../test/local-registry.ts";
import { archive, header } from "../test/tar-archive.ts";
import { intake } from "./intake.ts";
import { NpmRegistry, type Registry, type VersionInfo } from "./registry.ts";
import { resolvePacks } from "./resolve.ts";

beforeAll(() => {
  initSync({ module: readFileSync(fileURLToPath(import.meta.resolve("@compmd/processor/compmd_wasm_bg.wasm"))) });
});

/** A registry in memory that serves `tarball` for every version, as a host's own might. */
function serving(tarball: Uint8Array<ArrayBuffer>, unpackedSize: number): { registry: Registry; info: VersionInfo & { integrity: string } } {
  const info = { version: "1.0.0", tarball: "memory:t.tgz", integrity: integrityOf(Buffer.from(tarball)), unpackedSize };
  const registry: Registry = {
    url: "memory:/",
    versions: () => Promise.resolve({ versions: [info], maxAge: 0 }),
    tarball: () => Promise.resolve(tarball),
  };
  return { registry, info };
}

/** `tar`, gzipped as a tarball is. */
const gzipped = (tar: Uint8Array): Uint8Array<ArrayBuffer> => new Uint8Array(gzipSync(tar));

describe("a tarball's intake", () => {
  it("gives a normal pack's files", async () => {
    const { registry, info } = serving(new Uint8Array(tarballOf("kit-charts-1.1.0.tgz")), 2000);
    const files = await intake(registry, info, info.integrity, 10_000_000);
    expect([...(files?.keys() ?? [])].sort()).toEqual(["dist/chart.js", "index.comp.md", "package.json"]);
  });

  it("gives nothing for bytes that don't match the integrity", async () => {
    const { registry, info } = serving(new Uint8Array(tarballOf("kit-charts-1.1.0.tgz")), 2000);
    expect(await intake(registry, info, "sha512-other", 10_000_000)).toBeUndefined();
  });

  it("refuses a tarball larger than the cap, even from a registry that read it all", async () => {
    const { registry, info } = serving(new Uint8Array(tarballOf("kit-charts-1.1.0.tgz")), 2000);
    await expect(intake(registry, info, info.integrity, 100)).rejects.toThrow("memory:t.tgz is larger than 100 bytes");
  });

  it("refuses a small tarball whose gzip expands past the cap", async () => {
    const { registry, info } = serving(gzipped(new Uint8Array(5_000_000)), 1000);
    await expect(intake(registry, info, info.integrity, 1_000_000)).rejects.toThrow("it unzips to more than 1000000 bytes");
  });

  it("refuses files larger than the unpacked size the registry declares", async () => {
    const { registry, info } = serving(gzipped(archive(["package/a.js", "x".repeat(5000)])), 1000);
    await expect(intake(registry, info, info.integrity, 10_000_000)).rejects.toThrow("its files take more than the 1000 bytes its registry declares");
  });

  it("refuses an entry with a negative size, which would read the tar backwards for ever", async () => {
    const tar = new Uint8Array(4096);
    tar.set(header("package/a.js", "-0000002000", "0"));
    const { registry, info } = serving(gzipped(tar), 1000);
    await expect(intake(registry, info, info.integrity, 10_000_000)).rejects.toThrow("a tar entry's size isn't a whole number of bytes");
  });
});

describe("resolving through the intake", () => {
  const request: ProcessRequest = { path: "a.comp.md", files: { "a.comp.md": '---\nmd:\n  import:\n    - from: "@kit/charts@1"\n      names: "*"\n---\n' }, now: "2026-11-01T09:00:00Z" };

  it("gives a refused tarball as a problem for the host's logs, and the processor's F026 to the document", async () => {
    const { registry } = serving(gzipped(archive(["package/a.js", "x".repeat(5000)])), 1000);
    const { request: ready, problems } = await resolvePacks(request, { registries: { "*": registry } });
    expect(problems).toEqual([{ pack: "@kit/charts", message: "its files take more than the 1000 bytes its registry declares" }]);
    expect(process(ready).diagnostics.map(({ code }) => code)).toEqual(["F026"]);
  });

  it("refuses metadata naming a tarball on another origin than its registry's, fetching nothing from there", async () => {
    const asked: string[] = [];
    const metadata = { versions: { "1.0.0": { dist: { tarball: "https://elsewhere.example/charts.tgz", integrity: "sha512-x", unpackedSize: 10 } } } };
    const fetcher = (url: string | URL | Request) => {
      asked.push(url instanceof Request ? url.url : url.toString());
      return Promise.resolve(new Response(JSON.stringify(metadata)));
    };
    const registry = new NpmRegistry({ url: "https://npm.example/", fetch: fetcher, headers: { authorization: "Bearer secret" } });
    const { problems } = await resolvePacks(request, { registries: { "*": registry } });
    expect(problems).toEqual([
      { pack: "@kit/charts", message: "https://elsewhere.example/charts.tgz isn't on its registry's origin, https://npm.example: a tarball is fetched only from there" },
    ]);
    expect(asked).toEqual(["https://npm.example/@kit%2fcharts"]);
  });
});
