// The playground's address, in the installed Chrome, built and served as the site is: the document chosen in the menu is
// the address's `?doc=`, so the address bar holds a link to it, and back and forward open the documents again.
import { compmd } from "@compmd/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { readFileSync } from "node:fs";
import { rm } from "node:fs/promises";
import { join } from "node:path";
import { chromium, type Browser, type Page } from "playwright-core";
import { build, preview, type InlineConfig, type PreviewServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const EDITOR = join(import.meta.dirname, "..");
const OUT = join(import.meta.dirname, ".out", "address");
const PORT = 4244;

let playground: PreviewServer;
let browser: Browser;

beforeAll(async () => {
  await rm(OUT, { recursive: true, force: true });
  const config = JSON.parse(readFileSync(join(EDITOR, "compmd.config.json"), "utf8")) as Record<string, unknown>;
  const site: InlineConfig = {
    configFile: false,
    root: EDITOR,
    base: "./",
    publicDir: join(EDITOR, "../public/playground"),
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

/** The menu of documents, in the label that says "Document". */
const menu = (page: Page) => page.locator("label").filter({ has: page.getByText("Document", { exact: true }) }).locator("select");

/** The first line of the document the editor shows. */
const heading = async (page: Page) => (await page.locator(".text-area textarea").inputValue()).split("\n").find((line) => line.startsWith("# "));

describe("the playground's address", () => {
  it("names the document chosen in the menu, and back opens the one before", async () => {
    const page = await browser.newPage();
    await page.goto(`http://localhost:${String(PORT)}/?doc=kyoto`);
    await expect.poll(() => heading(page), { timeout: 20_000 }).toBe("# {{md.doc.title}}");
    await menu(page).selectOption({ label: "Packs: diagrams from a built module" });
    expect(new URL(page.url()).search).toBe("?doc=diagrams");
    await expect.poll(() => heading(page)).toBe("# Diagrams from a pack");
    // The link opens the document it names.
    const link = page.url();
    const shared = await browser.newPage();
    await shared.goto(link);
    await expect.poll(() => heading(shared), { timeout: 20_000 }).toBe("# Diagrams from a pack");
    await shared.close();
    await page.goBack();
    expect(new URL(page.url()).search).toBe("?doc=kyoto");
    await expect.poll(() => heading(page)).toBe("# {{md.doc.title}}");
    await expect.poll(() => menu(page).inputValue()).toBe("6");
    await page.close();
  });
});
