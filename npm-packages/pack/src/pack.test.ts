// Reading a pack's folder: its modules, from its document or its library's entry.
import { rm } from "node:fs/promises";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { documentOf, folder, write } from "../test/folders.ts";
import { readPack } from "./pack.ts";

let dir: string;

beforeEach(async () => {
  dir = await folder();
});

afterEach(async () => {
  await rm(dir, { recursive: true, force: true });
});

describe("readPack", () => {
  it("finds each built module its document imports, and the source it is built from", async () => {
    await write(dir, {
      "package.json": { name: "@kit/charts", version: "0.1.0", compmd: "./index.comp.md" },
      "index.comp.md": documentOf(["dist/line.js", "dist/bar.js", "notes.comp.md"]),
      "src/line.ts": "",
      "src/bar.tsx": "",
    });
    const pack = await readPack(dir);
    expect(pack.kind).toBe("components");
    expect(pack.modules).toEqual([
      { source: "src/line.ts", out: "dist/line.js" },
      { source: "src/bar.tsx", out: "dist/bar.js" },
    ]);
  });

  it("finds a library pack's library, the package its entry re-exports", async () => {
    await write(dir, { "package.json": { name: "@kit/three", version: "0.0.0", compmd: { library: "dist/index.js" } }, "src/index.ts": '// As packs import it: `import { … } from "@kit/three"`.\nexport * from "three";\nexport { default } from "three/addons";\n' });
    const pack = await readPack(dir);
    expect(pack).toMatchObject({ kind: "library", library: "three", modules: [{ source: "src/index.ts", out: "dist/index.js" }] });
  });

  it("says what to fix in a folder it can't read", async () => {
    await write(dir, { "package.json": { name: "@kit/charts", version: "0.1.0" } });
    await expect(readPack(dir)).rejects.toThrow(/names the pack's document/);
    await write(dir, { "package.json": { name: "@kit/charts", version: "0.1.0", compmd: "./index.comp.md" } });
    await expect(readPack(dir)).rejects.toThrow("package.json's `compmd` is ./index.comp.md, which isn't there");
    await write(dir, { "index.comp.md": documentOf(["dist/line.js"]) });
    await expect(readPack(dir)).rejects.toThrow("dist/line.js is imported by the pack's document, but there is no src/line.ts (or .tsx, .js, .jsx) to build it from");
    await write(dir, { "package.json": { name: "@kit/two", version: "0.0.0", compmd: { library: "dist/index.js" } } });
    await expect(readPack(dir)).rejects.toThrow("a library pack builds dist/index.js from src/index.ts, which isn't there");
    await write(dir, { "src/index.ts": 'export * from "one";\nexport * from "two";\n' });
    await expect(readPack(dir)).rejects.toThrow(/should re-export one npm package/);
  });
});
