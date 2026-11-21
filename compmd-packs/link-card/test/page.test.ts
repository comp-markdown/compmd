// The test page `compmd-pack dev` serves (`demo.comp.md`), in Chrome and WebKit: link cards in a grid row are as tall
// as the tallest, their cards filling the boxes the grid stretches, and a click on a card's text follows its link.
import { join } from "node:path";
import { readPack, servePack } from "@compmd/pack";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions, type Locator, type Page } from "playwright";
import type { ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const ROOT = join(import.meta.dirname, "..");
const PORT = 4225;

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

/** The "read next" example's three links, one for each card. */
const LINKS = ["#harbour", "#orchard", "#canal"];

let server: ViteDevServer;

beforeAll(async () => {
  server = await servePack(await readPack(ROOT), { port: PORT });
}, 120_000);

afterAll(async () => {
  await server.close();
});

/** The card, in the "read next" example, whose title links to `href`: a link card's card, inside the box with `stretch-link`. */
const card = (page: Page, href: string) => page.locator(`.compmd-stretch-link > div.compmd-box:has(> h3 > a[href="${href}"])`);

/** A click at the middle of `target`, whatever covers it there: the stretched link, over a card's text, is meant to. */
const clickOver = (target: Locator) => target.click({ force: true });

describe.each(BROWSERS)("the test page in %s", (_name, type, options) => {
  let browser: Browser;
  let page: Page;
  const problems: string[] = [];

  beforeAll(async () => {
    browser = await type.launch({ headless: true, ...options });
    page = await browser.newPage({ viewport: { width: 1100, height: 900 } });
    page.on("pageerror", (error) => problems.push(error.message));
    await page.goto(`http://localhost:${String(PORT)}/`);
    await card(page, "#canal").waitFor({ timeout: 30_000 });
  }, 60_000);

  afterAll(async () => {
    await browser.close();
  });

  it("makes the cards of a grid row as tall as the tallest, side by side", async () => {
    const boxes = await Promise.all(LINKS.map((href) => card(page, href).boundingBox()));
    const [first] = boxes;
    for (const box of boxes) {
      expect(box?.y).toBeCloseTo(first?.y ?? -1, 0);
      expect(box?.height).toBeCloseTo(first?.height ?? -1, 0);
    }
    expect(problems).toEqual([]);
  }, 60_000);

  it("follows a card's link from a click on its text", async () => {
    await clickOver(card(page, "#orchard").getByText("Apples in October."));
    await expect.poll(() => new URL(page.url()).hash).toBe("#orchard");
  }, 60_000);

  it("follows a card with stretch-link's title from a click on its image, and its other link from a click on that", async () => {
    const rich = page.locator('div.compmd-box.compmd-stretch-link:has(> h3 > a[href="#harbour"]):has(img)');
    await clickOver(rich.locator("img"));
    await expect.poll(() => new URL(page.url()).hash).toBe("#harbour");
    await rich.getByRole("link", { name: "the tide table" }).click();
    await expect.poll(() => new URL(page.url()).hash).toBe("#tides");
  }, 60_000);
});
