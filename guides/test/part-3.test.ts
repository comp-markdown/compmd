// The Developer's Guide's Part III, Components, against what its chapters claim: the app's own components, in React
// and Svelte (chapter 10),
// and the two page modes with the headers the frames are served with (chapter 11), built with Vite and the plugin and
// opened in Chrome and WebKit; and the Developer Tools (chapter 12), the inspector and the editor, on the dev server.
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { initSync, process } from "@compmd/processor";
import { compmd, loadApp } from "@compmd/vite";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions, type Page } from "playwright";
import { build, createServer, preview, type PreviewServer, type ViteDevServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const EXAMPLES = join(import.meta.dirname, "../developers/examples");
const OUT = join(import.meta.dirname, ".out/part-3");

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

beforeAll(() => {
  initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) });
});

/**
 * Builds the example `name` with its own `vite.config.ts`, or with the plugin alone as the quick start's is, and serves
 * the build on `port`. Each build has a cache of its own, where the plugin writes the frames, so builds that other test
 * files run at the same time can't remove them under it.
 */
async function served(name: string, port: number): Promise<PreviewServer> {
  const root = join(EXAMPLES, name);
  const own = join(root, "vite.config.ts");
  const config = {
    root,
    ...(existsSync(own) ? { configFile: own } : { configFile: false as const, plugins: [compmd()] }),
    logLevel: "warn" as const,
    cacheDir: join(OUT, `${name}.cache`),
    build: { outDir: join(OUT, name), emptyOutDir: true, target: "es2024" },
  };
  await build(config);
  return preview({ ...config, preview: { port, strictPort: true } });
}

/**
 * Opens `url` in a new page of `browser`, collecting the page's errors, the warnings its code logs, and every answer
 * of the server's that failed (a dev server's 504 for a dependency it optimized again, by its URL, which a module that
 * fails to load doesn't name).
 */
async function opened(browser: Browser, url: string): Promise<{ page: Page; errors: string[] }> {
  const page = await browser.newPage();
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "warning") errors.push(message.text());
  });
  page.on("response", (response) => {
    if (response.status() >= 500) errors.push(`${String(response.status())} ${response.statusText()} for ${response.url()}`);
  });
  await page.goto(url);
  return { page, errors };
}

