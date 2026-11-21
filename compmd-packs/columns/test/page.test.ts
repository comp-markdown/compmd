// The test page `compmd-pack dev` serves (`demo.comp.md`), in Chrome and WebKit: the columns side by side in their
// box, a wide one twice the others, the first on the right in a right-to-left document, and stacked in order on a
// phone.
import { join } from "node:path";
import { readPack, servePack } from "@compmd/pack";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions, type Page } from "playwright";
import type { ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const ROOT = join(import.meta.dirname, "..");
const PORT = 4227;

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

let server: ViteDevServer;

beforeAll(async () => {
  server = await servePack(await readPack(ROOT), { port: PORT });
}, 120_000);

afterAll(async () => {
  await server.close();
});

/** Where each column whose text starts with one of `starts` is, in their order. */
async function places(page: Page, starts: string[]) {
  return Promise.all(starts.map(async (start) => page.locator("div.compmd-box > div.compmd-box", { hasText: start }).last().boundingBox()));
}

describe.each(BROWSERS)("the test page in %s", (_name, type, options) => {
  let browser: Browser;
  let page: Page;
  const problems: string[] = [];

  beforeAll(async () => {
    browser = await type.launch({ headless: true, ...options });
    page = await browser.newPage({ viewport: { width: 1100, height: 900 } });
    page.on("pageerror", (error) => problems.push(error.message));
    await page.goto(`http://localhost:${String(PORT)}/`);
    await page.locator("div.compmd-box > div.compmd-box", { hasText: "Evening" }).last().waitFor({ timeout: 30_000 });
  }, 60_000);

  afterAll(async () => {
    await browser.close();
  });

  it("lays three columns side by side, in order, one height apart from none", async () => {
    const [morning, noon, evening] = await places(page, ["Morning", "Noon", "Evening"]);
    expect(morning && noon && evening && morning.x < noon.x && noon.x < evening.x).toBe(true);
    expect(morning?.y).toBeCloseTo(evening?.y ?? -1, 0);
    expect(problems).toEqual([]);
  }, 60_000);

  it("makes a wide column twice the other", async () => {
    const [ingredients, method] = await places(page, ["Ingredients", "Method"]);
    expect((method?.width ?? 0) / (ingredients?.width ?? 1)).toBeCloseTo(2, 0);
  }, 60_000);

  it("puts the first column on the right in right-to-left text", async () => {
    const [first, last] = await places(page, ["الميناء", "البستان"]);
    expect(first && last && first.x > last.x).toBe(true);
  }, 60_000);

  it("stacks the columns in order on a phone", async () => {
    await page.setViewportSize({ width: 375, height: 800 });
    try {
      await expect.poll(async () => {
        const [morning, noon] = await places(page, ["Morning", "Noon"]);
        return morning && noon ? noon.y > morning.y + morning.height - 1 : false;
      }).toBe(true);
      const [morning, noon, evening] = await places(page, ["Morning", "Noon", "Evening"]);
      expect(morning?.x).toBeCloseTo(noon?.x ?? -1, 0);
      expect(evening && noon && evening.y > noon.y).toBe(true);
    } finally {
      await page.setViewportSize({ width: 1100, height: 900 });
    }
  }, 60_000);
});
