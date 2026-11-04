// The playground's document of a pack from npm, end to end: the playground built and served as the site is, with npm in
// its configuration replaced by a local registry that serves this repository's Mermaid pack, packed as it is published.
// In the installed Chrome, light and dark: the pack is fetched once, checked, and its diagrams drawn in the page's frame.
import { compmd } from "@compmd/vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readdirSync, readFileSync } from "node:fs";
import { mkdir, rm } from "node:fs/promises";
import { createServer as createHttpServer, type Server } from "node:http";
import { join } from "node:path";
import { gunzipSync } from "node:zlib";
import { chromium, type Browser, type Frame, type Page } from "playwright-core";
import { build, preview, type InlineConfig, type PreviewServer } from "vite";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

const EDITOR = join(import.meta.dirname, "..");
const MERMAID = join(EDITOR, "../packs/mermaid");
const OUT = join(import.meta.dirname, ".out");
const REGISTRY_PORT = 4241;
const PLAYGROUND_PORT = 4242;
const NAME = "@comp-markdown/mermaid";

let registry: Server;
let playground: PreviewServer;
let browser: Browser;
/** The registry's requests, by path. */
const requests: string[] = [];

/** The sum of the sizes of the files in a gzipped tar: the unpacked size npm's metadata states. */
function unpackedSize(gzipped: Buffer): number {
  const tar = gunzipSync(gzipped);
  let total = 0;
  for (let at = 0; at + 512 <= tar.length; ) {
    if (tar[at] === 0) break;
    const size = parseInt(tar.subarray(at + 124, at + 136).toString("ascii").replace(/\0.*$/, "").trim() || "0", 8);
    if (tar[at + 156] === 0x30 || tar[at + 156] === 0) total += size;
    at += 512 + Math.ceil(size / 512) * 512;
  }
  return total;
}

beforeAll(async () => {
  // The pack's tarball as `pnpm pack` writes it for npm, from its build.
  await rm(OUT, { recursive: true, force: true });
  await mkdir(OUT, { recursive: true });
  execFileSync("pnpm", ["pack", "--pack-destination", OUT], { cwd: MERMAID, stdio: "pipe" });
  const tarball = readFileSync(join(OUT, readdirSync(OUT).find((name) => name.endsWith(".tgz")) ?? ""));
  const { version } = JSON.parse(readFileSync(join(MERMAID, "package.json"), "utf8")) as { version: string };
  const integrity = `sha512-${createHash("sha512").update(tarball).digest("base64")}`;
  registry = createHttpServer((request, response) => {
    const path = decodeURIComponent(new URL(request.url ?? "/", "http://localhost").pathname);
    requests.push(path);
    // The reader's browser asks from the playground's origin: npm answers any origin.
    response.setHeader("Access-Control-Allow-Origin", "*");
    if (path === "/tarball.tgz") return response.writeHead(200, { "content-type": "application/octet-stream" }).end(tarball);
    if (path !== `/${NAME}`) return response.writeHead(404).end();
    const dist = { tarball: `http://127.0.0.1:${String(REGISTRY_PORT)}/tarball.tgz`, integrity, unpackedSize: unpackedSize(tarball) };
    response.writeHead(200, { "content-type": "application/vnd.npm.install-v1+json" });
    response.end(JSON.stringify({ name: NAME, versions: { [version]: { version, dist } } }));
  });
  await new Promise<void>((resolve) => registry.listen(REGISTRY_PORT, "127.0.0.1", resolve));
  // The playground's own configuration, but for its registry, which is the local one; built as the site is, and
  // served from the build, since a worker's bundle is built apart from the page's.
  const config = JSON.parse(readFileSync(join(EDITOR, "compmd.config.json"), "utf8")) as Record<string, unknown>;
  const site: InlineConfig = {
    configFile: false,
    root: EDITOR,
    base: "./",
    logLevel: "warn",
    plugins: [svelte(), compmd({ config: { ...config, registries: { "*": `http://127.0.0.1:${String(REGISTRY_PORT)}/` } } })],
    worker: { format: "es" },
    build: { target: "es2024", outDir: join(OUT, "site"), emptyOutDir: true },
    preview: { port: PLAYGROUND_PORT, strictPort: true, cors: { origin: "*" } },
  };
  await build(site);
  playground = await preview(site);
  browser = await chromium.launch({ channel: "chrome", headless: true });
});

