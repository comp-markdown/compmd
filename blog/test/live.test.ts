// A blog's live post in Chrome and WebKit, served as `cblo dev` serves it: the page shows the post rendered at build
// time, then its script renders it again through the sandbox, and the post's own JSX component draws in its frame.

import type { Server } from "node:http";
import { rm } from "node:fs/promises";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions } from "playwright";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { serve } from "../src/serve.ts";
import { BLOG, post, writeBlog } from "../src/test-blog.ts";

const LIVE = post(
  { title: "Live", date: "2026-11-15", summary: "A component of its own." },
  ["```jsx component=hello runtime=react@19", "export default function Hello({ name }) {", "  return <b>Hello, {name}, live</b>;", "}", "```", "", "```compmd component=hello signature", "params:", "  name: string", "```", "", ":: hello[Hello, reader] {name=reader}", ""].join("\n"),
);

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

let dir: string;
let server: Server;
let origin: string;

beforeAll(async () => {
  dir = await writeBlog({ ...BLOG, "posts/live.comp.md": LIVE });
  server = await serve(dir, 0, { drafts: false });
  const address = server.address();
  origin = `http://localhost:${String(typeof address === "object" && address ? address.port : 0)}`;
}, 120_000);

afterAll(async () => {
  server.close();
  await rm(dir, { recursive: true, force: true });
});

describe.each(BROWSERS)("a live post in %s", (_name, type, options) => {
  let browser: Browser;

  beforeAll(async () => {
    browser = await type.launch({ headless: true, ...options });
  });

  afterAll(async () => {
    await browser.close();
  });

  it("shows the fallback first, then the post's component drawn in its frame, in the static post's place", async () => {
    const page = await browser.newPage();
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto(`${origin}/blog/live/`);
    await expect.poll(() => page.frameLocator(".post-body iframe").locator("b").innerText(), { timeout: 20_000 }).toBe("Hello, reader, live");
    expect(await page.locator(".post-body").count()).toBe(1);
    expect(await page.locator(".post-body").evaluate((body) => getComputedStyle(body).visibility)).toBe("visible");
    expect(errors).toEqual([]);
    await page.close();
  }, 60_000);

  it("loads no script on a post without components", async () => {
    const page = await browser.newPage();
    const scripts: string[] = [];
    page.on("request", (request) => {
      if (request.resourceType() === "script") scripts.push(request.url());
    });
    await page.goto(`${origin}/blog/first/`);
    expect(await page.locator("article > h1").innerText()).toBe("First");
    expect(scripts).toEqual([]);
    await page.close();
  });
});
