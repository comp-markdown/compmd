// The guide's chapters against what they claim: each block of code marked `snippet=<file>` is that example file's text,
// each CompMD document in a chapter processes without diagnostics, the contents lists every chapter, and the examples
// build and run in Chrome and WebKit as the chapters say.
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { initSync, process } from "@compmd/processor";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions } from "playwright";
import { build, preview, type PreviewServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { compmd } from "@compmd/vite";

const GUIDE = join(import.meta.dirname, "..");
const EXAMPLES = join(GUIDE, "examples");
const CHAPTERS = readdirSync(GUIDE).filter((name) => /^\d\d-.+\.md$/.test(name)).sort();

/** The fenced blocks of `markdown`: their info string and their text. */
function blocksOf(markdown: string): { info: string; text: string }[] {
  const blocks: { info: string; text: string }[] = [];
  const lines = markdown.split("\n");
  for (let i = 0; i < lines.length; i++) {
    const opener = /^(`{3,})(.*)$/.exec(lines[i] ?? "");
    if (!opener) continue;
    const [, fence = "", info = ""] = opener;
    const end = lines.findIndex((line, j) => j > i && line.startsWith(fence) && line.trim() === fence);
    blocks.push({ info: info.trim(), text: lines.slice(i + 1, end).join("\n") });
    i = end;
  }
  return blocks;
}

beforeAll(() => {
  initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) });
});

describe("the chapters", () => {
  it("are numbered from one, with no gap", () => {
    expect(CHAPTERS.map((name) => Number(name.slice(0, 2)))).toEqual(CHAPTERS.map((_, i) => i + 1));
  });

  for (const chapter of CHAPTERS) {
    const markdown = readFileSync(join(GUIDE, chapter), "utf8");

    it(`${chapter}: its code is its examples' files`, () => {
      for (const { info, text } of blocksOf(markdown)) {
        const file = /\bsnippet=(\S+)/.exec(info)?.[1];
        if (file === undefined) continue;
        expect(text, file).toBe(readFileSync(join(EXAMPLES, file), "utf8").replace(/\n$/, ""));
      }
    });

    it(`${chapter}: its CompMD documents process without diagnostics`, () => {
      for (const { info, text } of blocksOf(markdown).filter(({ info }) => info.split(" ")[0] === "md")) {
        const { diagnostics } = process({ path: "doc.comp.md", files: { "doc.comp.md": text }, now: "2026-10-31T09:00:00Z" });
        expect(diagnostics.map(({ code, message }) => `${code} ${message}`), `${chapter}: ${info}`).toEqual([]);
      }
    });
  }

  it("show every file of their examples", () => {
    const shown = new Set(CHAPTERS.flatMap((chapter) => blocksOf(readFileSync(join(GUIDE, chapter), "utf8")).map(({ info }) => /\bsnippet=(\S+)/.exec(info)?.[1])));
    const files = readdirSync(EXAMPLES, { recursive: true, withFileTypes: true })
      .filter((entry) => entry.isFile())
      .map((entry) => join(entry.parentPath, entry.name).slice(EXAMPLES.length + 1).split("\\").join("/"));
    expect(files.length).toBeGreaterThan(0);
    for (const file of files) expect(shown.has(file), file).toBe(true);
  });

  it("are each listed in the contents, which links only to chapters that are there", () => {
    const contents = readFileSync(join(GUIDE, "README.md"), "utf8");
    const linked = [...contents.matchAll(/\]\((\d\d-[^)]+\.md)\)/g)].map((match) => match[1]);
    expect(linked).toEqual(CHAPTERS);
    for (const file of linked) expect(existsSync(join(GUIDE, file ?? ""))).toBe(true);
  });
});

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

describe("the quick start", () => {
  const root = join(EXAMPLES, "quick-start");
  const out = join(import.meta.dirname, ".out/quick-start");
  const port = 4201;
  let server: PreviewServer;

  beforeAll(async () => {
    const config = { root, configFile: false as const, logLevel: "warn" as const, plugins: [compmd()], build: { outDir: out, emptyOutDir: true, target: "es2024" } };
    await build(config);
    server = await preview({ ...config, preview: { port, strictPort: true } });
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

    it("renders the document, and follows its input as the chapter says", async () => {
      const page = await browser.newPage();
      const errors: string[] = [];
      page.on("pageerror", (error) => errors.push(error.message));
      await page.goto(`http://localhost:${String(port)}/`);
      const frame = page.frameLocator("iframe");
      await expect.poll(() => frame.locator("body").innerText(), { timeout: 20_000 }).toContain("1 of 4 done");
      expect(await frame.locator("li").allInnerTexts()).toEqual(["Write the outline", "Draft the first part", "Ask for a review", "Publish"]);
      await frame.locator("input[type=number]").fill("3");
      await expect.poll(() => frame.locator("body").innerText()).toContain("3 of 4 done");
      expect(errors).toEqual([]);
      await page.close();
    });
  });
});
