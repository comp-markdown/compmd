// The Developer's Guide's Part IV, writing packs, against what its chapters claim: each example pack is copied, built
// and tested with `compmd-pack`, as its author would, and documents that import it, as the chapters show them,
// process with the diagnostics the chapters name.
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync } from "node:fs";
import { cp, rm, symlink } from "node:fs/promises";
import { createRequire } from "node:module";
import { join } from "node:path";
import { env, execPath } from "node:process";
import { initSync, OpenDocument } from "@compmd/processor";
import { beforeAll, describe, expect, it } from "vitest";

const GUIDES = join(import.meta.dirname, "..");
const EXAMPLES = join(GUIDES, "developers/examples");
const OUT = join(import.meta.dirname, ".out/part-4");
const CLI = join(GUIDES, "node_modules/@compmd/pack/dist/cli.js");

const chapter = (file: string) => readFileSync(join(GUIDES, "developers", file), "utf8");

beforeAll(() => {
  initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) });
});

/** A copy of the example pack `name`, so what the build writes stays out of the examples; built when `build` says. */
async function copyOf(name: string): Promise<string> {
  const copy = join(OUT, name);
  await rm(copy, { recursive: true, force: true });
  await cp(join(EXAMPLES, name), copy, { recursive: true });
  return copy;
}

/**
 * The environment of an author's shell: without the `NODE_ENV` the test runner sets, `test`, with which Vite would
 * build JSX for development, with React's development runtime bundled in.
 */
const AUTHORS = Object.fromEntries(Object.entries(env).filter(([name]) => name !== "NODE_ENV"));

/** `compmd-pack <command>` in `folder`, as the pack's scripts run it. */
const compmdPack = (folder: string, command: string) => execFileSync(execPath, [CLI, command], { cwd: folder, stdio: "pipe", env: AUTHORS });

/** The built pack in `folder`, as a host reads it from npm. */
function packIn(folder: string): { name: string; release: { version: string; integrity: string; files: Record<string, string> } } {
  const file = (path: string) => readFileSync(join(folder, path), "utf8");
  const { name, version } = JSON.parse(file("package.json")) as { name: string; version: string };
  const files: Record<string, string> = { "package.json": file("package.json"), "index.comp.md": file("index.comp.md") };
  for (const built of readdirSync(join(folder, "dist")).filter((path) => path.endsWith(".js"))) files[`dist/${built}`] = file(`dist/${built}`);
  return { name, release: { version, integrity: "sha512-from-the-build", files } };
}

/** `text` opened with the pack in `folder`. */
function open(folder: string, text: string, inputs: Record<string, unknown> = {}): OpenDocument {
  const { name, release } = packIn(folder);
  return new OpenDocument({ path: "doc.comp.md", files: { "doc.comp.md": text }, packs: { [name]: [release] }, inputs, now: "2026-10-31T09:00:00Z" });
}

const codesOf = (opened: OpenDocument) => opened.diagnostics.map(({ code }) => code);

/** Every node of an expanded tree, depth first. */
function nodesOf(tree: unknown): Record<string, unknown>[] {
  if (Array.isArray(tree)) return tree.flatMap(nodesOf);
  if (typeof tree !== "object" || tree === null) return [];
  const node = tree as Record<string, unknown>;
  return [node, ...Object.values(node).flatMap(nodesOf)];
}

/** The params of the first node named `name` of the opened document's tree, by key. */
function paramsOf(opened: OpenDocument, type: string, name: string): Record<string, unknown> {
  const node = nodesOf(opened.expanded).find((found) => found.type === type && (found.name === name || (found.component as { name?: string } | undefined)?.name === name));
  const params = (node?.params ?? (node?.component as { params?: unknown } | undefined)?.params ?? []) as { key: string; value: unknown; input?: string }[];
  return Object.fromEntries(params.map(({ key, value, input }) => [key, input === undefined ? value : { value, input }]));
}

