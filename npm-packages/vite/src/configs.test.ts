import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { configFolderOf, configFolders, holdsConfig } from "./configs.ts";

const ROOT = join(import.meta.dirname, "../test/.out/configs");

/** Writes `files` under the root, by their `/`-separated paths. */
function write(files: Record<string, string>): void {
  for (const [path, text] of Object.entries(files)) {
    const file = join(ROOT, ...path.split("/"));
    mkdirSync(join(file, ".."), { recursive: true });
    writeFileSync(file, text);
  }
}

beforeAll(() => {
  rmSync(ROOT, { recursive: true, force: true });
  write({
    "compmd.config.json": "{}",
    "playground/compmd.config.json": "{}",
    "playground/src/App.svelte": "",
    "playground/packs/mini/package.json": JSON.stringify({ name: "@a/mini", compmd: "mini.comp.md" }),
    "community/package.json": JSON.stringify({ name: "community", compmd: { packs: {} } }),
    "community/deep/main.ts": "",
    "broken/package.json": "{",
    "node_modules/@a/pack/compmd.config.json": "{}",
    ".cache/compmd.config.json": "{}",
    "dist/compmd.config.json": "{}",
  });
});

afterAll(() => {
  rmSync(ROOT, { recursive: true, force: true });
});

describe("configFolders", () => {
  it("finds every folder with a configuration, the root first, but not in node_modules, hidden or skipped folders", () => {
    expect(configFolders(ROOT, [join(ROOT, "dist")])).toEqual(["", "community", "playground"]);
  });

  it("takes a package.json's `compmd` object, but not a pack's document name or broken JSON", () => {
    expect(holdsConfig(join(ROOT, "community"))).toBe(true);
    expect(holdsConfig(join(ROOT, "playground/packs/mini"))).toBe(false);
    expect(holdsConfig(join(ROOT, "broken"))).toBe(false);
    expect(holdsConfig(join(ROOT, "playground/src"))).toBe(false);
  });
});

describe("configFolderOf", () => {
  const folders = ["", "community", "playground"];

  it("is the nearest folder with a configuration above the module", () => {
    expect(configFolderOf(join(ROOT, "playground/src/App.svelte"), ROOT, folders)).toBe("playground");
    expect(configFolderOf(join(ROOT, "playground/main.ts?import"), ROOT, folders)).toBe("playground");
    expect(configFolderOf(join(ROOT, "community/deep/main.ts"), ROOT, folders)).toBe("community");
    expect(configFolderOf(join(ROOT, "src/main.ts"), ROOT, folders)).toBe("");
    expect(configFolderOf(join(ROOT, "main.ts"), ROOT, folders)).toBe("");
  });

  it("is the root's for a module outside the root, or no module", () => {
    expect(configFolderOf(join(ROOT, "../elsewhere/main.ts"), ROOT, folders)).toBe("");
    expect(configFolderOf(undefined, ROOT, folders)).toBe("");
  });
});
