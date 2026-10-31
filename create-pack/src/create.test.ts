import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { componentOf, createPack, CreateError, problemOf } from "./create.ts";

let dir: string;

beforeEach(async () => {
  dir = await mkdtemp(join(tmpdir(), "compmd-create-pack-"));
});

afterEach(async () => {
  await rm(dir, { recursive: true, force: true });
});

describe("createPack", () => {
  it("writes a leaf's pack, with its names filled in", async () => {
    const folder = join(dir, "badge");
    const written = await createPack({ folder, name: "@kit/badge", component: "badge" });
    expect(written).toEqual([
      ".gitignore",
      "README.md",
      "demo/compmd.config.json",
      "demo/demo.comp.md",
      "demo/index.html",
      "demo/main.ts",
      "index.comp.md",
      "package.json",
      "scripts/check.js",
      "src/badge.test.ts",
      "src/badge.ts",
      "tsconfig.json",
      "vite.config.ts",
    ]);
    const document = await readFile(join(folder, "index.comp.md"), "utf8");
    expect(document).toContain('    - from: ./dist/badge.js\n      names: [badge]\n  export:\n    pack: "@kit/badge"');
    expect(document).toContain("```compmd component=badge signature\nparams:\n  value: string\n```");
    expect(await readFile(join(folder, "demo/demo.comp.md"), "utf8")).toContain('- from: "@kit/badge@0"');
    expect(await readFile(join(folder, "vite.config.ts"), "utf8")).toContain('fileName: () => "badge.js"');
    const all = await Promise.all(written.map((path) => readFile(join(folder, path), "utf8")));
    expect(all.join("\n")).not.toMatch(/__(pack|component|language|version:[^_]*)__/);
  });

  it("pins the packages it builds with to this package's versions", async () => {
    const folder = join(dir, "badge");
    await createPack({ folder, name: "@kit/badge", component: "badge" });
    const manifest = JSON.parse(await readFile(join(folder, "package.json"), "utf8")) as { name: string; devDependencies: Record<string, string> };
    const ours = JSON.parse(await readFile(join(import.meta.dirname, "../package.json"), "utf8")) as { devDependencies: Record<string, string> };
    expect(manifest.name).toBe("@kit/badge");
    expect(manifest.devDependencies.vite).toBe(ours.devDependencies.vite);
    expect(manifest.devDependencies.vitest).toBe(ours.devDependencies.vitest);
    // In the repository, `workspace:` is the version installed: the one the repository releases.
    const vite = JSON.parse(await readFile(join(import.meta.dirname, "../../vite/package.json"), "utf8")) as { version: string };
    expect(manifest.devDependencies["@compmd/vite"]).toBe(vite.version);
    expect(Object.values(manifest.devDependencies).every((version) => /^\d+\.\d+\.\d+$/.test(version))).toBe(true);
  });

  it("writes a code renderer's pack with --renders", async () => {
    const folder = join(dir, "diagram");
    await createPack({ folder, name: "@kit/diagram", component: "diagram", renders: "dot" });
    expect(await readFile(join(folder, "index.comp.md"), "utf8")).toContain("```compmd component=diagram signature\nrenders: dot\n");
    expect(await readFile(join(folder, "demo/demo.comp.md"), "utf8")).toContain('```dot {alt="Two lines of dot"}');
    expect(await readFile(join(folder, "src/diagram.ts"), "utf8")).toContain("code: string;");
  });

  it("writes into an empty folder, and refuses one with files", async () => {
    const empty = join(dir, "empty");
    await mkdir(empty);
    await expect(createPack({ folder: empty, name: "@kit/empty", component: "empty" })).resolves.toContain("package.json");
    const full = join(dir, "full");
    await mkdir(full);
    await writeFile(join(full, "notes.txt"), "");
    await expect(createPack({ folder: full, name: "@kit/full", component: "full" })).rejects.toThrow(new CreateError(`${full} has files in it: choose a new folder, or an empty one`));
  });

  it("refuses names a pack can't have", async () => {
    await expect(createPack({ folder: join(dir, "x"), name: "badge", component: "badge" })).rejects.toThrow(CreateError);
  });
});

describe("problemOf", () => {
  it("says what's wrong with a name, a component or a language", () => {
    expect(problemOf({ folder: "x", name: "@kit/badge", component: "status-badge", renders: "vega-lite" })).toBeUndefined();
    expect(problemOf({ folder: "x", name: "badge", component: "badge" })).toBe("badge isn't a scoped npm name, such as @kit/mermaid: packs are always scoped");
    expect(problemOf({ folder: "x", name: "@kit/badge", component: "Badge" })).toBe("Badge isn't a component name: lowercase letters and digits, words joined by -");
    expect(problemOf({ folder: "x", name: "@kit/badge", component: "badge", renders: "C#" })).toBe("C# isn't a code block's language, such as mermaid");
  });
});

describe("componentOf", () => {
  it("makes a component name of a folder's", () => {
    expect(componentOf("mermaid")).toBe("mermaid");
    expect(componentOf("packs/My Chart")).toBe("my-chart");
    expect(componentOf("2d_plot")).toBe("d-plot");
    expect(componentOf("---")).toBe("component");
  });
});