describe("13, your first pack", () => {
  let folder: string;
  const demo = '---\nmd:\n  import:\n    - from: "@you/hello@0"\n      names: [hello]\n---\n\n# hello\n\n:: hello[Hello, from hello] {value="Hello"}\n';

  beforeAll(async () => {
    folder = await copyOf("first-pack");
  });

  it("builds into one module and passes its own tests", () => {
    compmdPack(folder, "build");
    expect(readdirSync(join(folder, "dist")).filter((path) => path.endsWith(".js"))).toEqual(["hello.js"]);
    compmdPack(folder, "test");
  });

  it("is the pack its test page renders, with no diagnostics", () => {
    expect(chapter("13-your-first-pack.md")).toContain(demo);
    const opened = open(folder, demo);
    expect(codesOf(opened)).toEqual([]);
    expect(opened.components.map(({ name, module }) => [name, module?.path])).toEqual([["hello", "dist/hello.js"]]);
    opened.free();
  });
});

describe("14, signatures and params", () => {
  let folder: string;
  const used =
    '---\nmd:\n  import:\n    - from: "@you/rating@0"\n      names: [rating, lanes]\n  inputs:\n    stars: number = 3\ntasks:\n  - {title: Write the outline, status: doing}\n  - {title: Ask for a review, status: waiting}\n  - {title: Draft the first part, status: doing}\n---\n\n:: rating[3 of 5 stars] {value=stars size=large}\n\n::: lanes {rows=(md.doc.tasks) by=status as=t}\n**{{t.title}}**\n::: /lanes\n';
  const mistaken =
    '---\nmd:\n  import:\n    - from: "@you/rating@0"\n      names: [rating]\n  inputs:\n    stars: number = 3\n    note: string = ""\n---\n\n:: rating[Stars] {value=stars colour=gold}\n\n:: rating[Stars] {value=stars max=many}\n\n:: rating[Stars] {value=stars size=huge}\n\n:: rating[Stars] {max=4}\n\n:: rating[Stars] {value=note}\n';

  beforeAll(async () => {
    folder = await copyOf("rating-pack");
    compmdPack(folder, "build");
  });

  it("builds a module per component and passes its tests", () => {
    expect(readdirSync(join(folder, "dist")).filter((path) => path.endsWith(".js")).sort()).toEqual(["lanes.js", "rating.js"]);
    compmdPack(folder, "test");
  });

  it("gives a component its params typed, with its defaults, a bound input, and a template slot per record", () => {
    expect(chapter("14-signatures-and-params.md")).toContain(used);
    const opened = open(folder, used, { stars: 4 });
    expect(codesOf(opened)).toEqual([]);
    expect(paramsOf(opened, "componentLeaf", "rating")).toEqual({ value: { value: 4, input: "stars" }, size: "large", max: 5 });
    const lanes = nodesOf(opened.expanded).find((node) => node.type === "componentContainer" && node.name === "lanes");
    expect((lanes?.instances as unknown[] | undefined)?.length).toBe(3);
    opened.free();
  });

  it("reports each mistake in a use as the chapter says, before anything runs", () => {
    expect(chapter("14-signatures-and-params.md")).toContain(mistaken);
    const opened = open(folder, mistaken);
    expect(codesOf(opened)).toEqual(["F008", "F009", "F009", "F010", "F030"]);
    opened.free();
  });
});

