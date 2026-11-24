import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { publishedFiles } from "./published.ts";

let pack: string;

beforeEach(async () => {
  pack = join(await mkdtemp(join(tmpdir(), "compmd-published-")), "pack");
  const files = ["package.json", "README.md", "LICENSE", "index.comp.md", "dist/a.js", "dist/b.js", "dist/chunks/c.js", "dist/.cache/x", "src/a.ts", "node_modules/x/index.js"];
  for (const path of [...files, "../secret.txt"]) {
    await mkdir(dirname(join(pack, path)), { recursive: true });
    await writeFile(join(pack, path), path);
  }
});

afterEach(async () => {
  await rm(dirname(pack), { recursive: true, force: true });
});

describe("publishedFiles", () => {
  it("is what `files` matches, globs and folders, and what npm always adds", async () => {
    expect(await publishedFiles(pack, ["dist/*.js", "./index.comp.md"])).toEqual(["LICENSE", "README.md", "dist/a.js", "dist/b.js", "index.comp.md", "package.json"]);
    expect(await publishedFiles(pack, ["dist/", "!dist/b.js"])).toEqual(["LICENSE", "README.md", "dist/a.js", "dist/b.js", "dist/chunks/c.js", "package.json"]);
  });

  it("drops a match outside the pack, and its own dependencies", async () => {
    expect(await publishedFiles(pack, ["../**", "../secret.txt", "/index.comp.md", "node_modules/**"])).toEqual(["LICENSE", "README.md", "index.comp.md", "package.json"]);
  });

  it("is undefined without a list: npm publishes the whole folder", async () => {
    expect(await publishedFiles(pack, undefined)).toBeUndefined();
  });
});
