import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { builtFrames, folder } from "./frames.testing.ts";
import { buildHashedFrames, FRAMES_JSON, hashOf } from "./index.ts";

describe("buildHashedFrames", () => {
  it("builds into a folder named by the files' hash, and names it in frames.json", async () => {
    const out = await folder();
    const from = await builtFrames();
    const built = await buildHashedFrames({ out, from, source: "app://localhost/frames/" });
    expect(built.hash).toMatch(/^[0-9a-f]{16}$/);
    expect(built.folder).toBe(join(out, built.hash));
    expect(await hashOf(built.folder)).toBe(built.hash);
    expect(JSON.parse(await readFile(join(out, FRAMES_JSON), "utf8"))).toEqual({ hash: built.hash });
    expect(await readFile(join(built.folder, "strict.html"), "utf8")).toContain('data-compmd-source="app://localhost/frames/"');
    expect(built.definition.runtimes.map(({ name }) => name)).toEqual(["react"]);
    expect((await readdir(out)).sort(), "nothing left of the staging folder").toEqual([FRAMES_JSON, built.hash].sort());
  });

  it("gives the same files the same hash, and changed files a new one, removing the older build and nothing else", async () => {
    const out = await folder();
    const from = await builtFrames();
    const first = await buildHashedFrames({ out, from });
    expect((await buildHashedFrames({ out, from })).hash, "the same files").toBe(first.hash);
    await mkdir(join(out, "keep"));
    await writeFile(join(out, "notes.txt"), "the app's own");
    await writeFile(join(from, "runtime.js"), "// a new runtime\n");
    const second = await buildHashedFrames({ out, from });
    expect(second.hash).not.toBe(first.hash);
    expect((await readdir(out)).sort()).toEqual([FRAMES_JSON, second.hash, "keep", "notes.txt"].sort());
  });

  it("hashes each file's path with its bytes, so moving bytes between files changes it", async () => {
    const one = await folder();
    const other = await folder();
    await writeFile(join(one, "a"), "bc");
    await writeFile(join(other, "ab"), "c");
    expect(await hashOf(one)).not.toBe(await hashOf(other));
  });
});
