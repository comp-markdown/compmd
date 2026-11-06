// @vitest-environment node
// The landing page's hero in Chrome and WebKit, live: the book club's shelf, `content/shelf.js` in a sandbox frame, picks the book
// clicked, again and again, and the page under it follows.
import { join } from "node:path";
import { chromium, webkit, type Browser, type BrowserType, type Frame, type LaunchOptions, type Page } from "playwright-core";
import { createServer, type ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const PORT = 4193;

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

/** The frame the shelf's covers are drawn in, once they are. */
async function shelfOf(page: Page): Promise<Frame> {
  let found: Frame | undefined;
  await expect
    .poll(
      async () => {
        for (const frame of page.frames()) if ((await frame.locator("[role=radio]").count().catch(() => 0)) > 0) found = frame;
        return found !== undefined;
      },
      { timeout: 30_000 },
    )
    .toBe(true);
  return found as Frame;
}

/** The title of the cover picked: the one checked. */
const picked = (shelf: Frame) => shelf.locator("[role=radio][aria-checked=true]").getAttribute("aria-label");

describe.each(BROWSERS)("the hero's shelf in %s", (_name, type, launch) => {
  let browser: Browser;

  beforeAll(async () => {
    browser = await type.launch({ headless: true, ...launch });
  });

  afterAll(async () => {
    await browser.close();
  });

  it("picks the book clicked first, and keeps it picked when it is clicked again", async () => {
    const page = await browser.newPage({ colorScheme: "light" });
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto(`http://localhost:${String(PORT)}/`);
    const shelf = await shelfOf(page);
    expect(await picked(shelf)).toMatch(/^Klara and the Sun/);
    // Every line of every title fits its cover: 92 units wide, the title starting at 14, with 8 to spare at the end;
    // within half a unit, since a title shrunk to fit exactly measures a hair over by rounding.
    const widest = await shelf.locator("svg text tspan").evaluateAll((spans) => Math.max(...spans.map((span) => (span as SVGTextContentElement).getComputedTextLength())));
    expect(widest).toBeLessThanOrEqual(70.5);
    // Still being read: no rating yet.
    await expect.poll(() => page.locator(".hero .proof").textContent(), { timeout: 10_000 }).toContain("Not rated yet.");
    const hailMary = shelf.getByRole("radio", { name: /^Project Hail Mary/ });
    // Low on the cover, where a neighbour's pattern could reach if it weren't kept to its own cover.
    const box = await hailMary.boundingBox();
    await hailMary.click({ position: { x: 8, y: (box?.height ?? 100) - 30 } });
    await expect.poll(() => picked(shelf), { timeout: 10_000 }).toMatch(/^Project Hail Mary/);
    await expect.poll(() => page.locator(".hero .proof").textContent(), { timeout: 10_000 }).toContain("Andy Weir");
    await hailMary.click();
    // Given time to go wrong: a second click would have been answered by now.
    await page.waitForTimeout(1500);
    expect(await picked(shelf)).toMatch(/^Project Hail Mary/);
    await shelf.getByRole("radio", { name: /^Piranesi/ }).click();
    await expect.poll(() => picked(shelf), { timeout: 10_000 }).toMatch(/^Piranesi/);
    // Read, and rated: its stars, from the club's data.
    await expect.poll(() => page.locator(".hero").getByRole("img", { name: "Rated 5 of 5" }).count(), { timeout: 10_000 }).toBe(1);
    expect(await page.locator(".hero").getByRole("img", { name: "Rated 5 of 5" }).textContent()).toBe("★★★★★");
    expect(errors).toEqual([]);
    await page.close();
  }, 90_000);
});