describe("the app's own components", () => {
  const root = join(EXAMPLES, "own-components");
  const port = 4220;
  let server: PreviewServer;

  beforeAll(async () => {
    server = await served("own-components", port);
  });

  afterAll(async () => {
    await server.close();
  });

  it("are packs the plugin reads as the chapter says: stars and spoiler trusted, in React and Svelte; reviews permissive with host", async () => {
    const app = await loadApp(root);
    expect(app.warnings).toEqual([]);
    expect(app.trusted.map(({ name, binding }) => [name, binding])).toEqual([
      ["stars", "react"],
      ["spoiler", "svelte"],
    ]);
    expect(app.request.ambient).toEqual(["@you/app"]);
    expect(app.request.policy.packs).toEqual({ "@you/app": { level: "trusted" }, "@you/reviews": { level: "permissive", network: [], host: true } });
  });

  // The chapter shows them as `markdown`, since the book's check processes a document only at the top of its example.
  it.each(["components", "reviews"])("has a pack, %s/, whose document processes without diagnostics", (folder) => {
    const files = Object.fromEntries(readdirSync(join(root, folder)).map((name) => [name, readFileSync(join(root, folder, name), "utf8")]));
    const { diagnostics } = process({ path: "index.comp.md", files, now: "2026-10-31T09:00:00Z" });
    expect(diagnostics.map(({ code, message }) => `${code} ${message}`)).toEqual([]);
  });

  it("has a document that processes without diagnostics, with the app's packs", async () => {
    const { request } = await loadApp(root);
    const text = readFileSync(join(root, "notes.comp.md"), "utf8");
    const { diagnostics } = process({ ...request, path: "notes.comp.md", files: { "notes.comp.md": text }, now: "2026-10-31T09:00:00Z" });
    expect(diagnostics.map(({ code, message }) => `${code} ${message}`)).toEqual([]);
  });

  describe.each(BROWSERS)("in %s", (_name, type, launch) => {
    let browser: Browser;

    beforeAll(async () => {
      browser = await type.launch({ headless: true, ...launch });
    });

    afterAll(async () => {
      await browser.close();
    });

    it("mounts stars in the page, follows its click, and gives reviews the app's answer through host", async () => {
      const { page, errors } = await opened(browser, `http://localhost:${String(port)}/`);
      const stars = page.locator("#page [role=radiogroup] button");
      await expect.poll(() => stars.count(), { timeout: 20_000 }).toBe(5);
      expect(await page.locator("#page").innerText()).toContain("My rating: 3 of 5.");
      const reviews = page.frameLocator("#page iframe[title=reviews]").locator("body");
      await expect.poll(() => reviews.innerText(), { timeout: 20_000 }).toContain("12 readers gave it 4.2 of 5 on average.");
      expect(await page.locator("#page iframe").count()).toBe(1);
      expect(await page.locator("#page iframe").getAttribute("src")).toMatch(/\/compmd\/frames\/[0-9a-f]{16}\/permissive\.html$/);
      await page.locator('#page [aria-label="5 of 5"]').click();
      await expect.poll(() => page.locator("#page").innerText()).toContain("My rating: 5 of 5.");
      expect(await stars.allInnerTexts()).toEqual(["★", "★", "★", "★", "★"]);
      expect(errors).toEqual([]);
      await page.close();
    });

    it("mounts spoiler in the page, its content hidden until the reader asks, and hidden again", async () => {
      const { page, errors } = await opened(browser, `http://localhost:${String(port)}/`);
      const spoiler = page.locator("#page button[aria-expanded]");
      const ending = page.locator("#page").getByText("Estraven is shot at the border");
      await expect.poll(() => spoiler.innerText(), { timeout: 20_000 }).toBe("Show the ending");
      // `reviews`, above the spoiler, takes its height once its frame has drawn, and moves the spoiler down: the click
      // waits for the page to settle, as a reader's would.
      await page.locator("#page[data-compmd-settled]").waitFor({ timeout: 20_000 });
      expect(await ending.isVisible()).toBe(false);
      await spoiler.click();
      await expect.poll(() => ending.isVisible()).toBe(true);
      expect(await spoiler.innerText()).toBe("Hide the ending");
      expect(await spoiler.getAttribute("aria-expanded")).toBe("true");
      await spoiler.click();
      await expect.poll(() => ending.isVisible()).toBe(false);
      expect(await spoiler.innerText()).toBe("Show the ending");
      expect(errors).toEqual([]);
      await page.close();
    });
  });
});

describe("the two page modes", () => {
  const port = 4221;
  let server: PreviewServer;

  beforeAll(async () => {
    server = await served("two-modes", port);
  });

  afterAll(async () => {
    await server.close();
  });

  /** The frames' folder in the build's `compmd/`, `frames/<hash>/`: the build's only one. */
  const frames = (): string => {
    const builds = readdirSync(join(OUT, "two-modes/compmd/frames"));
    expect(builds).toHaveLength(1);
    return `frames/${builds[0] ?? ""}/`;
  };

  it("writes the frame documents and their runtime into the build's compmd/frames/<hash>/", () => {
    for (const file of ["strict.html", "permissive.html", "runtime.js", "library-definition.json"]) expect(existsSync(join(OUT, "two-modes/compmd", frames(), file)), file).toBe(true);
  });

  it("serves compmd/ with the headers the frames need", async () => {
    const response = await fetch(`http://localhost:${String(port)}/compmd/${frames()}strict.html`);
    expect(response.headers.get("access-control-allow-origin")).toBe("*");
    expect(response.headers.get("content-security-policy")).toBe("sandbox allow-scripts");
    const runtime = await fetch(`http://localhost:${String(port)}/compmd/${frames()}runtime.js`);
    expect(runtime.headers.get("access-control-allow-origin")).toBe("*");
  });

  describe.each(BROWSERS)("in %s", (_name, type, launch) => {
    let browser: Browser;

    beforeAll(async () => {
      browser = await type.launch({ headless: true, ...launch });
    });

    afterAll(async () => {
      await browser.close();
    });

    it("renders the document in one frame, and in the page with a frame per use, and both follow an input", async () => {
      const { page, errors } = await opened(browser, `http://localhost:${String(port)}/`);
      const framed = page.frameLocator("#frame iframe").locator("body");
      await expect.poll(() => framed.innerText(), { timeout: 20_000 }).toContain("Tests: 90%");
      expect(await page.locator("#frame iframe").count()).toBe(1);
      expect(await page.locator("#host h1").innerText()).toBe("Release status");
      const meters = page.locator("#host iframe[title=meter]");
      await expect.poll(() => meters.count(), { timeout: 20_000 }).toBe(2);
      expect(await meters.evaluateAll((frames) => frames.map((frame) => frame.getAttribute("src")?.split("/").pop()))).toEqual(["strict.html", "strict.html"]);
      await expect.poll(() => page.frameLocator("#host iframe[title=meter] >> nth=0").locator("body").innerText(), { timeout: 20_000 }).toContain("Docs: 72%");
      await page.locator("#host input[type=number]").fill("40");
      await expect.poll(() => framed.innerText()).toContain("Docs: 40%");
      await expect.poll(() => page.frameLocator("#host iframe[title=meter] >> nth=0").locator("body").innerText()).toContain("Docs: 40%");
      expect(errors).toEqual([]);
      await page.close();
    });
  });
});

