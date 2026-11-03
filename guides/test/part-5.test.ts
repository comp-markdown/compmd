// Part V and the appendices of the Developer's Guide against what they claim: the other host's server and page, run
// without the Vite plugin; the configuration's reference against the schema and `readConfig`; and the diagnostics'
// appendix against the processor, each code at the stage the appendix gives it.
import { execFileSync, spawn, type ChildProcess } from "node:child_process";
import { createHash } from "node:crypto";
import { cpSync, readFileSync } from "node:fs";
import { mkdir, rm, writeFile } from "node:fs/promises";
import { createServer, type Server } from "node:http";
import { createRequire } from "node:module";
import { join } from "node:path";
import { execPath } from "node:process";
import { initSync, process as processDocument, type PackRelease, type Policy, type Request } from "@compmd/processor";
import { ConfigError, readConfig } from "@compmd/vite/config";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions } from "playwright";
import { build } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const GUIDES = join(import.meta.dirname, "..");
const BOOK = join(GUIDES, "developers");
const OUT = join(import.meta.dirname, ".out/part-5");
const chapter = (file: string) => readFileSync(join(BOOK, file), "utf8");

beforeAll(() => {
  initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) });
});

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

describe("chapter 18, other hosts", () => {
  const example = join(BOOK, "examples/other-hosts");
  const site = join(OUT, "public");
  const port = 4260;
  const registryPort = 4261;
  const base = `http://localhost:${String(port)}`;
  const tarball = Buffer.from("the tarball's bytes");
  let registry: Server;
  let server: ChildProcess;

  // The chapter's steps: the page bundled into `public/` (by Vite, as a bundler alone), the document beside it, the
  // frames built into `public/compmd/`, then the server, with the configuration's registry a local one.
  beforeAll(async () => {
    await rm(OUT, { recursive: true, force: true });
    await build({ root: example, configFile: false, logLevel: "warn", build: { outDir: site, emptyOutDir: true, target: "es2024" } });
    cpSync(join(example, "plan.comp.md"), join(site, "plan.comp.md"));
    execFileSync(execPath, ["build.ts", join(site, "compmd")], { cwd: example, stdio: "pipe" });
    registry = createServer((request, response) => {
      const path = decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname);
      if (path === "/tarball.tgz") return response.writeHead(200, { "content-type": "application/octet-stream" }).end(tarball);
      if (path !== "/@comp-markdown/mermaid") return response.writeHead(404).end();
      const dist = { tarball: `http://127.0.0.1:${String(registryPort)}/tarball.tgz`, integrity: "sha512-AAAA", unpackedSize: 100 };
      response.writeHead(200, { "content-type": "application/json" }).end(JSON.stringify({ name: "@comp-markdown/mermaid", versions: { "0.1.1": { version: "0.1.1", dist } } }));
    });
    await new Promise<void>((resolve) => registry.listen(registryPort, "127.0.0.1", resolve));
    const config = JSON.parse(readFileSync(join(example, "compmd.config.json"), "utf8")) as Record<string, unknown>;
    const configFile = join(OUT, "compmd.config.json");
    await writeFile(configFile, JSON.stringify({ ...config, registries: { "*": `http://127.0.0.1:${String(registryPort)}/` } }));
    server = spawn(execPath, ["server.ts", site, String(port), configFile], { cwd: example, stdio: ["ignore", "pipe", "pipe"] });
    let errors = "";
    server.stderr?.on("data", (data: Buffer) => (errors += data.toString()));
    await new Promise<void>((resolve, reject) => {
      server.stdout?.on("data", (data: Buffer) => {
        if (data.toString().includes(`${base}/`)) resolve();
      });
      server.on("exit", () => {
        reject(new Error(`the server stopped: ${errors}`));
      });
    });
  });

  afterAll(async () => {
    server.kill();
    registry.closeAllConnections();
    await new Promise((resolve) => registry.close(resolve));
  });

  it("serves the frames with their two headers", async () => {
    const strict = await fetch(`${base}/compmd/strict.html`);
    expect(strict.status).toBe(200);
    expect(strict.headers.get("content-security-policy")).toBe("sandbox allow-scripts");
    expect(strict.headers.get("access-control-allow-origin")).toBe("*");
    const runtime = await fetch(`${base}/compmd/runtime.js`);
    expect(runtime.headers.get("access-control-allow-origin")).toBe("*");
    expect(runtime.headers.get("content-security-policy")).toBeNull();
  });

  it("gives a request that opens the document with the pack given by hand, and the policy the configuration says", async () => {
    const { request, modules } = (await (await fetch(`${base}/app.json`)).json()) as { request: Omit<Request, "path" | "now">; modules: Record<string, string> };
    const text = readFileSync(join(example, "plan.comp.md"), "utf8");
    const opened = processDocument({ ...request, path: "plan.comp.md", files: { "plan.comp.md": text }, now: "2026-11-02T09:00:00Z" });
    expect(opened.diagnostics).toEqual([]);
    expect(request.policy?.runtimes?.map(({ name }) => name)).toEqual(["react", "react-dom", "svelte"]);
    // `imports: listed`: a pack the configuration doesn't list is F026.
    const other = "---\nmd:\n  import:\n    - from: \"@acme/kit@1\"\n      names: [card]\n---\n";
    expect(processDocument({ ...request, path: "a.comp.md", files: { "a.comp.md": other }, now: "2026-11-02T09:00:00Z" }).diagnostics.map(({ code }) => code)).toEqual(["F026"]);
    // The built module the server serves is the file whose hash the page gives its frame.
    const [place, integrity] = Object.entries(modules)[0] ?? [];
    expect(place).toBe("@comp-markdown/mermaid@0.1.1/dist/mermaid.js");
    const served = await fetch(`${base}/compmd/packs/${place ?? ""}`);
    expect(served.headers.get("access-control-allow-origin")).toBe("*");
    expect(integrity).toBe(`sha384-${createHash("sha384").update(Buffer.from(await served.arrayBuffer())).digest("base64")}`);
  });

  it("passes a pack's metadata and tarball on through its proxy, and nothing else", async () => {
    const metadata = (await (await fetch(`${base}/compmd/registry/@comp-markdown/mermaid`)).json()) as { versions: Record<string, { dist: { tarball: string } }> };
    const proxied = metadata.versions["0.1.1"]?.dist.tarball ?? "";
    expect(proxied).toBe(`${base}/compmd/registry/-/tarball/@comp-markdown/mermaid/0.1.1`);
    expect(Buffer.from(await (await fetch(proxied)).arrayBuffer())).toEqual(tarball);
    expect((await fetch(`${base}/compmd/registry/not-a-pack`)).status).toBe(404);
  });

  describe.each(BROWSERS)("in %s", (_name, type, launch) => {
    let browser: Browser;

    beforeAll(async () => {
      browser = await type.launch({ headless: true, ...launch });
    });

    afterAll(async () => {
      await browser.close();
    });

    it("renders the document, its diagram drawn in a strict frame by the module the server serves", async () => {
      const page = await browser.newPage();
      const errors: string[] = [];
      page.on("pageerror", (error) => errors.push(error.message));
      await page.goto(`${base}/`);
      const frame = page.frameLocator("iframe");
      const drawn = frame.locator('[role="img"][aria-label="The server gives the page, the frames and the built modules"] svg');
      await expect.poll(() => drawn.count(), { timeout: 30_000 }).toBe(1);
      expect(await frame.locator("h1").innerText()).toBe("Where a page's parts come from");
      expect(errors).toEqual([]);
      await page.close();
    });
  });
});

