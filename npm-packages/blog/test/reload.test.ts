// A post open in Chrome and WebKit while `cblo dev` runs: saved, the page follows, with no reload by hand.

import type { Server } from "node:http";
import { rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions } from "playwright";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { serve } from "../src/serve.ts";
import { post, writeBlog } from "../src/test-blog.ts";

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

let dir: string;
let server: Server;
let origin: string;

beforeAll(async () => {
  dir = await writeBlog();
  server = await serve(dir, 0, { drafts: true });
  const address = server.address();
  origin = `http://localhost:${String(typeof address === "object" && address ? address.port : 0)}`;
});

afterAll(async () => {
  server.close();
  await rm(dir, { recursive: true, force: true });
});

describe.each(BROWSERS)("a post open in %s", (name, type, options) => {
  let browser: Browser;

  beforeAll(async () => {
    browser = await type.launch({ headless: true, ...options });
  });

  afterAll(async () => {
    await browser.close();
  });

  it("follows a save of the post", async () => {
    const page = await browser.newPage();
    await page.goto(`${origin}/blog/first/`);
    expect(await page.locator(".post-body").innerText()).not.toContain(`Saved in ${name}.`);
    // The page's stream opens once its script runs; a save before it would go untold.
    await page.waitForTimeout(500);
    await writeFile(join(dir, "posts/first.comp.md"), post({ title: "First", date: "2026-11-01", summary: "The first post." }, `Saved in ${name}.\n`));
    await expect.poll(() => page.locator(".post-body").innerText(), { timeout: 10_000 }).toContain(`Saved in ${name}.`);
    await page.close();
  });
});