describe("the developer tools, on the dev server", () => {
  const root = join(EXAMPLES, "devtools");
  const port = 4222;
  let server: ViteDevServer;

  beforeAll(async () => {
    server = await createServer({ root, configFile: join(root, "vite.config.ts"), logLevel: "warn", cacheDir: join(OUT, "devtools.cache"), server: { port, strictPort: true } });
    await server.listen();
  });

  afterAll(async () => {
    await server.close();
  });

  describe.each(BROWSERS)("in %s", (_name, type, launch) => {
    let browser: Browser;

    beforeAll(async () => {
      browser = await type.launch({ headless: true, ...launch });
    });

    afterAll(async () => {
      await browser.close();
    });

    it("opens the inspector under the page, which follows the document the app processes again", async () => {
      const { page, errors } = await opened(browser, `http://localhost:${String(port)}/`);
      const framed = page.frameLocator("#page iframe").locator("body");
      await expect.poll(() => framed.innerText(), { timeout: 20_000 }).toContain("1 of 4 done");
      const inspector = page.locator("#inspector");
      expect(await inspector.getByRole("tab", { name: "Page", exact: true }).count()).toBe(0);
      await inspector.getByRole("tab", { name: "Data", exact: true }).click({ timeout: 20_000 });
      await expect.poll(() => inspector.locator(".data").innerText(), { timeout: 20_000 }).toMatch(/done[\s\S]*1/);
      await page.frameLocator("#page iframe").locator("input[type=number]").fill("3");
      await expect.poll(() => framed.innerText()).toContain("3 of 4 done");
      await expect.poll(() => inspector.locator(".data").innerText(), { timeout: 20_000 }).toMatch(/done[\s\S]*3/);
      await inspector.getByRole("tab", { name: "Sandbox", exact: true }).click();
      await expect.poll(() => inspector.locator(".sandbox").innerText(), { timeout: 20_000 }).toMatch(/frame 1\s+strict\s+the page/);
      expect(errors).toEqual([]);
      await page.close();
    });

    it("embeds the editor: the page follows the text, and a diagnostic's fix mends it", async () => {
      const { page, errors } = await opened(browser, `http://localhost:${String(port)}/editor.html`);
      const tools = page.locator("#tools");
      const framed = tools.frameLocator(".page-frame iframe").locator("body");
      await expect.poll(() => framed.innerText(), { timeout: 20_000 }).toContain("1 of 4 done");
      const text = tools.getByRole("textbox", { name: "The document's text" });
      await text.focus();
      await page.keyboard.press("ControlOrMeta+End");
      await page.keyboard.type("\nAnd a party.");
      await expect.poll(() => framed.innerText()).toContain("And a party.");
      await text.fill((await text.inputValue()).replace("border=subtle", "border=subtl"));
      const diagnostics = tools.getByRole("list", { name: "Diagnostics" });
      await expect.poll(() => diagnostics.innerText(), { timeout: 20_000 }).toContain("F009");
      await diagnostics.getByRole("button", { name: "Use subtle" }).click();
      expect(await text.inputValue()).toContain("border=subtle");
      await expect.poll(() => diagnostics.count()).toBe(0);
      expect(errors).toEqual([]);
      await page.close();
    });
  });
});
