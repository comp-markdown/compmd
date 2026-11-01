// The example app, built with the plugin and served as a host serves it, in Chrome and WebKit: the app's own `stars`
// mounts in its page and sets its input, the strict pack's built module runs in a frame, checked against its hash, and
// the denied pack's use shows its fallback, with the reason.
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions } from "playwright";
import { build, createServer, preview, type PreviewServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { compmd } from "../src/plugin.ts";

const EXAMPLE = join(import.meta.dirname, "../example");
const OUT = join(import.meta.dirname, ".out/example");
const PORT = 4196;
const URL_ = `http://localhost:${String(PORT)}/`;

let server: PreviewServer;

beforeAll(async () => {
  const config = { root: EXAMPLE, configFile: false as const, logLevel: "warn" as const, plugins: [compmd()], build: { outDir: OUT, emptyOutDir: true, target: "es2024" } };
  await build(config);
  server = await preview({ ...config, preview: { port: PORT, strictPort: true } });
}, 120_000);

afterAll(async () => {
  await server.close();
});

describe("the example's build", () => {
  it("has the frame documents and the built module, with the hash the app gives", () => {
    expect(existsSync(join(OUT, "compmd/strict.html"))).toBe(true);
    expect(existsSync(join(OUT, "compmd/permissive.html"))).toBe(true);
    const module = readFileSync(join(OUT, "compmd/packs/@example/charts@1.0.0/dist/bar.js"));
    const hash = `sha384-${createHash("sha384").update(module).digest("base64")}`;
    const bundle = readFileSync(join(OUT, "assets", readFileSync(join(OUT, "index.html"), "utf8").match(/assets\/(index-[^"]+\.js)/)?.[1] ?? ""), "utf8");
    expect(bundle).toContain(hash);
    // The frames' runtimes, in the request's policy too, so the processor checks against them (spec §7.6).
    const definition = JSON.parse(readFileSync(join(OUT, "compmd/library-definition.json"), "utf8")) as { runtimes: { name: string; version: string }[] };
    expect(definition.runtimes.map(({ name }) => name)).toEqual(["react", "react-dom", "svelte"]);
    const version = (definition.runtimes[0]?.version ?? "").replaceAll(".", "\\.");
    // Minified, the request's JSON is an object literal: `runtimes:[{name:"react",version:"19.3.0"}`, in any quotes.
    expect(bundle).toMatch(new RegExp(`runtimes:\\[\\{name:["'\`]react["'\`],version:["'\`]${version}["'\`]\\}`));
    // The denied pack's code is nowhere: it is never served, and its uses fall back.
    expect(existsSync(join(OUT, "compmd/packs/@example/old@1.0.0"))).toBe(false);
  });
});

describe("the example on the dev server", () => {
  it("serves the frames and the built module, and gives the page its packs", async () => {
    const dev = await createServer({ root: EXAMPLE, configFile: false, logLevel: "warn", plugins: [compmd()], server: { port: PORT + 1, strictPort: true } });
    await dev.listen();
    const browser = await chromium.launch({ headless: true, channel: "chrome" });
    try {
      const page = await browser.newPage();
      await page.goto(`http://localhost:${String(PORT + 1)}/`);
      await expect.poll(() => page.getByRole("radiogroup", { name: "3 of 5" }).count(), { timeout: 15_000 }).toBe(1);
      const bars = page.frameLocator("iframe").locator('svg[aria-label="Pages read each day: 30, 52, 18, 41"] rect');
      await expect.poll(() => bars.count(), { timeout: 15_000 }).toBe(4);
      const response = await fetch(`http://localhost:${String(PORT + 1)}/compmd/strict.html`);
      expect(response.headers.get("access-control-allow-origin")).toBe("*");
      expect(response.headers.get("content-security-policy")).toBe("sandbox allow-scripts");
      expect((await fetch(`http://localhost:${String(PORT + 1)}/compmd/packs/@example/old@1.0.0/index.comp.md`)).headers.get("access-control-allow-origin")).toBeNull();
    } finally {
      await browser.close();
      await dev.close();
    }
  }, 60_000);
});

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

describe.each(BROWSERS)("the example in %s", (_name, type, options) => {
  let browser: Browser;

  beforeAll(async () => {
    browser = await type.launch({ headless: true, ...options });
  });

  afterAll(async () => {
    await browser.close();
  });

  it("mounts stars in the page, runs the built module in a frame, and falls back for the denied pack", async () => {
    const page = await browser.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto(URL_);

    const stars = page.getByRole("radiogroup", { name: "3 of 5" });
    await expect.poll(() => stars.count()).toBe(1);
    await expect.poll(() => page.getByRole("radio", { name: "3 of 5" }).getAttribute("aria-checked")).toBe("true");
    await page.getByRole("radio", { name: "5 of 5" }).click();
    await expect.poll(() => page.locator("main").innerText()).toContain("How good was it? 5 of 5.");

    const bars = page.frameLocator("iframe").locator('svg[aria-label="Pages read each day: 30, 52, 18, 41"] rect');
    await expect.poll(() => bars.count(), { timeout: 10_000 }).toBe(4);

    await expect.poll(() => page.locator("main").innerText()).toContain("An old gauge: 40");
    // The denied import, and nothing else.
    expect(await page.locator("#diagnostics li").allInnerTexts()).toEqual(["F026: `@example/old@1.0.0` is denied here: it draws with a library this app no longer serves; the import is dropped"]);
    expect(errors).toEqual([]);
    await page.close();
  }, 30_000);
});
