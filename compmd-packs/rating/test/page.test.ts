// The test page `compmd-pack dev` serves (`demo.comp.md`), as a host renders it: in host mode, each rating in a strict
// frame of its own with the pack's built module loaded by URL, in Chrome and WebKit. Every example's stars draw, with
// their label as the text alternative and their value filled; inline ones sit in their sentence; they print as drawn;
// and the strict policy refuses nothing.
import { join } from "node:path";
import { readPack, servePack } from "@compmd/pack";
import { chromium, webkit, type Browser, type BrowserType, type Frame, type LaunchOptions, type Page } from "playwright";
import type { ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const ROOT = join(import.meta.dirname, "..");
const PORT = 4223;

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

/** Each example's label, and the stars it fills of how many. */
const RATINGS: [label: string, filled: number, of: number][] = [
  ["Four of five stars", 4, 5],
  ["Three and a half of five", 3.5, 5],
  ["Seven of ten", 7, 10],
  ["two of five", 2, 5],
  ["five of five", 5, 5],
];

let server: ViteDevServer;

beforeAll(async () => {
  server = await servePack(await readPack(ROOT), { port: PORT });
}, 120_000);

afterAll(async () => {
  await server.close();
});

const drawing = (label: string) => `svg[role="img"][aria-label="${label}"]`;

/** The frame that draws the rating labelled `label`, once one does. */
async function frameWith(page: Page, label: string): Promise<Frame> {
  let found: Frame | undefined;
  await expect
    .poll(
      async () => {
        for (const frame of page.frames()) if ((await frame.locator(drawing(label)).count().catch(() => 0)) > 0) found = frame;
        return found !== undefined;
      },
      { timeout: 30_000 },
    )
    .toBe(true);
  return found as Frame;
}

describe.each(BROWSERS)("the test page in %s", (_name, type, options) => {
  let browser: Browser;
  let page: Page;
  const problems: string[] = [];

  beforeAll(async () => {
    browser = await type.launch({ headless: true, ...options });
    page = await browser.newPage();
    page.on("pageerror", (error) => problems.push(error.message));
    page.on("console", (message) => {
      if (/Content.Security.Policy|Refused to/i.test(message.text())) problems.push(message.text());
    });
    await page.goto(`http://localhost:${String(PORT)}/`);
  }, 60_000);

  afterAll(async () => {
    await browser.close();
  });

  it("draws every example's stars, each labelled, its value filled, with nothing refused or reported", async () => {
    for (const [label, filled, of] of RATINGS) {
      const frame = await frameWith(page, label);
      const stars = frame.locator(drawing(label));
      expect(await stars.locator("g[fill-opacity] path").count(), label).toBe(of);
      expect(Number(await stars.locator("clipPath rect").getAttribute("width")) / 24, label).toBe(filled);
      expect((await stars.boundingBox())?.width ?? 0, label).toBeGreaterThan(0);
    }
    expect(await page.locator("#diagnostics li").allInnerTexts()).toEqual([]);
    expect(problems).toEqual([]);
  }, 60_000);

  it("sits an inline rating in its sentence, as high as the text around it", async () => {
    await frameWith(page, "two of five");
    const frame = page.locator('iframe[title="rating"]').nth(3);
    const box = await frame.boundingBox();
    const line = await page.locator("p", { hasText: "The cheese was" }).boundingBox();
    expect(box && line && box.y >= line.y - 1 && box.y + box.height <= line.y + line.height + 1).toBe(true);
  }, 60_000);

  it("prints its stars as drawn", async () => {
    await page.emulateMedia({ media: "print" });
    try {
      const frame = await frameWith(page, "Four of five stars");
      expect((await frame.locator(drawing("Four of five stars")).boundingBox())?.width ?? 0).toBeGreaterThan(0);
    } finally {
      await page.emulateMedia({ media: "screen" });
    }
  }, 60_000);
});
