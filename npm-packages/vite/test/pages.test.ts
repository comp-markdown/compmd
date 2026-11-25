// One page reaching two configurations (docs/vite-per-page-config.md, decision 5): a page whose own script imports the
// root's `virtual:compmd`, and a helper in `posts/`, which has a configuration of its own, imports that one. The build
// stops, naming the modules, and the dev server shows the same in the page.
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { chromium } from "playwright";
import { build, createServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { compmd } from "../src/plugin.ts";

const ROOT = join(import.meta.dirname, ".out/two-configurations");
const PORT = 4210;

const FILES: Record<string, string> = {
  "index.html": '<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Two</title></head><body><script type="module" src="./main.ts"></script></body></html>',
  "main.ts": 'import { request } from "virtual:compmd";\nimport { posts } from "./posts/helper.ts";\ndocument.body.dataset.loaded = String(Boolean(request && posts));\n',
  "posts/compmd.config.json": "{}",
  "posts/helper.ts": 'import { request } from "virtual:compmd";\nexport const posts = request;\n',
};

beforeAll(() => {
  rmSync(ROOT, { recursive: true, force: true });
  for (const [path, text] of Object.entries(FILES)) {
    mkdirSync(join(ROOT, path, ".."), { recursive: true });
    writeFileSync(join(ROOT, path), text);
  }
});

afterAll(() => {
  rmSync(ROOT, { recursive: true, force: true });
});

describe("a page that reaches two configurations", () => {
  it("stops the build, naming the configurations and the modules that import them", async () => {
    const building = build({ root: ROOT, configFile: false, logLevel: "silent", plugins: [compmd()], build: { outDir: join(ROOT, "dist"), emptyOutDir: true, target: "es2024" } });
    await expect(building).rejects.toThrow(/index\.html reaches two configurations[\s\S]*the root's configuration, imported by main\.ts[\s\S]*posts\/'s configuration, imported by posts\/helper\.ts/);
  }, 120_000);

  it("shows the same in the page on the dev server", async () => {
    const dev = await createServer({ root: ROOT, configFile: false, logLevel: "silent", plugins: [compmd()], server: { port: PORT, strictPort: true } });
    await dev.listen();
    const browser = await chromium.launch({ headless: true, channel: "chrome" });
    try {
      const page = await browser.newPage();
      await page.goto(`http://localhost:${String(PORT)}/`);
      const overlay = page.locator("vite-error-overlay");
      await expect.poll(() => overlay.count(), { timeout: 15_000 }).toBe(1);
      expect(await overlay.evaluate((element) => element.shadowRoot?.textContent ?? "")).toContain("reaches two configurations");
    } finally {
      await browser.close();
      await dev.close();
    }
  }, 60_000);
});
