// The publish check: each thing a host would refuse, or render wrong, in a pack about to be published.
import { rm, truncate } from "node:fs/promises";
import { join } from "node:path";
import { DEFAULT_MAX_PACK_SIZE } from "@compmd/registry";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { documentOf, folder, write } from "../test/folders.ts";
import { checkPack } from "./check.ts";
import { readPack } from "./pack.ts";

let dir: string;

/** A built pack of one component, as `compmd-pack build` leaves it. */
const PACK = {
  "package.json": { name: "@kit/charts", version: "0.1.0", compmd: "./index.comp.md", files: ["dist", "index.comp.md", "README.md"] },
  "index.comp.md": documentOf(["dist/line.js"]),
  "src/line.ts": "export default () => undefined;",
  "dist/line.js": "export default () => undefined;",
  "dist/LICENSES.md": "",
  "README.md": "# @kit/charts",
};

beforeEach(async () => {
  dir = await folder();
  await write(dir, PACK);
});

afterEach(async () => {
  await rm(dir, { recursive: true, force: true });
});

const problems = async () => checkPack(await readPack(dir));

describe("checkPack", () => {
  it("finds nothing wrong with a pack as the build leaves it", async () => {
    expect(await problems()).toEqual([]);
  });

  it("finds a document whose version isn't package.json's", async () => {
    await write(dir, { "index.comp.md": documentOf(["dist/line.js"], "0.2.0") });
    expect(await problems()).toEqual(["index.comp.md's export.version is 0.2.0, package.json's version is 0.1.0: write the same in both (F022 otherwise)"]);
  });

  it("finds a module that isn't built, and one `files` doesn't publish", async () => {
    await rm(join(dir, "dist/line.js"));
    expect(await problems()).toEqual(["index.comp.md imports ./dist/line.js, which isn't there: run the build, or fix the import"]);
    await write(dir, { "dist/line.js": "", "package.json": { ...PACK["package.json"], files: ["index.comp.md"] } });
    expect(await problems()).toEqual(["index.comp.md imports ./dist/line.js, which package.json's `files` doesn't publish"]);
  });

  it("finds a chunk beside the modules, which a host couldn't load", async () => {
    await write(dir, { "dist/chunk-a1.js": "" });
    expect(await problems()).toEqual(["dist/ has dist/chunk-a1.js beside the pack's modules: a built module is one file (W121 otherwise); remove what an old build left"]);
  });

  it("finds a pack larger than a host takes by default", async () => {
    await write(dir, { "dist/big.wasm": "" });
    await truncate(join(dir, "dist/big.wasm"), DEFAULT_MAX_PACK_SIZE + 1);
    expect(await problems()).toEqual([`the pack is about 10.0 MB unpacked, more than the 10 MB a host takes by default`]);
  });

  it("finds a library pack whose version isn't its library's, or that isn't built", async () => {
    await write(dir, {
      "package.json": { name: "@kit/three", version: "0.0.0", compmd: { library: "dist/index.js" }, files: ["dist"] },
      "src/index.ts": 'export * from "three";',
      "node_modules/three/package.json": { name: "three", version: "0.180.0" },
    });
    await rm(join(dir, "dist/line.js"));
    expect(await problems()).toEqual(["package.json's version is 0.0.0, three's is 0.180.0: a library pack's version is its library's, which the build writes", "dist/index.js is missing: run the build"]);
  });
});
