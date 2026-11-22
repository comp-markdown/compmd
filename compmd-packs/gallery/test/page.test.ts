// The test page `compmd-pack dev` serves (`demo.comp.md`), in Chrome and WebKit. A gallery's cards are one height,
// across rows too; with `ratio=4/3`, photos of different shapes are cropped to that shape, covering it, or shown whole
// in it with `image-fit=contain`; and they print cropped as they show.
import { join } from "node:path";
import { readPack, servePack } from "@compmd/pack";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions, type Locator, type Page } from "playwright";
import type { ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const ROOT = join(import.meta.dirname, "..");
const PORT = 4234;

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

/** The demo's examples, in order: three walks first, the photos cropped to 4:3 sixth, shown whole seventh. */
const WALKS = 0;
const CROPPED = 5;
const WHOLE = 6;

let server: ViteDevServer;

beforeAll(async () => {
  server = await servePack(await readPack(ROOT), { port: PORT });
}, 120_000);

afterAll(async () => {
  await server.close();
});

/** What the example `index` draws, without its source. */
const output = (page: Page, index: number): Locator => page.locator(".compmd-example").nth(index).locator(".compmd-example-document");

/** The heights of the cards of the gallery in `example`: the children of its grid. */
const cardHeights = (example: Locator) =>
  example.evaluate((root) => {
    const grid = [...root.querySelectorAll("*")].find((element) => getComputedStyle(element).display === "grid");
    return [...(grid?.children ?? [])].map((card) => Math.round(card.getBoundingClientRect().height));
  });

/** Each image of `example` once loaded: its box's width over its height, its own fit and position, and whether its picture is cut. */
async function imagesOf(example: Locator) {
  await expect.poll(() => example.locator("img").evaluateAll((images) => images.length > 0 && images.every((image) => (image as HTMLImageElement).complete && (image as HTMLImageElement).naturalWidth > 0)), { timeout: 30_000 }).toBe(true);
  return example.locator("img").evaluateAll((images) =>
    images.map((image) => {
      const box = (image.parentElement?.closest(".compmd-image-box") ?? image).getBoundingClientRect();
      const style = getComputedStyle(image);
      return { ratio: Math.round((box.width / box.height) * 100) / 100, fit: style.objectFit, position: style.objectPosition };
    }),
  );
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
    await page.locator(".compmd-example").nth(WHOLE).waitFor({ timeout: 30_000 });
  }, 60_000);

  afterAll(async () => {
    await browser.close();
  });

  it("makes every card of a gallery one height, with nothing reported", async () => {
    const heights = await cardHeights(output(page, WALKS));
    expect(heights).toHaveLength(3);
    expect(new Set(heights).size).toBe(1);
    expect(await page.locator("#diagnostics li").allInnerTexts()).toEqual([]);
    expect(problems).toEqual([]);
  }, 60_000);

  it("crops photos of different shapes to one shape, covering it", async () => {
    const images = await imagesOf(output(page, CROPPED));
    expect(images.map(({ ratio }) => ratio)).toEqual([1.33, 1.33, 1.33]);
    expect(images.map(({ fit }) => fit)).toEqual(["cover", "cover", "cover"]);
  }, 60_000);

  it("shows each photo whole in the shape, at its top, with image-fit=contain and image-position=\"start center\"", async () => {
    const images = await imagesOf(output(page, WHOLE));
    expect(images.map(({ ratio }) => ratio)).toEqual([1.33, 1.33, 1.33]);
    expect(images.map(({ fit }) => fit)).toEqual(["contain", "contain", "contain"]);
    expect(images.map(({ position }) => position)).toEqual(["50% 0%", "50% 0%", "50% 0%"]);
  }, 60_000);

  it("prints the photos cropped as they show", async () => {
    await page.emulateMedia({ media: "print" });
    try {
      const images = await imagesOf(output(page, CROPPED));
      expect(images.map(({ ratio }) => ratio)).toEqual([1.33, 1.33, 1.33]);
      expect(images.map(({ fit }) => fit)).toEqual(["cover", "cover", "cover"]);
    } finally {
      await page.emulateMedia({ media: "screen" });
    }
  }, 60_000);
});
