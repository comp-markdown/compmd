// What kind a package is, from its manifest and types; its custom elements by tag; and an installed package found from a
// folder below it.
import { rm } from "node:fs/promises";
import { join } from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { folder, write } from "../../test/folders.ts";
import { WRAPPED } from "../../test/wrapped.ts";
import { installedPackage } from "./installed.ts";
import { kindOf } from "./kind.ts";
import { elementTags } from "./props.ts";

let dir: string;

beforeAll(async () => {
  dir = await folder();
  await write(dir, {
    ...WRAPPED,
    "node_modules/conditioned/package.json": { name: "conditioned", version: "1.0.0", exports: { ".": { svelte: "./index.svelte", default: "./index.js" } } },
    "node_modules/described/package.json": { name: "described", version: "1.0.0" },
    "node_modules/described/custom-elements.json": "{}",
  });
});

afterAll(async () => {
  await rm(dir, { recursive: true, force: true });
});

const kind = (name: string) => {
  const installed = installedPackage(dir, name);
  if (!installed) throw new Error(`${name} isn't in the fixture`);
  return kindOf(dir, installed);
};

describe("kindOf", () => {
  it("reads React and Svelte from the peer dependencies or the export conditions, a web component from its types or its manifest, else a library", () => {
    expect(["react", "charts", "conditioned", "viewer", "described", "plain"].map(kind)).toEqual(["library", "svelte", "svelte", "web-component", "web-component", "library"]);
    expect(kindOf(dir, { dir: join(dir, "node_modules/qr"), name: "qr", version: "2.2.0", peerDependencies: { react: "^19" } })).toBe("react");
  });
});

describe("elementTags", () => {
  it("lists the custom elements a package's types define, not the DOM's", () => {
    expect(elementTags(dir, "viewer")).toEqual(["x-viewer"]);
    expect(elementTags(dir, "qr")).toEqual([]);
  });
});

describe("installedPackage", () => {
  it("finds a package from a folder below the one it is installed in, and nothing that isn't installed", () => {
    expect(installedPackage(join(dir, "src", "deep"), "qr")).toMatchObject({ name: "qr", version: "2.2.0", dir: join(dir, "node_modules/qr") });
    expect(installedPackage(dir, "nowhere-at-all")).toBeUndefined();
  });
});
