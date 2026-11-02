// A pack written by `createPack`, as its author gets it (docs/create-pack.md, C4): it builds into one module with no
// import left but the runtimes', its publish check holds, and its test page renders the component in the strict sandbox,
// in Chrome and WebKit. The pack sits in this package's folder, so it finds the repository's packages as installed.
import { RUNTIMES } from "@compmd/sandbox/build";
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { mkdir, rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions } from "playwright";
import { build, createServer, type ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { createPack } from "../src/create.ts";

const OUT = join(import.meta.dirname, ".out");

const PACKS = [
  { variant: "a leaf", folder: "badge", name: "@kit/badge", component: "badge", port: 4198, shows: { label: "Hello, from badge", text: "Hello" } },
  { variant: "a code renderer", folder: "diagram", name: "@kit/diagram", component: "diagram", renders: "dot", port: 4199, shows: { label: "Two lines of dot", text: "first line" } },
];

/** The bare and relative imports left in a built module, static and dynamic. */
const importsOf = (module: string): string[] => [...module.matchAll(/\b(?:import|from)\s*\(?\s*["']([^"']+)["']/g)].map((match) => match[1] ?? "");

/** Writes `files` under `folder`, by `/`-separated path. */
async function write(folder: string, files: Record<string, string | object>): Promise<void> {
  for (const [path, content] of Object.entries(files)) {
    const file = join(folder, ...path.split("/"));
    await mkdir(dirname(file), { recursive: true });
    await writeFile(file, typeof content === "string" ? content : JSON.stringify(content));
  }
}

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

describe.each(PACKS)("a pack of $variant", ({ folder: name, port, shows, ...options }) => {
  const folder = join(OUT, name);
  let server: ViteDevServer;

  beforeAll(async () => {
    await rm(folder, { recursive: true, force: true });
    await createPack({ folder, ...options });
    // Its own type check and tests pass as written, before anything is built.
    execFileSync(join(import.meta.dirname, "../node_modules/.bin/tsc"), ["--noEmit"], { cwd: folder, stdio: "pipe" });
    execFileSync(join(import.meta.dirname, "../node_modules/.bin/vitest"), ["run"], { cwd: folder, stdio: "pipe" });
    // The pack's config sets its own roots: the pack for the build, `demo/` for the test page.
    await build({ configFile: join(folder, "vite.config.ts"), logLevel: "warn" });
    server = await createServer({ configFile: join(folder, "vite.config.ts"), logLevel: "warn", server: { port, strictPort: true } });
    await server.listen();
  });

  afterAll(async () => {
    await server.close();
  });

  it("builds into one module, with no import left but the runtimes'", () => {
    expect(readdirSync(join(folder, "dist"))).toEqual([`${options.component}.js`]);
    const module = readFileSync(join(folder, "dist", `${options.component}.js`), "utf8");
    const imports = importsOf(module);
    const runtimes = new Set(RUNTIMES.flatMap((runtime) => runtime.imports));
    expect(imports.filter((specifier) => !runtimes.has(specifier))).toEqual([]);
    // It imports no runtime, so it needs none from the host (spec §6.4).
    expect((JSON.parse(readFileSync(join(folder, "package.json"), "utf8")) as { peerDependencies?: unknown }).peerDependencies).toBeUndefined();
    expect(module).toMatch(/export\s*\{[^}]*as default/);
  });

  it("checks before publishing that its document's version is package.json's", () => {
    expect(() => execFileSync("node", ["scripts/check.js"], { cwd: folder, stdio: "pipe" })).not.toThrow();
    const manifest = join(folder, "package.json");
    const written = readFileSync(manifest, "utf8");
    writeFileSync(manifest, written.replace('"version": "0.1.0"', '"version": "0.2.0"'));
    try {
      expect(() => execFileSync("node", ["scripts/check.js"], { cwd: folder, stdio: "pipe" })).toThrow(/export\.version is 0\.1\.0, package\.json's version is 0\.2\.0/);
    } finally {
      writeFileSync(manifest, written);
    }
  });

  describe.each(BROWSERS)("its test page in %s", (_browser, type, launch) => {
    let browser: Browser;

    beforeAll(async () => {
      browser = await type.launch({ headless: true, ...launch });
    });

    afterAll(async () => {
      await browser.close();
    });

    it("renders the component in the strict sandbox, with no diagnostics", async () => {
      const page = await browser.newPage();
      const errors: string[] = [];
      page.on("pageerror", (error) => errors.push(error.message));
      await page.goto(`http://localhost:${String(port)}/`);
      const drawn = page.frameLocator("iframe").locator(`[aria-label="${shows.label}"]`);
      await expect.poll(() => drawn.count(), { timeout: 20_000 }).toBe(1);
      expect(await drawn.innerText()).toContain(shows.text);
      expect(await page.locator("#diagnostics li").allInnerTexts()).toEqual([]);
      expect(errors).toEqual([]);
      await page.close();
    });
  });
});

describe("a library pack", () => {
  const folder = join(OUT, "tiny");

  beforeAll(async () => {
    await rm(folder, { recursive: true, force: true });
    await createPack({ folder, name: "@kit/tiny", component: "tiny", library: "tiny-lib" });
    // The library it wraps, as `npm install` would put it: it uses React, and loads a part of itself on demand.
    await write(folder, {
      "node_modules/tiny-lib/package.json": { name: "tiny-lib", version: "2.3.4", type: "module", exports: "./index.js" },
      "node_modules/tiny-lib/index.js": 'import { useState } from "react";\nexport const answer = 42;\nexport const state = useState;\nexport const later = async () => (await import("./later.js")).value;\n',
      "node_modules/tiny-lib/later.js": 'export const value = "later";\n',
      "node_modules/react/package.json": { name: "react", version: "19.3.0" },
    });
    await build({ configFile: join(folder, "vite.config.ts"), logLevel: "silent" });
  });

  it("builds into one module, with no chunk, and no import left but the runtimes'", () => {
    expect(readdirSync(join(folder, "dist"))).toEqual(["index.js"]);
    const module = readFileSync(join(folder, "dist/index.js"), "utf8");
    expect(importsOf(module)).toEqual(["react"]);
    expect(module).toContain('"later"');
  });

  it("takes the library's version as its own, and its runtimes as peers", () => {
    const manifest = JSON.parse(readFileSync(join(folder, "package.json"), "utf8")) as { version: string; peerDependencies?: Record<string, string>; compmd: unknown };
    expect(manifest.version).toBe("2.3.4");
    expect(manifest.peerDependencies).toEqual({ react: "^19.3.0" });
    expect(manifest.compmd).toEqual({ library: "dist/index.js" });
  });

  it("checks before publishing that its version is the library's", () => {
    expect(() => execFileSync("node", ["scripts/check.js"], { cwd: folder, stdio: "pipe" })).not.toThrow();
    const manifest = join(folder, "package.json");
    const written = readFileSync(manifest, "utf8");
    writeFileSync(manifest, written.replace('"version": "2.3.4"', '"version": "2.4.0"'));
    try {
      expect(() => execFileSync("node", ["scripts/check.js"], { cwd: folder, stdio: "pipe" })).toThrow(/package\.json's version is 2\.4\.0, tiny-lib's is 2\.3\.4/);
    } finally {
      writeFileSync(manifest, written);
    }
  });
});

describe("a pack that shares a library pack", () => {
  it("leaves the library it names in peerDependencies out of its module", async () => {
    const folder = join(OUT, "gauge");
    await rm(folder, { recursive: true, force: true });
    await createPack({ folder, name: "@kit/gauge", component: "gauge" });
    const manifest = JSON.parse(readFileSync(join(folder, "package.json"), "utf8")) as object;
    writeFileSync(join(folder, "package.json"), JSON.stringify({ ...manifest, peerDependencies: { "@kit/tiny": "^2.3" } }));
    const source = join(folder, "src/gauge.ts");
    writeFileSync(source, `import { answer } from "@kit/tiny";\nexport const shared = answer;\n${readFileSync(source, "utf8")}`);
    await build({ configFile: join(folder, "vite.config.ts"), logLevel: "silent" });
    expect(readdirSync(join(folder, "dist"))).toEqual(["gauge.js"]);
    expect(importsOf(readFileSync(join(folder, "dist/gauge.js"), "utf8"))).toEqual(["@kit/tiny"]);
  });
});
