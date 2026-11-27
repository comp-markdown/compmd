// The preview page in Chrome and WebKit, built and served as a site, with the language server's routes played by the
// test: its events (each change in the writer's editor) and the document's request. The page renders the text, follows
// an edit, scrolls to the line being edited, and shows the CompMD Inspector behind its tab.
import { compmd } from "@compmd/vite";
import type { Request } from "@compmd/processor";
import type { ServerResponse } from "node:http";
import { rm } from "node:fs/promises";
import { join } from "node:path";
import { chromium, webkit, type Browser, type BrowserType, type Page } from "playwright-core";
import { build, preview, type InlineConfig, type Plugin, type PreviewServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const APP = join(import.meta.dirname, "..");
const OUT = join(import.meta.dirname, ".out");
const PORT = 4251;

/** The pages listening to the events, and the request the server gives. */
const listening = new Set<ServerResponse>();
const REQUEST: Request = { path: "notes.comp.md", files: {}, now: "2026-11-26T09:00:00Z" };

/** The language server's routes beside the page: `events`, and `request.json`. */
const server: Plugin = {
  name: "language-server",
  configurePreviewServer(preview) {
    preview.middlewares.use((request, response, next) => {
      if (request.url === "/events") {
        response.writeHead(200, { "content-type": "text/event-stream", "cache-control": "no-cache" });
        response.write(": open\n\n");
        listening.add(response);
        request.on("close", () => listening.delete(response));
        return;
      }
      if (request.url === "/request.json") {
        response.setHeader("content-type", "application/json");
        response.end(JSON.stringify(REQUEST));
        return;
      }
      next();
    });
  },
};

/** Sends each page listening the change `text`, with `line` being edited. */
function change(text: string, line: number): void {
  for (const response of listening) response.write(`event: change\ndata: ${JSON.stringify({ text, line })}\n\n`);
}

let site: PreviewServer;

beforeAll(async () => {
  await rm(OUT, { recursive: true, force: true });
  const config: InlineConfig = {
    configFile: false,
    root: APP,
    base: "./",
    cacheDir: join(OUT, ".vite"),
    logLevel: "warn",
    plugins: [compmd(), server],
    worker: { format: "es" },
    build: { target: "es2024", outDir: join(OUT, "site"), emptyOutDir: true },
    preview: { port: PORT, strictPort: true },
  };
  await build(config);
  site = await preview(config);
});

afterAll(async () => {
  await site.close();
});

/** The document's text: a title, then `count` paragraphs, each a line of its own after a blank one. */
const paragraphs = (count: number) => ["# Harbour notes", ...Array.from({ length: count }, (_, i) => `Paragraph ${String(i + 1)}.`)].join("\n\n") + "\n";

describe.each([
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
] as [string, BrowserType, object][])("the preview in %s", (_, type, options) => {
  let browser: Browser;

  beforeAll(async () => {
    browser = await type.launch({ headless: true, ...options });
  });

  afterAll(async () => {
    await browser.close();
  });

  /** A new page of the preview, once it listens to the events. */
  async function opened(): Promise<Page> {
    // The page before it gone, so the one listening is this one: it listens only once it has the request.
    await expect.poll(() => listening.size, { timeout: 20_000 }).toBe(0);
    const page = await browser.newPage({ viewport: { width: 1000, height: 700 } });
    await page.goto(`http://localhost:${String(PORT)}/`);
    await expect.poll(() => listening.size, { timeout: 20_000 }).toBe(1);
    return page;
  }

  it("renders the writer's text, and follows an edit", async () => {
    const page = await opened();
    change("# Harbour notes\n\nThe tide is out.\n", 3);
    const frame = page.frameLocator("iframe");
    await expect.poll(() => frame.getByRole("heading", { name: "Harbour notes" }).count(), { timeout: 20_000 }).toBe(1);
    change("# Harbour notes\n\nThe tide is in.\n", 3);
    await expect.poll(() => frame.getByText("The tide is in.").count(), { timeout: 20_000 }).toBe(1);
    await page.close();
  });

  it("scrolls to the block that holds the line being edited", async () => {
    const page = await opened();
    // Paragraph n is on line 2n + 1.
    change(paragraphs(80), 141);
    const frame = page.frameLocator("iframe");
    const seventieth = frame.getByText("Paragraph 70.");
    await expect.poll(() => seventieth.count(), { timeout: 20_000 }).toBe(1);
    // In the frame's window, which the page scrolled: the paragraph wholly in view, but for the part of a pixel a
    // window scrolled by whole pixels leaves of a line that isn't (the host's stylesheet's are 24.8px).
    const inView = () =>
      seventieth.evaluate((element) => {
        const box = element.getBoundingClientRect();
        return box.top >= 0 && box.bottom < window.innerHeight + 1;
      });
    await expect.poll(inView, { timeout: 20_000 }).toBe(true);
    expect(await frame.getByRole("heading", { name: "Harbour notes" }).evaluate(() => window.scrollY)).toBeGreaterThan(0);
    await page.close();
  });

  it("draws a table's cells bordered, its header apart and its rows banded", async () => {
    const page = await opened();
    change("| Tide | Time |\n| ---- | ---: |\n| High | 06:12 |\n| Low | 12:30 |\n| High | 18:41 |\n", 1);
    const frame = page.frameLocator("iframe");
    const cell = frame.getByRole("cell", { name: "06:12" });
    await expect.poll(() => cell.count(), { timeout: 20_000 }).toBe(1);
    const look = await frame.locator("table").evaluate((table) => {
      const style = (selector: string) => getComputedStyle(table.querySelector(selector) ?? table);
      return {
        border: style("td").borderTopStyle,
        header: style("thead th").backgroundColor,
        odd: style("tbody tr:nth-child(1)").backgroundColor,
        even: style("tbody tr:nth-child(2)").backgroundColor,
        aligned: style("tbody td:nth-child(2)").textAlign,
      };
    });
    expect(look.border).toBe("solid");
    expect(new Set([look.header, look.odd, look.even]).size, "three backgrounds").toBe(3);
    expect(look.aligned, "the column's alignment stays").toBe("end");
    // To look at, beside the built site.
    await frame.locator("table").screenshot({ path: join(OUT, `table-${type.name()}.png`) });
    await page.close();
  });

  it("shows the CompMD Inspector behind its tab, and the page again", async () => {
    const page = await opened();
    change("# Harbour notes\n", 1);
    const frame = page.frameLocator("iframe");
    await expect.poll(() => frame.getByRole("heading", { name: "Harbour notes" }).count(), { timeout: 20_000 }).toBe(1);
    await page.getByRole("tab", { name: "CompMD Inspector" }).click();
    await expect.poll(() => page.getByRole("tab", { name: "Expanded tree" }).isVisible(), { timeout: 20_000 }).toBe(true);
    await page.getByRole("tab", { name: "Page" }).click();
    await expect.poll(() => page.getByRole("tab", { name: "Expanded tree" }).isVisible()).toBe(false);
    await page.close();
  });
});
