// The guides' chapters against what they claim, book by book: each block of code marked `snippet=<file>` is that
// example file's text, each example file is shown, each CompMD document in a chapter processes without diagnostics,
// the contents lists every chapter, and the examples build and run in Chrome and WebKit as the chapters say.
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { initSync, process } from "@compmd/processor";
import { chromium, webkit, type Browser, type BrowserType, type LaunchOptions } from "playwright";
import { build, preview, type PreviewServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { compmd } from "@compmd/vite";

const GUIDES = join(import.meta.dirname, "..");
const BOOKS = ["authors", "developers"];

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

describe.each(BOOKS)("the %s' guide", (book) => {
  const folder = join(GUIDES, book);
  const examples = join(folder, "examples");
  const chapters = readdirSync(folder).filter((name) => /^\d\d-.+\.md$/.test(name)).sort();

  it("numbers its chapters from one, with no gap", () => {
    expect(chapters.map((name) => Number(name.slice(0, 2)))).toEqual(chapters.map((_, i) => i + 1));
  });

  for (const chapter of chapters) {
    const markdown = readFileSync(join(folder, chapter), "utf8");

    it(`${chapter}: its code is its examples' files`, () => {
      for (const { info, text } of blocksOf(markdown)) {
        const file = /\bsnippet=(\S+)/.exec(info)?.[1];
        if (file === undefined) continue;
        expect(text, file).toBe(readFileSync(join(examples, file), "utf8").replace(/\n$/, ""));
      }
    });

    it(`${chapter}: its CompMD documents process without diagnostics`, () => {
      for (const { info, text } of blocksOf(markdown).filter(({ info }) => info.split(" ")[0] === "md")) {
        const { diagnostics } = process({ path: "doc.comp.md", files: { "doc.comp.md": text }, now: "2026-10-31T09:00:00Z" });
        expect(diagnostics.map(({ code, message }) => `${code} ${message}`), `${chapter}: ${info}`).toEqual([]);
      }
    });
  }

  it("shows every file of its examples", () => {
    const shown = new Set(chapters.flatMap((chapter) => blocksOf(readFileSync(join(folder, chapter), "utf8")).map(({ info }) => /\bsnippet=(\S+)/.exec(info)?.[1])));
    const files = existsSync(examples)
      ? readdirSync(examples, { recursive: true, withFileTypes: true })
          .filter((entry) => entry.isFile())
          .map((entry) => join(entry.parentPath, entry.name).slice(examples.length + 1).split("\\").join("/"))
      : [];
    for (const file of files) expect(shown.has(file), file).toBe(true);
  });

  it("lists each chapter in its contents, which links only to chapters that are there", () => {
    const contents = readFileSync(join(folder, "README.md"), "utf8");
    const linked = [...contents.matchAll(/\]\((\d\d-[^)]+\.md)\)/g)].map((match) => match[1]);
    expect(linked).toEqual(chapters);
    for (const file of linked) expect(existsSync(join(folder, file ?? ""))).toBe(true);
  });
});

it("lists every book on the shelf", () => {
  const shelf = readFileSync(join(GUIDES, "README.md"), "utf8");
  for (const book of BOOKS) expect(shelf).toContain(`](${book}/README.md)`);
});

const BROWSERS: [string, BrowserType, LaunchOptions][] = [
  ["Chrome", chromium, { channel: "chrome" }],
  ["WebKit", webkit, {}],
];

describe("the developers' quick start", () => {
  const root = join(GUIDES, "developers/examples/quick-start");
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
