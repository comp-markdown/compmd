// The build: one file per module, with the runtimes and shared library packs left as imports, and `package.json` made
// to say what the modules need.
import { readdirSync, readFileSync } from "node:fs";
import { rm } from "node:fs/promises";
import { join } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { documentOf, folder, write } from "../test/folders.ts";
import { buildModules } from "./build.ts";
import { checkPack } from "./check.ts";
import { readPack } from "./pack.ts";

let dir: string;

/** The bare and relative imports left in a built module, static and dynamic. */
const importsOf = (module: string): string[] => [...module.matchAll(/\b(?:import|from)\s*\(?\s*["']([^"']+)["']/g)].map((match) => match[1] ?? "");

const manifestIn = (path: string) => JSON.parse(readFileSync(join(dir, path), "utf8")) as { version: string; peerDependencies?: Record<string, string> };

beforeEach(async () => {
  dir = await folder();
  // Libraries as `npm install` would put them: one uses React, and loads a part of itself on demand.
  await write(dir, {
    "node_modules/tiny-lib/package.json": { name: "tiny-lib", version: "2.3.4", type: "module", exports: "./index.js" },
    "node_modules/tiny-lib/index.js": 'import { useState } from "react";\nexport const answer = 42;\nexport const state = useState;\nexport const later = async () => (await import("./later.js")).value;\n',
    "node_modules/tiny-lib/later.js": 'export const value = "later";\n',
    "node_modules/react/package.json": { name: "react", version: "19.3.0" },
    "node_modules/@kit/shared/package.json": { name: "@kit/shared", version: "1.0.0", type: "module", exports: "./index.js" },
    "node_modules/@kit/shared/index.js": "export const shared = 1;\n",
    // A library with a development build, as React's own packages and many others have.
    "node_modules/two-builds/package.json": { name: "two-builds", version: "1.0.0", type: "module", exports: "./index.js" },
    "node_modules/two-builds/index.js": 'export const build = process.env.NODE_ENV === "production" ? "the production build" : "the development build";\n',
  });
});

afterEach(async () => {
  await rm(dir, { recursive: true, force: true });
});

describe("buildModules", () => {
  it("builds each component as one module, its runtimes and shared library packs left as imports, and adds the runtimes as peers", async () => {
    await write(dir, {
      "package.json": { name: "@kit/charts", version: "0.1.0", compmd: "./index.comp.md", files: ["dist", "index.comp.md"], peerDependencies: { "@kit/shared": "^1" } },
      "index.comp.md": documentOf(["dist/line.js", "dist/bar.js"]),
      "src/line.ts": 'import { later, state } from "tiny-lib";\nimport { shared } from "@kit/shared";\nexport default () => [later, state, shared];\n',
      "src/bar.ts": "export default () => undefined;\n",
    });
    await buildModules(await readPack(dir), { logLevel: "silent" });
    expect(readdirSync(join(dir, "dist")).sort()).toEqual(["bar.js", "line.LICENSES.md", "bar.LICENSES.md", "line.js"].sort());
    const line = readFileSync(join(dir, "dist/line.js"), "utf8");
    expect(importsOf(line).sort()).toEqual(["@kit/shared", "react"]);
    // The part `tiny-lib` loads on demand is in the module, in whichever quotes the minifier writes.
    expect(line).toMatch(/["'`]later["'`]/);
    expect(manifestIn("package.json").peerDependencies).toEqual({ "@kit/shared": "^1", react: "^19.3.0" });
    expect(await checkPack(await readPack(dir))).toEqual([]);
  });

  // Vitest runs this with NODE_ENV=test, as a pack's own tests or a CI step may build it.
  it("builds for production whatever NODE_ENV says: no development branches, no development JSX", async () => {
    expect(process.env.NODE_ENV).toBe("test");
    await write(dir, {
      "package.json": { name: "@kit/counter", version: "0.1.0", compmd: "./index.comp.md", files: ["dist", "index.comp.md"] },
      "index.comp.md": documentOf(["dist/counter.js"]),
      "src/counter.tsx": 'import { build } from "two-builds";\nexport const Counter = () => <p>{build}</p>;\nexport default Counter;\n',
    });
    await buildModules(await readPack(dir), { logLevel: "silent" });
    const counter = readFileSync(join(dir, "dist/counter.js"), "utf8");
    expect(counter).toContain("the production build");
    expect(counter).not.toContain("the development build");
    expect(counter).not.toContain("process.env");
    expect(importsOf(counter)).toEqual(["react/jsx-runtime"]);
  });

  it("minifies a module's whitespace as well as its names, with no comment naming where it was built", async () => {
    await write(dir, {
      "package.json": { name: "@kit/sum", version: "0.1.0", compmd: "./index.comp.md", files: ["dist", "index.comp.md"] },
      "index.comp.md": documentOf(["dist/sum.js"]),
      "src/sum.ts": "export default function mount(target: Element, { params }: { params: { values: number[] } }) {\n  let total = 0;\n  for (const value of params.values) {\n    total += value;\n  }\n  target.textContent = String(total);\n  return { destroy: () => target.replaceChildren() };\n}\n",
    });
    await buildModules(await readPack(dir), { logLevel: "silent" });
    const sum = readFileSync(join(dir, "dist/sum.js"), "utf8");
    expect(sum).not.toMatch(/^[\t ]/m);
    expect(sum.trim().split("\n")).toHaveLength(1);
    expect(sum).not.toContain(dir);
    expect(sum).not.toContain("sum.ts");
  });

  it("builds a library pack as one module, and takes its library's version", async () => {
    await write(dir, {
      "package.json": { name: "@kit/tiny", version: "0.0.0", compmd: { library: "dist/index.js" }, files: ["dist"] },
      "src/index.ts": 'export * from "tiny-lib";\n',
    });
    await buildModules(await readPack(dir), { logLevel: "silent" });
    expect(readdirSync(join(dir, "dist")).sort()).toEqual(["LICENSES.md", "index.js"]);
    expect(importsOf(readFileSync(join(dir, "dist/index.js"), "utf8"))).toEqual(["react"]);
    expect(manifestIn("package.json")).toMatchObject({ version: "2.3.4", peerDependencies: { react: "^19.3.0" } });
    expect(await checkPack(await readPack(dir))).toEqual([]);
  });
});
