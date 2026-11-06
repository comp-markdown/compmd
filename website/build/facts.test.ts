// @vitest-environment node
// The facts the landing page states, counted from the repository: a test folder's cases, a file's gzipped size, and
// the repository's own, which the page shows.
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { casesIn, factsOf, gzippedKb } from "./facts.ts";

const made: string[] = [];
afterEach(() => {
  for (const folder of made.splice(0)) rmSync(folder, { recursive: true, force: true });
});

/** A new folder with `files`, by their paths in it. */
function folderWith(files: Record<string, string>): string {
  const folder = mkdtempSync(join(tmpdir(), "compmd-facts-"));
  made.push(folder);
  for (const [path, text] of Object.entries(files)) {
    mkdirSync(join(folder, path, ".."), { recursive: true });
    writeFileSync(join(folder, path), text);
  }
  return folder;
}

describe("casesIn", () => {
  it("counts the case.json files at any depth, and nothing else", () => {
    const folder = folderWith({ "a/one/case.json": "{}", "a/two/case.json": "{}", "b/deep/three/case.json": "{}", "b/deep/three/input.comp.md": "", "README.md": "" });
    expect(casesIn(folder)).toBe(3);
  });
});

describe("gzippedKb", () => {
  it("is the file's size gzipped, in thousands of bytes", () => {
    const folder = folderWith({ "same.txt": "a".repeat(100_000) });
    const kb = gzippedKb(join(folder, "same.txt"));
    // A run of one letter compresses to almost nothing.
    expect(kb).toBeGreaterThanOrEqual(0);
    expect(kb).toBeLessThan(2);
  });
});

describe("factsOf", () => {
  it("counts the repository's own cases and weighs its processor", () => {
    const facts = factsOf(join(import.meta.dirname, "../.."));
    expect(facts.cases).toBeGreaterThan(200);
    expect(facts.wasmKb).toBeGreaterThan(100);
  });
});
