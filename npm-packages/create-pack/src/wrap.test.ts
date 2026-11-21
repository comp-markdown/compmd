// Writing a pack of an npm package: the package installed and pinned, its kind read, the glue for it, the allowlist, the
// document's generated signature and the demo; a plain library's skeleton; and what the options can't make. The
// install step writes the packages, as `npm install` would put them.
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { mkdtemp, mkdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { CreateError } from "./create.ts";
import { createWrappedPack, packageSpec, type WrappedPackOptions } from "./wrap.ts";

let dir: string;

beforeEach(async () => {
  dir = await mkdtemp(join(tmpdir(), "compmd-create-pack-"));
});

afterEach(async () => {
  await rm(dir, { recursive: true, force: true });
});

/** An install that puts `files` under the pack's folder, at once as `npm install` does, and counts how often it runs. */
function installing(files: Record<string, string | object>): { install: (folder: string) => void; runs: () => number } {
  let runs = 0;
  return {
    install: (folder) => {
      runs += 1;
      for (const [path, content] of Object.entries(files)) {
        const file = join(folder, ...path.split("/"));
        mkdirSync(dirname(file), { recursive: true });
        writeFileSync(file, typeof content === "string" ? content : JSON.stringify(content));
      }
    },
    runs: () => runs,
  };
}

const VIEWER = {
  "node_modules/viewer/package.json": { name: "viewer", version: "4.0.0", types: "index.d.ts", module: "index.js" },
  "node_modules/viewer/index.d.ts": 'export declare class Viewer extends HTMLElement { src: string; autoRotate: boolean }\ndeclare global { interface HTMLElementTagNameMap { "x-viewer": Viewer } }\n',
  "node_modules/viewer/index.js": 'new Worker("decoder.js");\ncustomElements.define("x-viewer", class extends HTMLElement {});\n',
};

const options = (folder: string, more: Partial<WrappedPackOptions>): WrappedPackOptions => ({ folder: join(dir, folder), name: `@kit/${folder}`, component: folder, from: "viewer", ...more });

describe("createWrappedPack", () => {
  it("wraps a web component by its only element, pins the package, and says what a strict frame would refuse", async () => {
    const { install, runs } = installing(VIEWER);
    const { written, kind, notes } = await createWrappedPack(options("viewer", { from: "viewer@^4", install }));
    expect(kind).toBe("web-component");
    expect(runs()).toBe(1);
    expect(written).toEqual([".gitignore", "README.md", "compmd.wrap.json", "demo.comp.md", "index.comp.md", "package.json", "src/viewer.test.ts", "src/viewer.ts", "tsconfig.json"]);
    const folder = join(dir, "viewer");
    const manifest = JSON.parse(await readFile(join(folder, "package.json"), "utf8")) as { dependencies: Record<string, string>; description: string };
    expect(manifest.dependencies).toEqual({ viewer: "4.0.0" });
    expect(manifest.description).toBe("A CompMD pack of viewer 4.0.0: viewer.");
    expect(JSON.parse(await readFile(join(folder, "compmd.wrap.json"), "utf8"))).toEqual({ from: "viewer", export: "x-viewer", kind: "web-component", params: ["src", "autoRotate"] });
    expect(await readFile(join(folder, "src/viewer.ts"), "utf8")).toContain('document.createElement("x-viewer")');
    expect(await readFile(join(folder, "index.comp.md"), "utf8")).toContain("```compmd component=viewer signature\nparams:\n  src: string\n  autoRotate: boolean\n```\n<!-- /compmd-pack:generated -->");
    expect(await readFile(join(folder, "demo.comp.md"), "utf8")).toContain(':: viewer[viewer, as text] {src="Hello" autoRotate=true}');
    expect(await readFile(join(folder, "src/viewer.test.ts"), "utf8")).toContain('params: {"src":"Hello","autoRotate":true}');
    expect(notes).toEqual(["viewer's index.js has a worker, which a strict frame can't start: `new Worker(`"]);
    const all = await Promise.all(written.map((path) => readFile(join(folder, path), "utf8")));
    expect(all.join("\n")).not.toMatch(/__[a-z_]+(?::[^_]*)?__/);
  });

  it("adds the development packages a kind's glue needs, and installs again for them", async () => {
    const { install, runs } = installing({
      "node_modules/qr/package.json": { name: "qr", version: "2.2.0", types: "index.d.ts", peerDependencies: { react: "^19" } },
      "node_modules/qr/index.d.ts": "export declare const Qr: (props: { value: string; title?: string }) => unknown;\n",
      "node_modules/react/package.json": { name: "react", version: "19.3.0", types: "index.d.ts" },
      "node_modules/react/index.d.ts": "export type ComponentProps<T> = T extends (props: infer P) => unknown ? P : never;\n",
    });
    const { kind } = await createWrappedPack(options("qr", { from: "qr", export: "Qr", install }));
    expect(kind).toBe("react");
    expect(runs()).toBe(2);
    const folder = join(dir, "qr");
    const manifest = JSON.parse(await readFile(join(folder, "package.json"), "utf8")) as { devDependencies: Record<string, string> };
    expect(Object.keys(manifest.devDependencies)).toEqual(expect.arrayContaining(["react", "react-dom", "@types/react", "@types/react-dom"]));
    const glue = await readFile(join(folder, "src/qr.ts"), "utf8");
    expect(glue).toContain('import { Qr as Wrapped } from "qr";');
    expect(glue).toContain('({ ...next, title: next["title"] ?? context.fallback.text })');
    expect(JSON.parse(await readFile(join(folder, "compmd.wrap.json"), "utf8"))).toMatchObject({ export: "Qr", params: ["value", "title"], label: "title" });
  });

  it("writes a plain library's draw skeleton, with a signature that is the author's", async () => {
    const { install } = installing({ "node_modules/plain/package.json": { name: "plain", version: "1.5.4", type: "module", exports: "./index.js" } });
    const { kind } = await createWrappedPack(options("plain", { from: "plain", install }));
    expect(kind).toBe("library");
    const folder = join(dir, "plain");
    expect(await readFile(join(folder, "src/plain.ts"), "utf8")).toContain('import * as library from "plain";');
    const document = await readFile(join(folder, "index.comp.md"), "utf8");
    expect(document).toContain("```compmd component=plain signature\nparams:\n  value: string\n```");
    expect(document).not.toContain("compmd-pack:generated");
  });

  it("refuses what can't make a pack: a name, a package, a folder with files, no element, and a package npm didn't install", async () => {
    const refused = (more: Partial<WrappedPackOptions>, folder = "x") => createWrappedPack(options(folder, { install: installing(VIEWER).install, ...more }));
    await expect(refused({ name: "kit" })).rejects.toThrow(CreateError);
    await expect(refused({ from: "Not A Package" })).rejects.toThrow("Not A Package isn't an npm package's name");
    await mkdir(join(dir, "full"), { recursive: true });
    await writeFile(join(dir, "full", "a.txt"), "");
    await expect(refused({}, "full")).rejects.toThrow("has files in it");
    await expect(refused({ from: "viewer", kind: "web-component", install: installing({ ...VIEWER, "node_modules/viewer/index.d.ts": "export {};\n" }).install })).rejects.toThrow("viewer defines no custom element in its types: give its tag with --export");
    await expect(refused({ from: "absent", install: () => undefined }, "y")).rejects.toThrow("absent isn't installed");
    expect(existsSync(join(dir, "y", "package.json"))).toBe(true);
  });
});

describe("packageSpec", () => {
  it("reads a package's name and version, scoped or not", () => {
    expect(["react-qr-code@2.2.0", "react-qr-code", "@google/model-viewer@4.4.0", "@google/model-viewer", "x@"].map(packageSpec)).toEqual([
      { name: "react-qr-code", version: "2.2.0" },
      { name: "react-qr-code", version: undefined },
      { name: "@google/model-viewer", version: "4.4.0" },
      { name: "@google/model-viewer", version: undefined },
      { name: "x", version: undefined },
    ]);
  });
});