afterAll(async () => {
  await browser.close();
  await playground.close();
  registry.closeAllConnections();
  await new Promise((resolve) => registry.close(resolve));
});

/** The frame whose content matches `selector`, once there is one. */
async function frameWith(page: Page, selector: string): Promise<Frame> {
  let found: Frame | undefined;
  await expect
    .poll(
      async () => {
        for (const frame of page.frames()) if ((await frame.locator(selector).count().catch(() => 0)) > 0) found = frame;
        return found !== undefined;
      },
      { timeout: 45_000 },
    )
    .toBe(true);
  return found as Frame;
}

const drawing = (label: string) => `[role="img"][aria-label="${label}"] svg`;

describe("the document of data from the web", () => {
  const LATEST = "https://registry.npmjs.org/@comp-markdown/mermaid/latest";

  it("shows the fallback, asks the reader in the playground's page, and with their Allow shows npm's data", async () => {
    const page = await browser.newPage();
    const asked: string[] = [];
    // npm's answer, from the test: nothing reaches the network.
    await page.route(LATEST, async (route) => {
      asked.push(route.request().url());
      await route.fulfill({ status: 200, contentType: "application/json", headers: { "access-control-allow-origin": "*" }, body: JSON.stringify({ name: NAME, version: "9.9.9", description: "Fetched just now." }) });
    });
    await page.goto(`http://localhost:${String(PLAYGROUND_PORT)}/?doc=remote`);
    const frame = await frameWith(page, "text=0.1.1 (a saved copy)");
    expect(await page.locator(".diagnostics li").allTextContents()).toEqual([expect.stringContaining("F028")]);
    const question = page.locator("dialog.compmd-origin-question");
    await expect.poll(() => question.isVisible(), { timeout: 10_000 }).toBe(true);
    expect(await question.textContent()).toContain("This document wants to load data from registry.npmjs.org. Allow?");
    await question.getByRole("button", { name: "Allow", exact: true }).click();
    await expect.poll(() => frame.locator("text=9.9.9").count(), { timeout: 10_000 }).toBe(1);
    expect(await frame.locator("text=Fetched just now.").count()).toBe(1);
    await expect.poll(() => page.locator(".diagnostics li").count(), { timeout: 10_000 }).toBe(0);
    expect(asked).toEqual([LATEST]);
    // Remembered: the next visit isn't asked again, and fetches at once.
    await page.reload();
    await frameWith(page, "text=9.9.9");
    expect(await question.count()).toBe(0);
    await page.close();
  });
});

describe.each(["light", "dark"] as const)("the document of a pack from npm, %s", (scheme) => {
  it("fetches the pack, checks it, and draws its diagrams in the page, with no diagnostics", async () => {
    const page = await browser.newPage({ colorScheme: scheme });
    const errors: string[] = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto(`http://localhost:${String(PLAYGROUND_PORT)}/?doc=npm`);
    const frame = await frameWith(page, drawing("An idea becomes a draft, is reviewed, then published"));
    expect(await frame.locator(drawing("The steps: Idea, Draft, Review and Published")).innerHTML()).toContain("Published");
    expect(await frame.locator(drawing("The reader asks the playground for the pack; the playground asks npm, checks it, and draws")).innerHTML()).toContain("Open the document");
    expect(await frame.evaluate(() => getComputedStyle(document.documentElement).colorScheme)).toBe(scheme);
    // The diagnostics the page lists, once it opened again with the pack: none.
    await expect.poll(() => page.locator(".diagnostics li").count(), { timeout: 10_000 }).toBe(0);
    expect(requests).toContain(`/${NAME}`);
    expect(requests).toContain("/tarball.tgz");
    expect(errors).toEqual([]);
    await page.close();
  });
});