describe("15, React, Svelte and libraries", () => {
  let folder: string;
  const used = '---\nmd:\n  import:\n    - from: "@you/sparkline@0"\n      names: [sparkline]\n---\n\n:: sparkline[Visits rose from 3 to 12] {values=[3, 5, 4, 8, 12]}\n';

  beforeAll(async () => {
    folder = await copyOf("sparkline-pack");
    compmdPack(folder, "build");
  });

  it("builds into one module with nothing left to import, and passes its tests", () => {
    const module = readFileSync(join(folder, "dist/sparkline.js"), "utf8");
    expect([...module.matchAll(/\b(?:import|from)\s*\(?\s*["']([^"']+)["']/g)]).toEqual([]);
    expect((JSON.parse(readFileSync(join(folder, "package.json"), "utf8")) as { peerDependencies?: unknown }).peerDependencies).toBeUndefined();
    compmdPack(folder, "test");
  });

  it("is used with a list of numbers, and no diagnostics", () => {
    expect(chapter("15-react-svelte-and-libraries.md")).toContain(used);
    const opened = open(folder, used);
    expect(codesOf(opened)).toEqual([]);
    expect(paramsOf(opened, "componentLeaf", "sparkline")).toEqual({ values: [3, 5, 4, 8, 12] });
    opened.free();
  });
});

describe("15, React inside mount", () => {
  let folder: string;
  const used = '---\nmd:\n  import:\n    - from: "@you/counter@0"\n      names: [counter]\n---\n\n:: counter[Cups of tea: 0] {label="Cups of tea"}\n';

  beforeAll(async () => {
    folder = await copyOf("counter-pack");
    // What `npm install` gives the pack: the build reads the version of each runtime it leaves out in its `node_modules`.
    await symlink(join(GUIDES, "node_modules"), join(folder, "node_modules"), "dir");
    compmdPack(folder, "build");
  });

  it("builds a module that leaves React to the frames, adds React to peerDependencies, and passes its tests", () => {
    const module = readFileSync(join(folder, "dist/counter.js"), "utf8");
    const imported = new Set([...module.matchAll(/\b(?:import|from)\s*\(?\s*["']([^"']+)["']/g)].map(([, name]) => name));
    expect([...imported].sort()).toEqual(["react", "react-dom", "react-dom/client", "react/jsx-runtime"]);
    expect((JSON.parse(readFileSync(join(folder, "package.json"), "utf8")) as { peerDependencies?: unknown }).peerDependencies).toEqual({ react: "^19.3.0", "react-dom": "^19.3.0" });
    compmdPack(folder, "test");
  });

  it("is used with a label, and no diagnostics", () => {
    expect(chapter("15-react-svelte-and-libraries.md")).toContain(used);
    const opened = open(folder, used);
    expect(codesOf(opened)).toEqual([]);
    expect(paramsOf(opened, "componentLeaf", "counter")).toEqual({ label: "Cups of tea", start: 0 });
    opened.free();
  });
});

describe("16, code renderers", () => {
  let folder: string;
  const used =
    '---\nmd:\n  import:\n    - from: "@you/csv@0"\n      names: [csv]\n---\n\n# Visits\n\n```csv {alt="Visits by city: Kyoto 12, Osaka 7"}\ncity,visits\nKyoto,12\nOsaka,7\n```\n\n:: csv[Open and done] {code=("open,3\\ndone,9") header=false}\n';
  const mistaken =
    '---\nmd:\n  import:\n    - from: "@you/csv@0"\n      names: [csv]\n---\n\n```csv\ncity,visits\n```\n\n```csv {alt="Visits" code="a,b"}\ncity,visits\n```\n\n```csv {alt="Visits" header=maybe}\ncity,visits\n```\n';

  beforeAll(async () => {
    folder = await copyOf("csv-pack");
    compmdPack(folder, "build");
  });

  it("builds and passes its tests", () => {
    expect(readdirSync(join(folder, "dist")).filter((path) => path.endsWith(".js"))).toEqual(["csv.js"]);
    compmdPack(folder, "test");
  });

  it("renders a block with its text as `code` and its alt as the fallback's text, and is a leaf too", () => {
    expect(chapter("16-code-renderers.md")).toContain(used);
    const opened = open(folder, used);
    expect(codesOf(opened)).toEqual([]);
    const block = nodesOf(opened.expanded).find((node) => node.type === "code");
    expect(block?.alt).toBe("Visits by city: Kyoto 12, Osaka 7");
    expect(paramsOf(opened, "code", "csv")).toEqual({ code: "city,visits\nKyoto,12\nOsaka,7", header: true });
    expect(paramsOf(opened, "componentLeaf", "csv")).toEqual({ code: "open,3\ndone,9", header: false });
    opened.free();
  });

  it("reports a block without alt, a code param on the info string, and a value outside its type", () => {
    expect(chapter("16-code-renderers.md")).toContain(mistaken);
    const opened = open(folder, mistaken);
    expect(codesOf(opened).sort()).toEqual(["F009", "F017", "W101"]);
    opened.free();
  });
});
