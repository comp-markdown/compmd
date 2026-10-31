// A pack written by `createPack`, as its author gets it (docs/create-pack.md, C4): it builds into one module with no
// import left but the runtimes', its publish check holds, and its test page renders the component in the strict sandbox,
// in Chrome and WebKit. The pack sits in this package's folder, so it finds the repository's packages as installed.
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { rm } from "node:fs/promises";
import { join } from "node:path";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions } from "playwright";
import { build, createServer, type ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { createPack } from "../src/create.ts";

const OUT = join(import.meta.dirname, ".out");

const PACKS = [
  { variant: "a leaf", folder: "badge", name: "@kit/badge", component: "badge", port: 4198, shows: { label: "Hello, from badge", text: "Hello" } },
  { variant: "a code renderer", folder: "diagram", name: "@kit/diagram", component: "diagram", renders: "dot", port: 4199, shows: { label: "Two lines of dot", text: "first line" } },
];

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
    const imports = [...module.matchAll(/\b(?:import|from)\s*["']([^"']+)["']/g)].map((match) => match[1] ?? "");
    expect(imports.filter((specifier) => !/^(react|react-dom|svelte)(\/|$)/.test(specifier))).toEqual([]);
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
