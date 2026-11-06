// A pack written by `createPack`, as its author gets it (docs/create-pack.md, C4; docs/writing-a-pack.md): it holds no
// build, check or test page of its own, its scripts are `compmd-pack`'s, it builds into one module with no import left
// but the runtimes', the build's check holds, and its test page renders the component in the strict sandbox, in Chrome
// and WebKit, in light and dark. The pack sits in this package's folder, so it finds the repository's packages.
import { RUNTIMES } from "@compmd/sandbox/build";
import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { mkdir, rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { readPack, servePack } from "@compmd/pack";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions, type Page } from "playwright";
import type { ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { createPack } from "../src/create.ts";

const OUT = join(import.meta.dirname, ".out");

/** `compmd-pack`, as a pack's scripts run it. */
const CLI = join(dirname(fileURLToPath(import.meta.resolve("@compmd/pack"))), "cli.js");
const compmdPack = (folder: string, ...args: string[]): string => execFileSync("node", [CLI, ...args], { cwd: folder, encoding: "utf8", stdio: "pipe" });

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
    compmdPack(folder, "test");
    compmdPack(folder, "build");
    server = await servePack(await readPack(folder), { port });
  });

  afterAll(async () => {
    await server.close();
  });

  it("holds only what is its own: no build, check or test page", () => {
    for (const path of ["vite.config.ts", "scripts", "demo"]) expect(existsSync(join(folder, path)), path).toBe(false);
    const { scripts } = JSON.parse(readFileSync(join(folder, "package.json"), "utf8")) as { scripts: Record<string, string> };
    expect(scripts).toMatchObject({ build: "compmd-pack build", dev: "compmd-pack dev", test: "compmd-pack test" });
  });

  it("builds into one module, with no import left but the runtimes'", () => {
    expect(readdirSync(join(folder, "dist")).filter((file) => file.endsWith(".js"))).toEqual([`${options.component}.js`]);
    const module = readFileSync(join(folder, "dist", `${options.component}.js`), "utf8");
    const runtimes = new Set(RUNTIMES.flatMap((runtime) => runtime.imports));
    expect(importsOf(module).filter((specifier) => !runtimes.has(specifier))).toEqual([]);
    // It imports no runtime, so it needs none from the host (spec §6.4).
    expect((JSON.parse(readFileSync(join(folder, "package.json"), "utf8")) as { peerDependencies?: unknown }).peerDependencies).toBeUndefined();
    expect(module).toMatch(/export\s*\{[^}]*as default/);
  });

  it("checks in its build that its document's version is package.json's", () => {
    const manifest = join(folder, "package.json");
    const written = readFileSync(manifest, "utf8");
    writeFileSync(manifest, written.replace('"version": "0.1.0"', '"version": "0.2.0"'));
    try {
      expect(() => compmdPack(folder, "build")).toThrow(/export\.version is 0\.1\.0, package\.json's version is 0\.2\.0/);
    } finally {
      writeFileSync(manifest, written);
    }
  });

  describe.each(BROWSERS)("its test page in %s", (_browser, type, launch) => {
    let browser: Browser;
    // The test page, opened by the first test that needs it and kept: opening it waits for the component to draw.
    // Inside a test, since that wait polls. The last test switches the scheme, and switches it back.
    let opened: Promise<{ page: Page; errors: string[] }> | undefined;
    const testPage = () => (opened ??= open());

    async function open(): Promise<{ page: Page; errors: string[] }> {
      const page = await browser.newPage();
      const errors: string[] = [];
      page.on("pageerror", (error) => errors.push(error.message));
      await page.goto(`http://localhost:${String(port)}/`);
      // Host mode: each use in a frame of its own; the demo's first use is the one checked.
      await expect.poll(() => page.locator("iframe").first().contentFrame().locator(`[aria-label="${shows.label}"]`).count(), { timeout: 20_000 }).toBe(1);
      return { page, errors };
    }

    beforeAll(async () => {
      browser = await type.launch({ headless: true, ...launch });
    });

    afterAll(async () => {
      await browser.close();
    });

    it("renders the component in the strict sandbox, with no diagnostics", async () => {
      const { page, errors } = await testPage();
      const drawn = page.locator("iframe").first().contentFrame().locator(`[aria-label="${shows.label}"]`);
      expect(await drawn.innerText()).toContain(shows.text);
      expect(await page.locator("#diagnostics li").allInnerTexts()).toEqual([]);
      expect(errors).toEqual([]);
    });

    it("makes each use's frame as tall as what it shows: neither cut short nor taller", async () => {
      const { page } = await testPage();
      const frames = page.locator("iframe");
      for (let i = 0; i < (await frames.count()); i += 1) {
        const frame = frames.nth(i);
        // How many pixels the frame is off its content, once it has reported its size. The body's height, which the
        // frame reports; the root's is never less than the frame's, so it hides a frame that is too tall.
        const off = async () => {
          const content = await frame.contentFrame().locator("body").evaluate((body) => body.scrollHeight);
          const height = await frame.evaluate((element) => element.getBoundingClientRect().height);
          return Math.abs(height - content) <= 1 ? 0 : height - content;
        };
        await expect.poll(off, { timeout: 10_000 }).toBe(0);
      }
    });

    it("switches the page between light and dark, and the frame follows", async () => {
      const { page } = await testPage();
      const frame = page.locator("iframe").first().contentFrame();
      const scheme = () => frame.locator(":root").evaluate((root) => getComputedStyle(root).colorScheme);
      await page.selectOption("#scheme", "dark");
      await expect.poll(scheme).toBe("dark");
      await page.selectOption("#scheme", "light");
      await expect.poll(scheme).toBe("light");
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
    compmdPack(folder, "build");
  });

  it("builds into one module, with no chunk, and no import left but the runtimes'", () => {
    expect(readdirSync(join(folder, "dist")).filter((file) => file.endsWith(".js"))).toEqual(["index.js"]);
    const module = readFileSync(join(folder, "dist/index.js"), "utf8");
    expect(importsOf(module)).toEqual(["react"]);
    // The part it loads on demand is in the module, in whichever quotes the minifier writes.
    expect(module).toMatch(/["'`]later["'`]/);
  });

  it("takes the library's version as its own, and its runtimes as peers", () => {
    const manifest = JSON.parse(readFileSync(join(folder, "package.json"), "utf8")) as { version: string; peerDependencies?: Record<string, string>; compmd: unknown };
    expect(manifest.version).toBe("2.3.4");
    expect(manifest.peerDependencies).toEqual({ react: "^19.3.0" });
    expect(manifest.compmd).toEqual({ library: "dist/index.js" });
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
    compmdPack(folder, "build");
    expect(readdirSync(join(folder, "dist")).filter((file) => file.endsWith(".js"))).toEqual(["gauge.js"]);
    expect(importsOf(readFileSync(join(folder, "dist/gauge.js"), "utf8"))).toEqual(["@kit/tiny"]);
  });
});
