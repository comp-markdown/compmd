// The test page (demo/), as a host renders it: in host mode, each diagram in a strict frame of its own with the pack's
// built module loaded by URL, in Chrome and WebKit, in light and dark. Every diagram of the demo draws, the one Mermaid
// can't read shows its code and what Mermaid found, the frames take the page's colour scheme, and the strict policy
// refuses nothing Mermaid does.
import { join } from "node:path";
import { chromium, webkit, type Browser, type BrowserType, type Frame, type LaunchOptions, type Page } from "playwright";
import { createServer, type ViteDevServer } from "vite";
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
  server = await createServer({ configFile: join(ROOT, "vite.config.ts"), logLevel: "warn", server: { port: PORT, strictPort: true } });
  await server.listen();
}, 60_000);

afterAll(async () => {
  await server.close();
});

/** The frame whose part matches `selector`, once there is one. */
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
const drawing = (label: string) => `svg[role="img"][aria-label="${label}"]`;

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

  it("draws every diagram, as a block and as a leaf, with no diagnostics and nothing refused", async () => {
    const { page, problems } = await open("light");
    expect(await page.locator("#diagnostics li").allInnerTexts()).toEqual([]);
    expect(await (await frameWith(page, drawing(LABELS[1] ?? ""))).locator("svg").innerHTML()).toContain("Open the page");
    expect(await (await frameWith(page, drawing(LABELS[2] ?? ""))).locator("svg").innerHTML()).toContain("Published");
    expect(problems).toEqual([]);
    await page.close();
  }, 60_000);

  it("shows the code of a diagram Mermaid can't read, and what Mermaid found, in a frame its size", async () => {
    const { page } = await open("light");
    const frame = await frameWith(page, "text=/^Mermaid couldn't read this diagram: /");
    expect(await frame.locator("pre", { hasText: "A -->" }).count()).toBe(1);
    const fits = await frame.evaluate(() => document.documentElement.scrollHeight <= window.innerHeight);
    expect(fits).toBe(true);
    await page.close();
  }, 60_000);

  it("draws with Mermaid's dark theme, in frames that take the page's dark scheme", async () => {
    const fill = async (page: Page) => (await frameWith(page, drawing(LABELS[0] ?? ""))).locator(".node rect, .node path").first().evaluate((step) => getComputedStyle(step).fill);
    const light = await open("light");
    const dark = await open("dark");
    expect(await fill(dark.page)).not.toBe(await fill(light.page));
    const frame = await frameWith(dark.page, drawing(LABELS[0] ?? ""));
    expect(await frame.evaluate(() => getComputedStyle(document.documentElement).colorScheme)).toBe("dark");
    await light.page.close();
    await dark.page.close();
  }, 60_000);
});
