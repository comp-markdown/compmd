// The test page `compmd-pack dev` serves (`demo.comp.md`), in Chrome and WebKit: each event's dot sits on the
// timeline's line, round, the current one in another colour, and the date is smaller than the text.
import { join } from "node:path";
import { readPack, servePack } from "@compmd/pack";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions, type Locator, type Page } from "playwright";
import type { ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const ROOT = join(import.meta.dirname, "..");
const PORT = 4228;

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

/** The first event whose text holds `text` (the data example repeats the current-event one's): the box with it that has a dot. */
const event = (page: Page, text: string): Locator =>
  page
    .locator("div.compmd-box", { hasText: text })
    .filter({ has: page.locator(":scope > div.compmd-box:empty") })
    .filter({ hasNot: page.locator("div.compmd-box div.compmd-box div.compmd-box") })
    .first();
const dotOf = (event: Locator) => event.locator(":scope > div.compmd-box:empty");

describe.each(BROWSERS)("the test page in %s", (_name, type, options) => {
  let browser: Browser;
  let page: Page;
  const problems: string[] = [];

  beforeAll(async () => {
    browser = await type.launch({ headless: true, ...options });
    page = await browser.newPage({ viewport: { width: 1100, height: 900 } });
    page.on("pageerror", (error) => problems.push(error.message));
    await page.goto(`http://localhost:${String(PORT)}/`);
    await event(page, "Lit again on a spring evening").waitFor({ timeout: 30_000 });
  }, 60_000);

  afterAll(async () => {
    await browser.close();
  });

  it("puts each event's dot on the timeline's line, round", async () => {
    const harbour = event(page, "Built in a single summer");
    const line = await harbour.locator("xpath=..").evaluate((timeline) => {
      const { left } = timeline.getBoundingClientRect();
      return left + parseFloat(getComputedStyle(timeline).borderInlineStartWidth) / 2;
    });
    const dot = await dotOf(harbour).boundingBox();
    expect(dot && Math.abs(dot.x + dot.width / 2 - line)).toBeLessThanOrEqual(1);
    expect(dot?.width).toBeCloseTo(dot?.height ?? -1, 0);
    expect(dot?.width ?? 0).toBeGreaterThan(8);
    expect(problems).toEqual([]);
  }, 60_000);

  it("colours the current event's dot apart from the others", async () => {
    const colour = (text: string) => dotOf(event(page, text)).evaluate((dot) => getComputedStyle(dot).backgroundColor);
    expect(await colour("Lit again on a spring evening")).not.toBe(await colour("Back on the quay"));
  }, 60_000);

  it("sets the date smaller than the event's text", async () => {
    const harbour = event(page, "Built in a single summer");
    const size = (locator: Locator) => locator.evaluate((element) => parseFloat(getComputedStyle(element).fontSize));
    expect(await size(harbour.locator(":scope > div.compmd-box", { hasText: "1897" }))).toBeLessThan(await size(harbour.locator("p", { hasText: "Built in a single summer" })));
  }, 60_000);
});
