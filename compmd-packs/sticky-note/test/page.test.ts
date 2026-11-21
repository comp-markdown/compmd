// The test page `compmd-pack dev` serves (`demo.comp.md`), in Chrome and WebKit. A note is tilted, with a shadow; the
// pinned one sits over its box's end on a wide screen and in the flow on a phone, inside the page's width either way;
// and it prints as it shows.
import { join } from "node:path";
import { readPack, servePack } from "@compmd/pack";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions, type Locator, type Page } from "playwright";
import type { ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const ROOT = join(import.meta.dirname, "..");
const PORT = 4224;

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

const PINNED = "Bring the tide table";

let server: ViteDevServer;

beforeAll(async () => {
  server = await servePack(await readPack(ROOT), { port: PORT });
}, 120_000);

afterAll(async () => {
  await server.close();
});

/** The rendered note whose text starts with `text`, in the examples' output (not their source). */
const note = (page: Page, text: string): Locator => page.locator("div.compmd-box", { hasText: text }).filter({ hasNot: page.locator("div.compmd-box", { hasText: text }) }).last();

/** Where the pinned note is against its box and the paragraph before it. */
async function placeOf(page: Page) {
  const pinned = note(page, PINNED);
  await expect.poll(() => pinned.count(), { timeout: 30_000 }).toBeGreaterThan(0);
  const box = await pinned.locator("xpath=..").boundingBox();
  const own = await pinned.boundingBox();
  const text = await page.locator("p", { hasText: "Back along the old wall" }).last().boundingBox();
  return { box, own, text, position: await pinned.evaluate((element) => getComputedStyle(element).position) };
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
  }, 60_000);

  afterAll(async () => {
    await browser.close();
  });

  it("tilts each note and gives it a shadow, with nothing reported", async () => {
    const first = note(page, "Bring the tide table: the quay floods");
    await expect.poll(() => first.count(), { timeout: 30_000 }).toBeGreaterThan(0);
    const style = await note(page, "Closed on Mondays").evaluate((element) => ({ rotate: getComputedStyle(element).rotate, shadow: getComputedStyle(element).boxShadow }));
    expect(style.rotate).toBe("3deg");
    expect(style.shadow).not.toBe("none");
    expect(await page.locator("#diagnostics li").allInnerTexts()).toEqual([]);
    expect(problems).toEqual([]);
  }, 60_000);

  it("pins the note over its box's end on a wide screen", async () => {
    const { box, own, position } = await placeOf(page);
    expect(position).toBe("absolute");
    expect(box && own && own.x + own.width <= box.x + box.width + 8 && own.x > box.x + box.width / 2).toBe(true);
  }, 60_000);

  it("keeps the note in the flow on a phone, after the text and inside the page", async () => {
    await page.setViewportSize({ width: 375, height: 800 });
    try {
      await expect.poll(async () => (await placeOf(page)).position).toBe("relative");
      const { box, own, text } = await placeOf(page);
      expect(own && text && own.y >= text.y + text.height - 8).toBe(true);
      expect(own && box && own.x >= box.x && own.x + own.width <= box.x + box.width + 8).toBe(true);
      expect(box && box.x >= 0 && box.x + box.width <= 375).toBe(true);
    } finally {
      await page.setViewportSize({ width: 1100, height: 900 });
    }
  }, 60_000);

  it("prints the notes as they show, tilted, with a shadow", async () => {
    await page.emulateMedia({ media: "print" });
    try {
      const style = await note(page, "Closed on Mondays").evaluate((element) => ({ rotate: getComputedStyle(element).rotate, shadow: getComputedStyle(element).boxShadow }));
      expect(style.rotate).toBe("3deg");
      expect(style.shadow).not.toBe("none");
    } finally {
      await page.emulateMedia({ media: "screen" });
    }
  }, 60_000);
});