describe("chapter 20, the configuration's reference", () => {
  const text = chapter("20-the-configurations-reference.md");
  const schema = JSON.parse(readFileSync(join(GUIDES, "node_modules/@compmd/vite/compmd.config.schema.json"), "utf8")) as {
    properties: Record<string, unknown>;
    $defs: { pack: { properties: Record<string, unknown> } };
  };

  it("has a section for every key of the schema, every pack option and every option of the plugin", () => {
    const sections = new Set([...text.matchAll(/^### `([^`]+)`/gm)].map(([, key]) => key));
    const keys = [...Object.keys(schema.properties).filter((key) => key !== "$schema"), ...Object.keys(schema.$defs.pack.properties).map((key) => `packs.*.${key}`), "folder", "config"];
    expect([...sections].sort()).toEqual(keys.sort());
  });

  it("shows the problems readConfig finds in the wrong configuration, as it writes them", () => {
    const shown = /```json snippet=bad-config\/compmd\.config\.json\n[\s\S]*?```[\s\S]*?```text\n([\s\S]*?)\n```/.exec(text)?.[1];
    const wrong: unknown = JSON.parse(readFileSync(join(BOOK, "examples/bad-config/compmd.config.json"), "utf8"));
    expect(() => readConfig(wrong)).toThrow(ConfigError);
    try {
      readConfig(wrong);
    } catch (error) {
      expect((error as Error).message).toBe(shown);
    }
  });

  it("fills in the defaults the reference gives", () => {
    const read = readConfig({ packs: { "@acme/maps": { trust: "permissive" } }, registries: { "*": "https://registry.npmjs.org" } });
    expect(read.imports).toBe("any");
    expect(read.fetch).toBe("host");
    expect(read.maxPackSize).toBeUndefined();
    expect(read.registries).toEqual({ "*": "https://registry.npmjs.org/" });
    expect(read.packs.get("@acme/maps")).toEqual({ trust: { level: "permissive", network: [], host: false }, ambient: false });
    expect(readConfig({ maxPackSize: "2.5 MB" }).maxPackSize).toBe(2_500_000);
    expect(readConfig({ libraries: ["d3"] }).libraries).toEqual([{ name: "d3", imports: ["d3"] }]);
  });
});

/** A release of a pack, as a host gives it: its manifest, with `manifest`'s fields, and its document. */
function release(name: string, version: string, document: string, manifest: Record<string, unknown> = {}, integrity: string | null = null): PackRelease {
  return { version, integrity, files: { "package.json": JSON.stringify({ name, version, ...manifest }), "index.comp.md": document } };
}

/** A pack's document exporting `card`, with `pack` and `version` in its export. */
const cardPack = (pack: string, version: string) => `---\nmd:\n  export:\n    pack: "${pack}"\n    version: "${version}"\n    names: [card]\n---\n\n\`\`\`compmd component=card\n---\nA card.\n\`\`\`\n`;
const fence = (name: string, body = "N.") => `\`\`\`compmd component=${name}\n---\n${body}\n\`\`\`\n\n`;
const importing = (from: string, names: string) => `---\nmd:\n  import:\n    - from: ${from}\n      names: ${names}\n---\n\n`;
const numbers = `[${Array.from({ length: 400 }, (_, i) => String(i)).join(", ")}]`;

/** For each code the processor reports, a request that makes it: the document's text, and what else it needs. */
type Trigger = Omit<Request, "path" | "now" | "files"> & { text: string; files?: Record<string, string> };
const TRIGGERS: Record<string, Trigger> = {
  F001: { text: "---\nmd:\n  version: 2\n---\n\nText.\n" },
  F002: { text: ":: nope[N]\n" },
  F003: { text: "::: box\nA\n::: /bxo\n" },
  F004: { text: "::: box\nA\n" },
  F005: { text: "::: /box\n" },
  F006: { text: "::: if {when=(true)}\nA\n:: #footer\nB\n::: /if\n" },
  F007: { text: "::: if {when=(true)}\nA\n:: #else\nB\n:: #else\nC\n::: /if\n" },
  F008: { text: "::: box {colour=red}\nA\n::: /box\n" },
  F009: { text: "::: box {border=wiggly}\nA\n::: /box\n" },
  F010: { text: "::: if\nA\n::: /if\n" },
  F011: { text: ':: box[B] {title="open\n' },
  F012: { text: "::: + gap=m\n" },
  F013: { text: fence("note", "A.") + fence("note", "B.") },
  F014: { text: "{{md.doc.nope}}\n" },
  F015: { text: ":: #footer\n" },
  F016: { text: ":::box\n" },
  F017: { text: "::: box {gap=s gap=m}\nA\n::: /box\n" },
  F018: { text: fence("note") + "::: note\nBody.\n::: /note\n" },
  F019: { text: fence("loop", ":: loop[L]") + ":: loop[L]\n" },
  F020: { text: importing("./b.comp.md", '"*"'), files: { "b.comp.md": importing("./doc.comp.md", '"*"') } },
  F021: { text: importing("./b.comp.md", "[other]") + ":: other[O]\n", files: { "b.comp.md": "---\nmd:\n  export:\n    names: [card]\n---\n\n" + fence("card") } },
  F022: { text: importing('"@acme/kit@1"', "[card]") + ":: card[C]\n", packs: { "@acme/kit": [release("@acme/kit", "1.0.0", cardPack("@acme/kit", "2.0.0"))] } },
  F023: { text: importing("./b.comp.md", '"*"').replace("---\n\n", "    - from: ./c.comp.md\n      names: \"*\"\n---\n\n") + ":: card[C]\n", files: { "b.comp.md": "---\nmd:\n  export:\n    names: \"*\"\n---\n\n" + fence("card"), "c.comp.md": "---\nmd:\n  export:\n    names: \"*\"\n---\n\n" + fence("card") } },
  F024: { text: "---\nmd:\n  data:\n    - from: ./items.json\n      as: items\n---\n\nThe sixth: :value[none]{of=(md.data.items[5])}.\n", files: { "items.json": "[1]" } },
  F025: { text: "::: each {in=([1, 2]) as=n join=inline}\nA\n\nB\n::: /each\n" },
  F026: { text: importing('"@acme/none@1"', "[card]") + ":: card[C]\n" },
  F027: { text: "---\nmd:\n  data:\n    - from: ./n.csv\n      as: n\n      schema: {n: number}\n---\n\nText.\n", files: { "n.csv": "n\nx\n" } },
  F028: { text: "---\nmd:\n  data:\n    - from: https://api.example.com/x.json\n      as: x\n---\n\nText.\n" },
  F029: { text: "---\nmd:\n  export:\n    names: [clock]\n---\n\n```jsx component=clock\nexport default function Clock() { return <time />; }\n```\n" },
  F030: { text: ":: number-input[N] {bind=nope}\n" },
  F031: { text: "---\ncity: berlin\nmd:\n  data:\n    - from: https://{{md.doc.city}}.example.com/d.json\n      as: d\n---\n\nText.\n" },
  F032: { text: "::: box\n- item\n\n  ::: /box\n::: /box\n" },
  F033: { text: "---\na: &x 1\n---\n\nText.\n" },
  F034: { text: "---\nmd:\n  data:\n    - from: ./rows.json\n      as: rows\n---\n\n::: each {in=(md.data.rows) as=r}\n::: each {in=(md.data.rows) as=c}\n{{r}}.{{c}}\n::: /each\n::: /each\n", files: { "rows.json": numbers } },
  F035: { text: "---\nmd:\n  import:\n    - from: \"@acme/kit@1.0.0\"\n      integrity: sha512-BBBB\n      names: [card]\n---\n\n:: card[C]\n", packs: { "@acme/kit": [release("@acme/kit", "1.0.0", cardPack("@acme/kit", "1.0.0"), {}, "sha512-AAAA")] } },
  F036: { text: "```compmd component=note signature {padding=s}\nparams: {}\n```\n" },
  F037: { text: "```compmd component=note\nparams:\n  n: numbr\n---\nN.\n```\n" },
  W101: { text: fence("note") + ":: note\n" },
  W102: { text: "```vue component=clock\n<template><time /></template>\n```\n\n```compmd component=clock signature\nparams: {}\n```\n\n:: clock[12:00]\n" },
  W103: { text: "::: each {in=([1, 2]) as=n}\n- {{n}}\n::: /each\n" },
  W105: { text: "```jsx component=clock runtime=react@19\nexport default function Clock() { return <time />; }\n```\n" },
  W106: { text: "{{ 1 + 2 }}\n" },
  W107: { text: ':value[x] {of="y"}\n' },
  W108: { text: "Text.\n::: box\nA\n::: /box\n" },
  W109: { text: "---\nurl: \"javascript:alert(1)\"\n---\n\n[A link]({{md.doc.url}})\n" },
  W110: { text: '---\nmd:\n  version: "1.9"\n---\n\nText.\n' },
  W111: { text: "---\nmd:\n  imports: []\n---\n\nText.\n" },
  W112: { text: fence("row", "R.") + ":: row[R]\n" },
  W113: { text: ":: card[C]\n", ambient: ["@host/c"], reportAmbient: true, packs: { "@host/c": [release("@host/c", "1.0.0", cardPack("@host/c", "1.0.0"))] } },
  W114: { text: '::: row {gap=m}\n::: box\n::: grid {md:grid-template-columns="1fr 1fr"}\nA\n\nB\n::: /grid\n::: /box\n::: /row\n' },
  W116: { text: "<div>\n:: box\n</div>\n" },
  W117: { text: "---\nmd:\n  export:\n    names: [clock]\n---\n\n```jsx component=clock\nexport default function Clock() { return <time />; }\n```\n" },
  W119: { text: "::: box\nA\n\n::: box {inset-block-start=1rem}\nB\n::: /box\n::: /box\n" },
  W120: { text: "```compmd component=tiles {display=contents padding=s}\nslots: [default]\n---\n{{md.slot}}\n```\n\n::: tiles\nText.\n::: /tiles\n" },
  W121: {
    text: importing('"@acme/charts@2"', "[legend]") + ":: legend[L]\n",
    packs: { "@acme/charts": [release("@acme/charts", "2.1.0", "---\nmd:\n  export:\n    pack: \"@acme/charts\"\n    version: \"2.1.0\"\n    names: \"*\"\n---\n\n" + fence("legend"), { peerDependencies: { svelte: "^5.57" } })] },
    policy: { runtimes: [{ name: "svelte", version: "5.40.0" }] } satisfies Policy,
  },
};

describe("chapter 21, the diagnostics", () => {
  const text = chapter("21-the-diagnostics.md");
  /** Each code in the appendix's tables, with the severity and the stage it is listed with. */
  const listed: { code: string; severity: string; stage: string | undefined }[] = [];
  let stage: string | undefined;
  for (const line of text.split("\n")) {
    if (line.startsWith("## ")) stage = /`(parse|resolve|validate|load|evaluate)`/.exec(line)?.[1];
    const row = /^\| ([FW]\d{3}) \| (error|warning) \|/.exec(line);
    if (row?.[1] && row[2]) listed.push({ code: row[1], severity: row[2], stage });
  }
  const catalogue = [...readFileSync(join(GUIDES, "../compmd-spec-v1.md"), "utf8").matchAll(/^\| ([FW]\d{3}) \|/gm)].map(([, code]) => code);

  it("lists every code of the spec's catalogue once, with its severity", () => {
    expect(listed.map(({ code }) => code).sort()).toEqual([...new Set(catalogue)].sort());
    for (const { code, severity } of listed) expect(severity, code).toBe(code.startsWith("F") ? "error" : "warning");
  });

  it.each(listed.filter(({ stage }) => stage !== undefined))("$code: the processor reports it at stage $stage", ({ code, stage }) => {
    const trigger = TRIGGERS[code];
    expect(trigger, `a document that makes ${code}`).toBeDefined();
    const { text: document, files = {}, ...rest } = trigger ?? { text: "" };
    const { diagnostics } = processDocument({ ...rest, path: "doc.comp.md", files: { ...files, "doc.comp.md": document }, now: "2026-11-02T09:00:00Z" });
    const found = diagnostics.find((diagnostic) => diagnostic.code === code);
    expect(found, diagnostics.map(({ code, message }) => `${code} ${message}`).join("\n")).toBeDefined();
    expect(found?.stage).toBe(stage);
  });

  // The other stages the appendix names for a code, in its cause.
  const others: [string, string, Trigger][] = [
    ["F026", "load", { text: "---\nmd:\n  data:\n    - from: ./missing.json\n      as: m\n---\n\nText.\n" }],
    ["F024", "validate", { text: ':value[x]{of=(1 + "a")}\n' }],
    ["F009", "resolve", { text: "---\nmd:\n  import:\n    - from: \"@acme/kit@1\"\n      integrity: sha512-AAAA\n      names: [card]\n---\n\n:: card[C]\n", packs: { "@acme/kit": [release("@acme/kit", "1.0.0", cardPack("@acme/kit", "1.0.0"), {}, "sha512-AAAA")] } }],
    ["F013", "validate", { text: "::: each {in=([1]) as=in}\n{{in}}\n::: /each\n" }],
  ];
  it.each(others)("%s is also found at stage %s", (code, stage, { text: document, files = {}, ...rest }) => {
    const { diagnostics } = processDocument({ ...rest, path: "doc.comp.md", files: { ...files, "doc.comp.md": document }, now: "2026-11-02T09:00:00Z" });
    expect(diagnostics.find((diagnostic) => diagnostic.code === code)?.stage, diagnostics.map(({ code, message }) => `${code} ${message}`).join("\n")).toBe(stage);
  });

  it("reports W113 for names from ambient packs only, not for a host: source, as the appendix says", () => {
    const text = "---\nmd:\n  data:\n    - from: host:tasks\n      as: tasks\n---\n\nText.\n";
    const { diagnostics } = processDocument({ path: "doc.comp.md", files: { "doc.comp.md": text }, host: { tasks: [] }, reportAmbient: true, now: "2026-11-02T09:00:00Z" });
    expect(diagnostics.map(({ code }) => code)).toEqual([]);
  });
});
