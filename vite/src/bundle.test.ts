import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { cp, mkdtemp, mkdir, readdir, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { compile, hashOf } from "@compmd/sandbox/build";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { loadApp } from "./app.ts";
import { buildBundle, MANIFEST, servedModules, type BuiltBundle } from "./bundle.ts";
import { headersFile } from "./headers.ts";

let root: string;
let out: string;
let built: BuiltBundle;

const LINE_MODULE = "export default function mount(target) { target.textContent = 'line'; }\n";
const BAR = "export default function Bar() { return <svg aria-label=\"bars\" />; }\n";
const DIAL = "<p>dial</p>\n<style>p { color: teal; }</style>\n";

/** `sha384-…` of `text`. */
const sha384 = (text: string): string => `sha384-${createHash("sha384").update(text).digest("base64")}`;

/** An app with a trusted folder pack, a strict pack with a built module, and a denied pack. */
const APP: Record<string, string | object> = {
  "package.json": { name: "app", dependencies: { "@acme/charts": "2.1.0", "@acme/old": "1.0.0" } },
  "compmd.config.json": {
    packs: { "./components": { trust: "trusted", ambient: true }, "@acme/charts": {} },
    deny: [{ pack: "@acme/old", reason: "it is old" }],
  },
  "components/package.json": { name: "@acme/app", version: "1.0.0" },
  "components/index.comp.md": "---\nmd:\n  import:\n    - from: ./gauge.js\n      names: [gauge]\n  export:\n    names: \"*\"\n---\n\n```compmd component=gauge signature\nparams:\n  value: number\n```\n",
  "components/gauge.js": "export default function mount() {}",
  "node_modules/@acme/charts/package.json": { name: "@acme/charts", version: "2.1.0" },
  "node_modules/@acme/charts/index.comp.md": [
    "---\nmd:\n  import:\n    - from: ./dist/line.js\n      names: [line]\n    - from: ./bar.jsx\n      names: [bar]\n    - from: ./dial.svelte\n      names: [dial]",
    "  export:\n    names: [line, bar, dial]\n---\n",
    ...["line", "bar", "dial"].map((name) => `\`\`\`compmd component=${name} signature\nparams:\n  rows: array\n\`\`\`\n`),
  ].join("\n"),
  "node_modules/@acme/charts/bar.jsx": BAR,
  "node_modules/@acme/charts/dial.svelte": DIAL,
  "node_modules/@acme/charts/dist/line.js": LINE_MODULE,
  "node_modules/@acme/charts/README.md": "Charts.",
  "node_modules/@acme/charts/node_modules/dep/index.js": "a dependency of its own",
  "node_modules/@acme/charts/.cache/x": "hidden",
  "node_modules/@acme/old/package.json": { name: "@acme/old", version: "1.0.0" },
  "node_modules/@acme/old/index.comp.md": "---\nmd:\n  export:\n    names: []\n---\n",
  "pnpm-lock.yaml": "lockfileVersion: '9.0'\npackages:\n  '@acme/charts@2.1.0':\n    resolution: {integrity: sha512-charts}\n",
  "out/keep.txt": "the host's own",
  "out/frames/0123456789abcdef/strict.html": "an earlier build",
};

/** Every file under `dir`, by `/`-separated path. */
async function filesUnder(dir: string): Promise<string[]> {
  const entries = await readdir(dir, { recursive: true, withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile())
    .map((entry) => join(entry.parentPath, entry.name).slice(dir.length + 1).split("\\").join("/"))
    .sort();
}

beforeAll(async () => {
  root = join(await mkdtemp(join(tmpdir(), "compmd-bundle-")), "app");
  for (const [path, content] of Object.entries(APP)) {
    const file = join(root, ...path.split("/"));
    await mkdir(dirname(file), { recursive: true });
    await writeFile(file, typeof content === "string" ? content : JSON.stringify(content));
  }
  out = join(root, "out");
  built = await buildBundle({ root, out, source: "compmd://localhost" });
}, 60_000);

afterAll(async () => {
  await rm(dirname(root), { recursive: true, force: true });
});

describe("buildBundle", () => {
  it("writes the frames under their hash, in place of an earlier build, and leaves the host's own files", async () => {
    const { hash } = built.manifest.frames;
    expect(await readdir(join(out, "frames"))).toEqual([hash]);
    expect(await hashOf(join(out, "frames", hash))).toBe(hash);
    expect(await readFile(join(out, "keep.txt"), "utf8")).toBe("the host's own");
    expect(existsSync(join(out, ".staging"))).toBe(false);
  });

  it("names the custom scheme's frames in their policies, and records it", async () => {
    expect(built.manifest.source).toBe("compmd://localhost/");
    expect(await readFile(join(out, "frames", built.manifest.frames.hash, "strict.html"), "utf8")).toContain("compmd://localhost/frames/");
  });

  it("writes each pack's files as published, but its own dependencies, hidden files and denied packs", async () => {
    expect(await filesUnder(join(out, "packs"))).toEqual([
      "@acme/app@1.0.0/gauge.js",
      "@acme/app@1.0.0/index.comp.md",
      "@acme/app@1.0.0/package.json",
      "@acme/charts@2.1.0/README.md",
      "@acme/charts@2.1.0/bar.jsx",
      "@acme/charts@2.1.0/bar.jsx.js",
      "@acme/charts@2.1.0/dial.svelte",
      "@acme/charts@2.1.0/dial.svelte.css",
      "@acme/charts@2.1.0/dial.svelte.js",
      "@acme/charts@2.1.0/dist/line.js",
      "@acme/charts@2.1.0/index.comp.md",
      "@acme/charts@2.1.0/package.json",
    ]);
  });

  it("describes the bundle in its manifest, as compmd-host reads it", async () => {
    const manifest = JSON.parse(await readFile(join(out, MANIFEST), "utf8")) as unknown;
    expect(manifest).toEqual(built.manifest);
    const { packs, libraries, ambient, policy, format } = built.manifest;
    expect(format).toBe(1);
    const hashed = async (path: string) => sha384(await readFile(join(out, "packs/@acme/charts@2.1.0", path), "utf8"));
    const modules = { "dist/line.js": sha384(LINE_MODULE), "bar.jsx.js": await hashed("bar.jsx.js"), "dial.svelte.js": await hashed("dial.svelte.js"), "dial.svelte.css": await hashed("dial.svelte.css") };
    expect(packs).toEqual([
      { pack: "@acme/charts", version: "2.1.0", integrity: "sha512-charts", trust: "strict", modules },
      { pack: "@acme/old", version: "1.0.0", trust: "strict", modules: {} },
      { pack: "@acme/app", version: "1.0.0", trust: "trusted", modules: {} },
    ]);
    expect(libraries).toEqual([]);
    expect(ambient).toEqual(["@acme/app"]);
    expect(policy.deny).toEqual([{ pack: "@acme/old", reason: "it is old" }]);
    expect(policy.runtimes?.map(({ name }) => name)).toEqual(["react", "react-dom", "svelte"]);
  });

  it("compiles a pack's source components beside them, as a frame's compiler would", async () => {
    const at = join(out, "packs/@acme/charts@2.1.0");
    expect(await readFile(join(at, "bar.jsx.js"), "utf8")).toBe((await compile({ name: "bar", language: "jsx", code: BAR })).code);
    const dial = await compile({ name: "dial", language: "svelte", code: DIAL });
    expect(await readFile(join(at, "dial.svelte.js"), "utf8")).toBe(dial.code);
    expect(await readFile(join(at, "dial.svelte.css"), "utf8")).toBe(dial.css);
    // A trusted pack's are the app's bundler's.
    expect(existsSync(join(out, "packs/@acme/app@1.0.0/gauge.js.js"))).toBe(false);
  });

  it("gives the frames each built module by the place the processor names, a compiled one by its source's", async () => {
    const app = await loadApp(root);
    const served = servedModules(app, built.manifest);
    const integrity = (path: string) => built.manifest.packs[0]?.modules[path] ?? "";
    expect(served).toEqual({
      "@acme/charts@2.1.0/dist/line.js": { path: "packs/@acme/charts@2.1.0/dist/line.js", integrity: sha384(LINE_MODULE) },
      "@acme/charts@2.1.0/bar.jsx": { path: "packs/@acme/charts@2.1.0/bar.jsx.js", integrity: integrity("bar.jsx.js") },
      "@acme/charts@2.1.0/dial.svelte": {
        path: "packs/@acme/charts@2.1.0/dial.svelte.js",
        integrity: integrity("dial.svelte.js"),
        css: { path: "packs/@acme/charts@2.1.0/dial.svelte.css", integrity: integrity("dial.svelte.css") },
      },
    });
  });

  it("stops at a pack's component that doesn't compile", async () => {
    const broken = join(dirname(root), "broken");
    await cp(root, broken, { recursive: true });
    await writeFile(join(broken, "node_modules/@acme/charts/bar.jsx"), "export default () => <svg;\n");
    await expect(buildBundle({ root: broken, out: join(broken, "out") })).rejects.toThrow(/^@acme\/charts@2\.1\.0: `bar\.jsx` doesn't compile: /);
  }, 60_000);

  it("writes the contract's headers for a static host", async () => {
    expect(await readFile(join(out, "_headers"), "utf8")).toBe(headersFile("/"));
  });

  it("builds for an http origin without a source", async () => {
    const plain = join(root, "plain");
    const { manifest } = await buildBundle({ root, out: plain });
    expect(manifest.source).toBeUndefined();
    expect(await readFile(join(plain, "frames", manifest.frames.hash, "strict.html"), "utf8")).not.toContain("compmd://");
  }, 60_000);
});
