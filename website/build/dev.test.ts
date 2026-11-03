// @vitest-environment node
// The dev server's spec and guide pages, in Chrome and WebKit: each paints styled from its HTML alone, as the build's
// prerendered pages do, so following a link shows no unstyled page while its script loads. And its PDFs, printed when
// asked for.
import { join } from "node:path";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions } from "playwright-core";
import { createServer, type ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const PORT = 4192;
const PAGES = ["spec/", "guides/", "guides/developers/", "guides/developers/quick-start/"];
/** The site's desk colour in the light scheme, `--desk` in `src/base.css`. */
const DESK = "rgb(233, 237, 241)";

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

let server: ViteDevServer;

beforeAll(async () => {
  const root = join(import.meta.dirname, "..");
  server = await createServer({ root, configFile: join(root, "vite.config.ts"), logLevel: "warn", server: { port: PORT, strictPort: true } });
  await server.listen();
}, 60_000);

afterAll(async () => {
  await server.close();
});

describe("the dev server's PDFs", () => {
  // Printed from the working tree when asked for, as the build prints them, not the HTML page a missing file would get.
  it.each(["guides/authors/compmd-authors-guide.pdf", "guides/developers/compmd-developers-guide.pdf", "spec/compmd-spec-draft.pdf"])("serves /%s as a PDF", async (path) => {
    const response = await fetch(`http://localhost:${String(PORT)}/${path}`);
    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toBe("application/pdf");
    expect(Buffer.from(await response.arrayBuffer()).subarray(0, 5).toString()).toBe("%PDF-");
  }, 60_000);
});

describe.each(BROWSERS)("the dev server's pages in %s", (_name, type, launch) => {
  let browser: Browser;

  beforeAll(async () => {
    browser = await type.launch({ headless: true, ...launch });
  });

  afterAll(async () => {
    await browser.close();
  });

  it.each(PAGES)("/%s is styled before any script runs", async (path) => {
    const page = await browser.newPage({ javaScriptEnabled: false, colorScheme: "light" });
    await page.goto(`http://localhost:${String(PORT)}/${path}`);
    expect(await page.evaluate(() => getComputedStyle(document.body).backgroundColor)).toBe(DESK);
    await page.close();
  }, 60_000);

  it("leaves the styles to the script once it runs, and stays styled", async () => {
    const page = await browser.newPage({ colorScheme: "light" });
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto(`http://localhost:${String(PORT)}/guides/developers/quick-start/`);
    await expect.poll(() => page.locator("link[data-dev]").count(), { timeout: 20_000 }).toBe(0);
    expect(await page.evaluate(() => getComputedStyle(document.body).backgroundColor)).toBe(DESK);
    expect(errors).toEqual([]);
    await page.close();
  }, 60_000);
});
