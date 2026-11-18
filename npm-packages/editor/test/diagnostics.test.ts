// The playground's diagnostics, in the installed Chrome, built and served as the site is: several at once are one list,
// which scrolls as one if it must, and no line of it scrolls on its own, in the list under the text and in the Console.
import { compmd } from "@compmd/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { readFileSync } from "node:fs";
import { rm } from "node:fs/promises";
import { join } from "node:path";
import { chromium, type Browser, type Locator, type Page } from "playwright-core";
import { build, preview, type InlineConfig, type PreviewServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const EDITOR = join(import.meta.dirname, "..");
const OUT = join(import.meta.dirname, ".out", "diagnostics");
const PORT = 4243;

/** A document with a diagnostic on each of its lines but the first. */
const DOCUMENT = ["# Several diagnostics", "", ":: nowhere[One]", "", ":: nothing[Two]", "", ":: nobody[Three] {and a long message to make it wrap=1}", ""].join("\n");

let playground: PreviewServer;
let browser: Browser;

beforeAll(async () => {
  await rm(OUT, { recursive: true, force: true });
  const config = JSON.parse(readFileSync(join(EDITOR, "compmd.config.json"), "utf8")) as Record<string, unknown>;
  const site: InlineConfig = {
    configFile: false,
    root: EDITOR,
    base: "./",
    // Its own cache, where the sandbox's frames are built: the other tests build the playground at the same time.
    cacheDir: join(OUT, ".vite"),
    logLevel: "warn",
    plugins: [svelte(), compmd({ config })],
    worker: { format: "es" },
    build: { target: "es2024", outDir: join(OUT, "site"), emptyOutDir: true },
    preview: { port: PORT, strictPort: true, cors: { origin: "*" } },
  };
  await build(site);
  playground = await preview(site);
  browser = await chromium.launch({ channel: "chrome", headless: true });
});

afterAll(async () => {
  await browser.close();
  await playground.close();
});

/** Each of `lines` that scrolls on its own: its content taller or wider than its box. */
async function scrolling(lines: Locator): Promise<string[]> {
  return lines.evaluateAll((elements) =>
    elements.filter((element) => element.scrollHeight > element.clientHeight || element.scrollWidth > element.clientWidth).map((element) => element.textContent),
  );
}

/** The playground at `width`, with the document typed in, once its three diagnostics show. */
async function opened(width: number, scheme: "light" | "dark"): Promise<Page> {
  const page = await browser.newPage({ viewport: { width, height: 800 }, colorScheme: scheme });
  await page.goto(`http://localhost:${String(PORT)}/`);
  await page.locator(".text-area textarea").fill(DOCUMENT);
  await expect.poll(() => page.locator(".diagnostics li").count(), { timeout: 20_000 }).toBe(3);
  return page;
}

describe.each([
  [1280, "light"],
  [1280, "dark"],
  [600, "light"],
] as const)("several diagnostics at %dpx, %s", (width, scheme) => {
  it("are one list under the text, with no line scrolling on its own", async () => {
    const page = await opened(width, scheme);
    expect(await scrolling(page.locator(".diagnostics li"))).toEqual([]);
    // The list itself scrolls, when it is longer than its room.
    expect(await page.locator(".text > .diagnostics").evaluate((list) => getComputedStyle(list).overflowY)).toBe("auto");
    await page.close();
  });

  it("are one entry in the Console, with no line scrolling on its own", async () => {
    const page = await opened(width, scheme);
    await page.getByRole("tab", { name: "CompMD Inspector" }).click();
    const console = page.locator(".console");
    await expect.poll(() => console.locator("ul li").count(), { timeout: 20_000 }).toBeGreaterThanOrEqual(3);
    expect(await scrolling(console.locator("li"))).toEqual([]);
    await page.close();
  });
});
