// The test page `compmd-pack dev` serves (`demo.comp.md`), as a host renders it: in the strict sandbox with the pack's
// built module loaded by URL, in Chrome and WebKit, in light and dark. Every diagram of the demo draws, the one Mermaid
// can't read shows its code and is reported as W118 with what Mermaid found, the frame takes the page's colour scheme,
// and the strict policy refuses nothing Mermaid does.
import { join } from "node:path";
import { readPack, servePack } from "@compmd/pack";
import { chromium, webkit, type Browser, type BrowserType, type Frame, type LaunchOptions, type Page } from "playwright";
import type { ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const ROOT = join(import.meta.dirname, "..");
const PORT = 4221;

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

const LABELS = [
  "Draft, then review, then published",
  "The reader asks the host for a page, and the host answers",
  "The steps: draft, review, published",
  "A request goes through the cache, then the server",
  "Idea, then plan",
];

let server: ViteDevServer;

beforeAll(async () => {
  server = await servePack(await readPack(ROOT), { port: PORT });
}, 120_000);

afterAll(async () => {
  await server.close();
});

/** The frame whose content matches `selector`, once there is one. */
async function frameWith(page: Page, selector: string): Promise<Frame> {
  let found: Frame | undefined;
  await expect
    .poll(
      async () => {
        for (const frame of page.frames()) if ((await frame.locator(selector).count().catch(() => 0)) > 0) found = frame;
        return found !== undefined;
      },
      { timeout: 30_000 },
    )
    .toBe(true);
  return found as Frame;
}

/** The drawing labelled `label`. */
const drawing = (label: string) => `[role="img"][aria-label="${label}"] svg`;

describe.each(BROWSERS)("the test page in %s", (_name, type, options) => {
  let browser: Browser;

  beforeAll(async () => {
    browser = await type.launch({ headless: true, ...options });
  });

  afterAll(async () => {
    await browser.close();
  });

  /** The page in `scheme` once every diagram is drawn, with what went wrong in it: errors, and the policy's refusals. */
  async function open(scheme: "light" | "dark"): Promise<{ page: Page; problems: string[] }> {
    const page = await browser.newPage({ colorScheme: scheme });
    const problems: string[] = [];
    page.on("pageerror", (error) => problems.push(error.message));
    page.on("console", (message) => {
      if (/Content.Security.Policy|Refused to/i.test(message.text())) problems.push(message.text());
    });
    await page.goto(`http://localhost:${String(PORT)}/`);
    for (const label of LABELS) await frameWith(page, drawing(label));
    return { page, problems };
  }

  it("draws every diagram, as a block and as a leaf, with nothing refused", async () => {
    const { page, problems } = await open("light");
    const frame = await frameWith(page, drawing(LABELS[1] ?? ""));
    expect(await frame.locator(drawing(LABELS[1] ?? "")).innerHTML()).toContain("Open the page");
    expect(await frame.locator(drawing(LABELS[2] ?? "")).innerHTML()).toContain("Published");
    expect(problems).toEqual([]);
    await page.close();
  }, 60_000);

  it("shows the code of a diagram Mermaid can't read, and reports what Mermaid found as W118", async () => {
    const { page } = await open("light");
    const frame = await frameWith(page, "pre >> text=A -->");
    expect(await frame.locator("pre", { hasText: "A -->" }).count()).toBe(1);
    await expect.poll(() => page.locator("#diagnostics li").allInnerTexts(), { timeout: 30_000 }).toEqual([expect.stringMatching(/^W118: mermaid: Mermaid couldn't read this diagram: /)]);
    await page.close();
  }, 60_000);

  it("draws with Mermaid's dark theme, in a frame that takes the page's dark scheme, and back when switched", async () => {
    const fill = async (page: Page) => (await frameWith(page, drawing(LABELS[0] ?? ""))).locator(".node rect, .node path").first().evaluate((step) => getComputedStyle(step).fill);
    const light = await open("light");
    const dark = await open("dark");
    const darkFill = await fill(dark.page);
    expect(darkFill).not.toBe(await fill(light.page));
    const frame = await frameWith(dark.page, drawing(LABELS[0] ?? ""));
    expect(await frame.evaluate(() => getComputedStyle(document.documentElement).colorScheme)).toBe("dark");
    // The page's own switch: the frame and its diagrams follow it, with no render.
    await dark.page.selectOption("#scheme", "light");
    await expect.poll(() => fill(dark.page), { timeout: 30_000 }).toBe(await fill(light.page));
    await light.page.close();
    await dark.page.close();
  }, 60_000);
});
