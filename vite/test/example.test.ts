// The example app, built with the plugin and served as a host serves it, in Chrome and WebKit: the app's own `stars`
// mounts in its page and sets its input, the strict pack's built module runs in a frame, checked against its hash, and
// the denied pack's use shows its fallback, with the reason.
import { createHash } from "node:crypto";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions } from "playwright";
import { build, createServer, preview, type PreviewServer } from "vite";
import { hashOf } from "@compmd/sandbox/build";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { CONTRACT_HEADERS } from "../src/headers.ts";
import { compmd } from "../src/plugin.ts";

/** The serving contract's headers, by their names as `fetch` gives them. */
const LOWER_CONTRACT = Object.fromEntries(Object.entries(CONTRACT_HEADERS).map(([name, value]) => [name.toLowerCase(), value]));

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

/** The frames' folder in the build, `compmd/frames/<hash>/`: the build's only one. */
const framesDir = (): string => {
  const builds = readdirSync(join(OUT, "compmd/frames"));
  expect(builds).toHaveLength(1);
  return join(OUT, "compmd/frames", builds[0] ?? "");
};

describe("the example's build", () => {
  it("has the frame documents under their hash, and the packs' files by their place", async () => {
    const frames = framesDir();
    expect(await hashOf(frames)).toBe(frames.split("/").at(-1));
    expect(existsSync(join(frames, "strict.html")) && existsSync(join(frames, "permissive.html"))).toBe(true);
    // Each pack's files as published, the trusted folder pack's too, whose document a host processing in Rust reads.
    expect(existsSync(join(OUT, "compmd/packs/@example/charts@1.0.0/charts.comp.md"))).toBe(true);
    expect(existsSync(join(OUT, "compmd/packs/@example/app@1.0.0/index.comp.md"))).toBe(true);
    // The manifest is the host's, not the page's.
    expect(existsSync(join(OUT, "compmd/manifest.json"))).toBe(false);
    expect(readFileSync(join(OUT, "_headers"), "utf8")).toContain("/compmd/frames/*\n  Content-Security-Policy: sandbox allow-scripts\n");
  });

  it("has the built module, with the hash the app gives", () => {
    const module = readFileSync(join(OUT, "compmd/packs/@example/charts@1.0.0/dist/bar.js"));
    const hash = `sha384-${createHash("sha384").update(module).digest("base64")}`;
    const bundle = readFileSync(join(OUT, "assets", readFileSync(join(OUT, "index.html"), "utf8").match(/assets\/(index-[^"]+\.js)/)?.[1] ?? ""), "utf8");
    expect(bundle).toContain(hash);
    // The frames' runtimes, in the request's policy too, so the processor checks against them (spec §7.6).
    const definition = JSON.parse(readFileSync(join(framesDir(), "library-definition.json"), "utf8")) as { runtimes: { name: string; version: string }[] };
    expect(definition.runtimes.map(({ name }) => name)).toEqual(["react", "react-dom", "svelte"]);
    const version = (definition.runtimes[0]?.version ?? "").replaceAll(".", "\\.");
    // Minified, the request's JSON is an object literal: `runtimes:[{name:"react",version:"19.3.0"}`, in any quotes.
    expect(bundle).toMatch(new RegExp(`runtimes:\\[\\{name:["'\`]react["'\`],version:["'\`]${version}["'\`]\\}`));
    // The source component compiled ahead, beside it, with the hash the page gets for its place.
    const total = readFileSync(join(OUT, "compmd/packs/@example/charts@1.0.0/Total.jsx.js"));
    expect(bundle).toContain(`sha384-${createHash("sha384").update(total).digest("base64")}`);
    // The denied pack's code is nowhere: it is never served, and its uses fall back.
    expect(existsSync(join(OUT, "compmd/packs/@example/old@1.0.0"))).toBe(false);
  });

  it("has Shiki and Temml once, in the frames' files, which the page loads too", () => {
    const frames = framesDir();
    expect(existsSync(join(frames, "highlight.js")) && existsSync(join(frames, "math.js"))).toBe(true);
    const chunks = readdirSync(join(frames, "chunks"));
    expect(chunks.some((name) => name.startsWith("cpp-")) && chunks.some((name) => name.startsWith("temml-"))).toBe(true);
    // The app's own bundle: no grammar, no Temml.
    expect(readdirSync(join(OUT, "assets")).filter((name) => /^(cpp|javascript|typescript|temml)-/.test(name))).toEqual([]);
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
      const bars = page.frameLocator("iframe >> nth=0").locator('svg[aria-label="Pages read each day: 30, 52, 18, 41"] rect');
      await expect.poll(() => bars.count(), { timeout: 15_000 }).toBe(4);
      const { sandbox } = (await dev.ssrLoadModule("virtual:compmd")) as { sandbox: { base: string } };
      expect(sandbox.base).toMatch(/^\/compmd\/frames\/[0-9a-f]{16}\/$/);
      const response = await fetch(`http://localhost:${String(PORT + 1)}${sandbox.base}strict.html`);
      expect(Object.fromEntries([...response.headers].filter(([name]) => name in LOWER_CONTRACT))).toEqual(LOWER_CONTRACT);
      const unserved = ["packs/@example/old@1.0.0/index.comp.md", "manifest.json", `${sandbox.base.slice("/compmd/".length)}../frames.json`];
      for (const path of unserved) expect((await fetch(`http://localhost:${String(PORT + 1)}/compmd/${path}`)).headers.get("access-control-allow-origin"), path).toBeNull();
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
    const requested: string[] = [];
    page.on("request", (request) => requested.push(new URL(request.url()).pathname));
    await page.goto(URL_);

    const stars = page.getByRole("radiogroup", { name: "3 of 5" });
    await expect.poll(() => stars.count()).toBe(1);
    await expect.poll(() => page.getByRole("radio", { name: "3 of 5" }).getAttribute("aria-checked")).toBe("true");
    await page.getByRole("radio", { name: "5 of 5" }).click();
    await expect.poll(() => page.locator("main").innerText()).toContain("How good was it? 5 of 5.");

    const bars = page.frameLocator("iframe >> nth=0").locator('svg[aria-label="Pages read each day: 30, 52, 18, 41"] rect');
    await expect.poll(() => bars.count(), { timeout: 10_000 }).toBe(4);
    // The pack's source component, compiled ahead: its frame loads the module by its URL, and never compiles it.
    await expect.poll(() => page.frameLocator("iframe >> nth=1").locator("p").innerText(), { timeout: 10_000 }).toBe("141 pages in all");
    expect(requested).toContain("/compmd/packs/@example/charts@1.0.0/Total.jsx.js");

    await expect.poll(() => page.locator("main").innerText()).toContain("An old gauge: 40");
    // The denied import, and nothing else.
    expect(await page.locator("#diagnostics li").allInnerTexts()).toEqual(["F026: `@example/old@1.0.0` is denied here: it draws with a library this app no longer serves; the import is dropped"]);
    // A build leaves the inspector out.
    expect(await page.locator("#inspector").evaluate((panel) => panel.shadowRoot)).toBeNull();
    expect(errors).toEqual([]);
    await page.close();
  }, 30_000);
});

describe.each(BROWSERS)("the example's inspector in %s, in development", (_name, type, options) => {
  let browser: Browser;
  let dev: Awaited<ReturnType<typeof createServer>>;
  const port = PORT + 2 + BROWSERS.findIndex(([name]) => name === _name);

  beforeAll(async () => {
    dev = await createServer({ root: EXAMPLE, configFile: false, logLevel: "warn", plugins: [compmd()], optimizeDeps: { exclude: ["@compmd/devtools"] }, server: { port, strictPort: true } });
    await dev.listen();
    browser = await type.launch({ headless: true, ...options });
  }, 60_000);

  afterAll(async () => {
    await browser.close();
    await dev.close();
  });

  /** The example in `mode`, with its inspector on the tab `tab`. */
  async function inspect(mode: "host" | "frame", tab: string) {
    const page = await browser.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto(`http://localhost:${String(port)}/?page=${mode}`);
    // The page above the inspector settled: its frames sized, so nothing moves the tab between finding it and the click.
    await page.locator("#page[data-compmd-settled]").waitFor({ timeout: 20_000 });
    const inspector = page.locator("#inspector");
    const button = inspector.getByRole("tab", { name: tab, exact: true });
    await button.click({ timeout: 20_000 });
    expect(await button.getAttribute("aria-selected")).toBe("true");
    return { page, inspector, errors };
  }

  it("shows the host's page, the frame it made and what that frame reports, in host mode", async () => {
    const { page, inspector, errors } = await inspect("host", "Sandbox");
    const sandbox = inspector.locator(".sandbox");
    await expect.poll(() => sandbox.innerText(), { timeout: 20_000 }).toMatch(/bar mounted in .* ms\s*reported by 1/);
    const text = await sandbox.innerText();
    expect(text).toMatch(/the host's page\s+trusted/);
    expect(text).toMatch(/stars mounted in [\d.]+ ms(?!\s*reported)/);
    expect(text).toMatch(/frame 1\s+strict\s+bar/);
    // The denied pack's import, from the inspector's own processing of the document, in the Console under the views.
    await expect.poll(() => inspector.locator(".drawer .console").innerText(), { timeout: 20_000 }).toContain("F026");
    expect(errors).toEqual([]);
    await page.close();
  }, 60_000);

  it("shows the page's frame and what it reports, its fallbacks among them, in frame mode", async () => {
    const { page, inspector, errors } = await inspect("frame", "Sandbox");
    const sandbox = inspector.locator(".sandbox");
    await expect.poll(() => sandbox.innerText(), { timeout: 20_000 }).toMatch(/stars shows its fallback: not-here/);
    const text = await sandbox.innerText();
    expect(text).toMatch(/frame 1\s+strict\s+the page/);
    expect(text).toMatch(/bar mounted in .* ms\s*reported by 1/);
    await inspector.getByRole("tab", { name: "Messages", exact: true }).click();
    await expect.poll(() => inspector.locator(".timeline").innerText(), { timeout: 20_000 }).toContain("render");
    expect(errors).toEqual([]);
    await page.close();
  }, 60_000);

  it("follows the document the app processes again, as a star's click sets its input", async () => {
    const { page, inspector, errors } = await inspect("host", "Data");
    await expect.poll(() => inspector.locator(".data").innerText(), { timeout: 20_000 }).toMatch(/rating[\s\S]*3/);
    await page.getByRole("radio", { name: "5 of 5" }).click();
    await expect.poll(() => inspector.locator(".data").innerText(), { timeout: 20_000 }).toMatch(/rating[\s\S]*5/);
    expect(errors).toEqual([]);
    await page.close();
  }, 60_000);
});
